use rocksdb::DB;

fn main() {
    let db_path = std::env::args().nth(1).unwrap_or_else(|| "./customer_db".to_string());
    let test_key = std::env::args().nth(2).unwrap_or_else(|| "ethereum:0x1234567890123456789012345678901234567890".to_string());

    println!("Opening RocksDB: {}", db_path);
    let db = DB::open_default(&db_path).expect("Failed to open RocksDB");

    println!("Removing key: {}", test_key);
    match db.delete(&test_key) {
        Ok(_) => println!("✅ Removed test address: {}", test_key),
        Err(e) => eprintln!("❌ Failed to remove: {}", e),
    }
}
