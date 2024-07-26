use {
    crate::{
        config::ConfigGrpc, prom::redis_xadd_status_inc, redis::metrics_xlen, util::create_shutdown,
    },
    anyhow::Context,
    futures::{channel::mpsc, stream::StreamExt, SinkExt},
    log::{debug, error},
    lru::LruCache,
    redis::{streams::StreamMaxlen, RedisResult, Value as RedisValue},
    std::{collections::HashMap, num::NonZeroUsize, sync::Arc, time::Duration},
    tokio::{
        spawn,
        task::JoinSet,
        time::{sleep, Instant},
    },
    tracing::warn,
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::{
        geyser::{SubscribeRequest, SubscribeUpdateTransaction},
        prelude::subscribe_update::UpdateOneof,
        prost::Message,
    },
    yellowstone_grpc_tools::config::GrpcRequestToProto,
};

pub struct GrpcStream {
    endpoint_index: u8,
    update: UpdateOneof,
}

pub async fn try_streaming_grpc_loop(
    config: &ConfigGrpc,
    tx: &mut mpsc::Sender<GrpcStream>,
    endpoint: (u8, &String),
) -> anyhow::Result<()> {
    let mut client = GeyserGrpcClient::build_from_shared(endpoint.1.to_owned())?
        .x_token(config.x_token.clone())?
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .connect()
        .await
        .context("failed to connect to gRPC")?;

    client.ping(1).await?;

    let mut accounts = HashMap::with_capacity(1);
    let mut transactions = HashMap::with_capacity(1);
    accounts.insert("das".to_string(), config.accounts.filter.clone().to_proto());
    transactions.insert(
        "das".to_string(),
        config.transactions.filter.clone().to_proto(),
    );

    let request = SubscribeRequest {
        accounts,
        transactions,
        ..Default::default()
    };
    debug!(
        "subscribing to client {:?}, status {:?}",
        endpoint,
        client.health_check().await?
    );
    match client.subscribe_with_request(Some(request)).await {
        Ok((_subscribe_tx, mut stream)) => {
            debug!("subscribtion sent to client {}", endpoint.1);

            while let Some(resp) = stream.next().await {
                match resp {
                    Ok(msg) => {
                        if let Some(update) = msg.update_oneof {
                            tx.send(GrpcStream {
                                endpoint_index: endpoint.0,
                                update,
                            })
                            .await
                            .expect("Failed to send update to management thread");
                        } else {
                            debug!("unhandled response from {:?}: {:?}", endpoint, msg);
                        }
                    }
                    Err(error) => {
                        debug!("got error from grpc {:?}: {}", endpoint, error);
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        }
        Err(error) => {
            debug!(
                "got error from grpc {:?} while connecting stream: {}",
                endpoint, error
            );
            Err(error.into())
        }
    }
}

pub async fn run(config: ConfigGrpc) -> anyhow::Result<()> {
    let config = Arc::new(config);
    let (tx, mut rx) = mpsc::channel::<GrpcStream>(config.geyser_update_message_buffer_size); // Adjust buffer size as needed

    // Connect to Redis
    let client = redis::Client::open(config.redis.url.clone())?;
    let connection = client.get_multiplexed_tokio_connection().await?;

    // Check stream length for the metrics
    let jh_metrics_xlen = spawn({
        let connection = connection.clone();
        let streams = vec![
            config.accounts.stream.clone(),
            config.transactions.stream.clone(),
        ];
        async move { metrics_xlen(connection, &streams).await }
    });
    tokio::pin!(jh_metrics_xlen);

    // Spawn gRPC client connections
    for (i, endpoint) in config.geyser_endpoints.iter().enumerate() {
        let config = Arc::clone(&config);
        let mut tx = tx.clone();
        let ep = endpoint.clone();
        debug!("client conncted for {}", &ep);
        let mut retry_count: usize = 0;
        spawn(async move {
            while retry_count < 10 {
                match try_streaming_grpc_loop(&config, &mut tx, (i as u8, &ep)).await {
                    Ok(_) => {
                        error!("try_streaming_grpc_loop: Ended unexpectedly");
                    }
                    Err(err) => {
                        error!("try_streaming_grpc_loop: Final Error, {}", err);
                    }
                }
                retry_count += 1;
            }
        });
    }

    // Management thread
    let mut shutdown = create_shutdown()?;
    let mut tasks = JoinSet::new();
    let mut pipe = redis::pipe();
    let mut pipe_accounts = 0;
    let mut pipe_transactions = 0;
    let deadline = sleep(config.redis.pipeline_max_idle);
    tokio::pin!(deadline);

    let mut pending_update_events =
        HashMap::<String, (u8, std::time::SystemTime, SubscribeUpdateTransaction)>::new();
    let mut seen_update_events = LruCache::<String, u64>::new(
        NonZeroUsize::new(config.solana_seen_event_cache_max_size).expect("Non zero value"),
    );

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let result = loop {
        tokio::select! {
            _ = interval.tick() => {
                let desired_time = std::time::SystemTime::now() - std::time::Duration::from_secs(10);
                pending_update_events = pending_update_events.into_iter().fold(HashMap::new(), |mut map, (slot_signature, (endpoint_index, timestamp, transaction))| {

                    if timestamp > desired_time {
                        debug!("[PENDING_LOOP] Retaining tx: {}", &slot_signature);
                        map.insert(slot_signature, (endpoint_index, timestamp, transaction));
                        return map;
                    }

                    debug!("[PENDING_LOOP] Consuming tx: {}", &slot_signature);
                    seen_update_events.put(slot_signature.to_owned(), 1);

                    pipe.xadd_maxlen(
                        &config.transactions.stream,
                        StreamMaxlen::Approx(config.transactions.stream_maxlen),
                        "*",
                        &[(
                            &config.transactions.stream_data_key,
                            transaction.encode_to_vec(),
                        )],
                    );

                    pipe.xadd_maxlen(
                        &String::from("TXN_CACHE"),
                        StreamMaxlen::Approx(config.transactions.stream_maxlen),
                        "*",
                        &[(
                            &config.transactions.stream_data_key,
                            [vec![endpoint_index], transaction.encode_to_vec()].concat(),
                        )],
                    );

                    pipe_transactions += 1;
                    map
                });
            },
            result = &mut jh_metrics_xlen => match result {
                Ok(Ok(_)) => unreachable!(),
                Ok(Err(error)) => break Err(error),
                Err(error) => break Err(error.into()),
            },
            Some(signal) = shutdown.next() => {
                warn!("{signal} received, waiting spawned tasks...");
                break Ok(());
            },
            Some(GrpcStream { endpoint_index, update }) = rx.next() => {
                match update {
                    UpdateOneof::Account(account) => {
                        let slot_pubkey = format!("{}:{}", account.slot, hex::encode(account.account.as_ref().map(|account| account.pubkey.clone()).unwrap_or_default()));

                        if let Some(cache) = seen_update_events.get_mut(&slot_pubkey) {
                            *cache += 1;
                            // seen_update_events.put(slot_pubkey, *cache);
                            continue;
                        } else {
                            debug!("Adding new account: {}", &slot_pubkey);
                            seen_update_events.put(slot_pubkey, 1);
                        };

                        pipe.xadd_maxlen(
                            &config.accounts.stream,
                            StreamMaxlen::Approx(config.accounts.stream_maxlen),
                            "*",
                            &[(&config.accounts.stream_data_key, account.encode_to_vec())],
                        );

                        pipe_accounts += 1;
                    }
                    UpdateOneof::Transaction(transaction) => {

                        if let Some(transaction) = transaction.transaction.as_ref() {
                            if transaction.meta.is_none() || transaction.meta.as_ref().unwrap().err.is_some() {
                                continue;
                            }
                        } else {
                            continue;
                        }


                        let slot_signature = hex::encode(transaction.transaction.as_ref().map(|t| t.signature.clone()).unwrap_or_default()).to_string();

                        if let Some(cache) = seen_update_events.get_mut(&slot_signature) {
                            *cache += 1;
                            continue;
                        }

                        if pending_update_events.get(&slot_signature).is_some() {
                            pending_update_events.insert(slot_signature, (endpoint_index, std::time::SystemTime::now(), transaction));
                            continue;
                        }

                        debug!("Adding new tx: {}", &slot_signature);
                        pending_update_events.remove(&slot_signature);
                        seen_update_events.put(slot_signature, 1);

                        pipe.xadd_maxlen(
                            &config.transactions.stream,
                            StreamMaxlen::Approx(config.transactions.stream_maxlen),
                            "*",
                            &[(&config.transactions.stream_data_key, transaction.encode_to_vec())]
                        );

                        pipe.xadd_maxlen(
                            &String::from("TXN_CACHE"),
                            StreamMaxlen::Approx(config.transactions.stream_maxlen),
                            "*",
                            &[(&config.transactions.stream_data_key, [vec![endpoint_index], transaction.encode_to_vec()].concat())]
                        );

                        pipe_transactions += 1;
                    }
                    _ => continue,
                }
                if pipe_accounts + pipe_transactions >= config.redis.pipeline_max_size {
                    let mut pipe = std::mem::replace(&mut pipe, redis::pipe());
                    let pipe_accounts = std::mem::replace(&mut pipe_accounts, 0);
                    let pipe_transactions = std::mem::replace(&mut pipe_transactions, 0);
                    deadline.as_mut().reset(Instant::now() + config.redis.pipeline_max_idle);

                    tasks.spawn({
                        let mut connection = connection.clone();
                        let config = Arc::clone(&config);
                        async move {
                            let result: RedisResult<RedisValue> =
                                pipe.atomic().query_async(&mut connection).await;

                            let status = result.map(|_| ()).map_err(|_| ());
                            redis_xadd_status_inc(&config.accounts.stream, status, pipe_accounts);
                            redis_xadd_status_inc(&config.transactions.stream, status, pipe_transactions);

                            Ok::<(), anyhow::Error>(())
                        }
                    });
                }
            },
            _ = &mut deadline => {
                if pipe_accounts + pipe_transactions > 0 {
                    let mut pipe = std::mem::replace(&mut pipe, redis::pipe());
                    let pipe_accounts = std::mem::replace(&mut pipe_accounts, 0);
                    let pipe_transactions = std::mem::replace(&mut pipe_transactions, 0);
                    deadline.as_mut().reset(Instant::now() + config.redis.pipeline_max_idle);

                    tasks.spawn({
                        let mut connection = connection.clone();
                        let config = Arc::clone(&config);
                        async move {
                            let result: RedisResult<RedisValue> =
                                pipe.atomic().query_async(&mut connection).await;

                            let status = result.map(|_| ()).map_err(|_| ());
                            redis_xadd_status_inc(&config.accounts.stream, status, pipe_accounts);
                            redis_xadd_status_inc(&config.transactions.stream, status, pipe_transactions);

                            Ok::<(), anyhow::Error>(())
                        }
                    });
                }
            },
        };

        while tasks.len() >= config.redis.max_xadd_in_process {
            if let Some(result) = tasks.join_next().await {
                result??;
            }
        }
    };

    tokio::select! {
        Some(signal) = shutdown.next() => {
            anyhow::bail!("{signal} received, force shutdown...");
        }
        result = async move {
            while let Some(result) = tasks.join_next().await {
                result??;
            }
            Ok::<(), anyhow::Error>(())
        } => result?,
    };

    result
}
