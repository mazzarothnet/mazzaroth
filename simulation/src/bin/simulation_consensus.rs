
#[allow(clippy::unwrap_used)]
fn main() {
    let db = rocksdb::DB::open_default("test.db").unwrap();

    //db.put(b"key", b"value").unwrap();
    let value = db.get(b"key").unwrap().unwrap();
    // get all keys
    let keys = db.full_iterator(rocksdb::IteratorMode::Start);
    for key in keys {
        let key = key.unwrap();
        let kk = String::from_utf8(key.0.to_vec()).unwrap();
        let value = String::from_utf8(key.1.to_vec()).unwrap();
        println!("key: {:?}, value: {:?}", kk, value);
    }
    println!("value: {}", String::from_utf8(value).unwrap());
}
