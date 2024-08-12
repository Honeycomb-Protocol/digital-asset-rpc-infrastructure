use redis::{streams::StreamRangeReply, Commands, Value};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    let mut client = redis::Client::open("redis://localhost:6379").unwrap();
    let StreamRangeReply { ids } = client.xrange_all("TXN_CACHE").unwrap();
    let txn_cache = ids
        .into_iter()
        .map(|id| {
            let Value::Data(data) = id.map.get("data").unwrap().to_owned() else {
                unreachable!()
            };
            data
        })
        .collect::<Vec<Vec<u8>>>();
    let file = File::create("TXN_CACHE.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &txn_cache).unwrap();
    writer.flush().unwrap();
}
