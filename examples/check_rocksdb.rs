use rocksdb::{DB, Options};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let db_path = if args.len() >= 2 {
        &args[1]
    } else {
        "./dummy_db"
    };

    println!("===================================");
    println!("🗄️  RocksDB Viewer");
    println!("===================================");
    println!("Path: {}", db_path);
    println!();

    let mut opts = Options::default();
    opts.create_if_missing(false);

    let db = match DB::open_for_read_only(&opts, db_path, false) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Failed to open RocksDB: {}", e);
            eprintln!();
            eprintln!("Tips:");
            eprintln!("  - Make sure RocksDB directory exists");
            eprintln!("  - Check if xScanner has created the database");
            eprintln!("  - Verify the path is correct");
            std::process::exit(1);
        }
    };

    println!("✅ Successfully opened RocksDB");
    println!();

    let mut total_keys = 0;
    let mut address_keys = 0;
    let mut other_keys = 0;

    let iter = db.iterator(rocksdb::IteratorMode::Start);

    println!("=== All Keys and Values ===");
    println!();

    for item in iter {
        match item {
            Ok((key, value)) => {
                total_keys += 1;
                let key_str = String::from_utf8_lossy(&key);
                let value_str = String::from_utf8_lossy(&value);

                // 주소 키 구분
                let is_address = key_str.contains(":0x") ||
                                key_str.contains(":bc1") ||
                                key_str.contains(":tb1") ||
                                key_str.starts_with("ETHEREUM:") ||
                                key_str.starts_with("BITCOIN:");

                if is_address {
                    address_keys += 1;
                    println!("🔑 [ADDRESS] {}", key_str);
                } else {
                    other_keys += 1;
                    println!("🔑 [DATA] {}", key_str);
                }

                // JSON 파싱 시도
                if value_str.starts_with('{') || value_str.starts_with('[') {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value_str) {
                        println!("   {}", serde_json::to_string_pretty(&json).unwrap());
                    } else {
                        println!("   {}", value_str);
                    }
                } else {
                    println!("   {}", value_str);
                }
                println!();
            }
            Err(e) => {
                eprintln!("❌ Error reading item: {}", e);
            }
        }
    }

    if total_keys == 0 {
        println!("⚠️  No keys found in RocksDB");
        println!();
        println!("This could mean:");
        println!("  - Database is empty (newly created)");
        println!("  - Customer addresses not yet synced");
        println!("  - xScanner hasn't processed any blocks yet");
        println!("  - memory_db = true (RocksDB only used when memory_db = false)");
        println!();
        println!("To populate RocksDB:");
        println!("  1. Set config.toml: memory_db = false");
        println!("  2. Run xScanner with PostgreSQL OR");
        println!("  3. Backend sends customer addresses via SQS");
        println!();
    }

    println!("===================================");
    println!("📊 Statistics");
    println!("===================================");
    println!("Total Keys:      {}", total_keys);
    println!("Address Keys:    {} (customer addresses)", address_keys);
    println!("Other Keys:      {} (metadata, state)", other_keys);
    println!();
}
