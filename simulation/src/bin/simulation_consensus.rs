fn main() {
    let db = rocksdb::DB::open_default("test.db").unwrap();
}
