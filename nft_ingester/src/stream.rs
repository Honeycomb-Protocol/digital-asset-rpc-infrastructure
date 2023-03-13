use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{error::IngesterError, metric};
use cadence_macros::{is_global_default_set, statsd_count, statsd_gauge};

use log::{error, info};
use plerkle_messenger::{ConsumptionType, Messenger, MessengerConfig, RecvData};
use tokio::{
    sync::{
        mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender}
    },
    task::{JoinHandle, JoinSet},
    time::{self, Duration, Instant},
};
use tokio_stream::{Stream, StreamExt};
pub const HOT_PATH_METRICS_SAMPLE_INTERVAL: u64 = 10;

pub struct MessengerStreamManager {
    config: MessengerConfig,
    stream_key: &'static str,
    message_receiver: JoinSet<Result<(), IngesterError>>,
}
impl MessengerStreamManager {
    pub fn new(stream: &'static str, messenger_config: MessengerConfig) -> Self {
        Self {
            config: messenger_config,
            stream_key: stream,
            message_receiver: JoinSet::new(),
        }
    }

    pub fn listen<T: Messenger>(
        &mut self,
        ct: ConsumptionType,
    ) -> Result<MessengerDataStream, IngesterError> {
        let key = self.stream_key.clone();
        let (stream, send, mut acks) = MessengerDataStream::new();
        let config = self.config.clone();
        let handle = async move {
            let mut metrics_time_sample = Instant::now();
            let mut messenger = T::new(config).await?;
            loop {
                if let Ok(msgs) = acks.try_recv() {
                    let len = msgs.len();
                    if let Err(e) = messenger.ack_msg(&key, &msgs).await {
                        error!("Error acking message: {}", e);
                    }
                    metric! {
                        statsd_count!("ingester.ack", len as i64, "stream" => key);
                    }
                }
                let ct = match ct {
                    ConsumptionType::All => ConsumptionType::All,
                    ConsumptionType::New => ConsumptionType::New,
                    ConsumptionType::Redeliver => ConsumptionType::Redeliver,
                };
                let key = key.clone();
                if let Ok(data) = messenger.recv(&key, ct).await {
                    let l = data.len();
                    if metrics_time_sample.elapsed().as_secs() >= HOT_PATH_METRICS_SAMPLE_INTERVAL {
                        metric! {
                            statsd_gauge!("ingester.batch_size", l as f64, "stream" => key);
                        }
                        metrics_time_sample = Instant::now();
                    }
                    for r in data {
                        if let Err(e) = send.send(r).await {
                            error!("Error forwarding to local stream: {}", e);
                        }
                    }
                }
            }
        };
        self.message_receiver.spawn(handle);
        Ok(stream)
    }
}

pub struct MessengerDataStream {
    ack_sender: UnboundedSender<Vec<String>>,
    message_chan: Receiver<RecvData>,
}

impl MessengerDataStream {
    pub fn new() -> (Self, Sender<RecvData>, UnboundedReceiver<Vec<String>>) {
        let (message_sender, message_chan) = channel::<RecvData>(10);
        let (ack_sender, ack_tracker) = unbounded_channel::<Vec<String>>();
        (
            MessengerDataStream {
                ack_sender,
                message_chan,
            },
            message_sender,
            ack_tracker,
        )
    }

    pub fn ack_sender(&self) -> UnboundedSender<Vec<String>> {
        self.ack_sender.clone()
    }
}

impl Stream for MessengerDataStream {
    type Item = RecvData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.message_chan.poll_recv(cx)
    }
}

pub struct StreamSizeTimer {
    interval: tokio::time::Duration,
    messenger_config: MessengerConfig,
    stream: &'static str,
}

impl StreamSizeTimer {
    pub fn new(
        interval_time: Duration,
        messenger_config: MessengerConfig,
        stream: &'static str,
    ) -> Result<Self, IngesterError> {
        Ok(Self {
            interval: interval_time,
            stream,
            messenger_config: messenger_config,
        })
    }

    pub async fn start<T: Messenger>(&mut self) -> Option<JoinHandle<()>> {
        metric! {
            let i = self.interval.clone();
            let messenger_config = self.messenger_config.clone();
            let stream = self.stream;

           return Some(tokio::spawn(async move {
            let messenger = T::new(messenger_config).await;
            if let Ok(mut messenger) = messenger {
            let mut interval = time::interval(i);
                loop {
                    interval.tick().await;
                    let size = messenger.stream_size(stream).await;
                    match size {
                        Ok(size) => {
                            statsd_gauge!("ingester.stream_size", size, "stream" => stream);
                        }
                        Err(e) => {
                            statsd_count!("ingester.stream_size_error", 1, "stream" => stream);
                            error!("Error getting stream size: {}", e);
                        }
                    }
                }
            };
            }));
        }

        None
    }
}
