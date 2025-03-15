use std::sync::Arc;

use redis::Commands;

const _REDIS_INTERNAL_KEY_: &str = "internals";

pub struct Ctx {
    pub redis_con: Arc<redis::Client>,
    pub rocks_con: Arc<rocksdb::DB>
}

pub struct CtxOptions {
    _args: Vec<String>,
    redis_host: &'static str,
    rocks_db_name: &'static str
}

impl Default for CtxOptions {
    fn default() -> Self {
        Self { 
            _args: std::env::args().collect(), 
            redis_host: "redis://127.0.0.1", 
            rocks_db_name: "spy-db" 
        }
    }
}

impl Ctx {
    #[allow(clippy::new_without_default)]
    pub fn new(options: CtxOptions) -> Self {
        print!("Connecting to Redis server... ");
        // init Redis client
        let rcon = redis::Client::open(options.redis_host);
        if rcon.is_err() {
            eprintln!("Couldn't connect to Redis server! Try again.");
            std::process::exit(1);
        }

        println!("DONE!");
        print!("Connecting to RocksDB server...");

        // init RocksDB client
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        let rocks_con = rocksdb::DB::open_default(options.rocks_db_name).expect("Couldn't connect to RocksDB server! Try again.");

        println!("DONE!");

        Self {
            redis_con: Arc::new(rcon.ok().unwrap()),
            rocks_con: Arc::new(rocks_con)
        }
    }

    pub fn incr_doc_count(&self) -> redis::RedisResult<()> {
        let mut con = self.redis_con.get_connection()?;
        con.hincr(_REDIS_INTERNAL_KEY_, "total_docs", 1)
    }

    pub fn get_internal_value<T>(&self, internal_key: &str) -> Option<T>
    where 
        T: redis::FromRedisValue
    {
        let mut conn = self.redis_con.get_connection().ok()?;

        let result = conn.hget(_REDIS_INTERNAL_KEY_, internal_key);
        result.ok()
    }

    pub fn get_partially_matching_keys(db: &rocksdb::DB, search_query: &str) -> Vec<String> {
        let mut similar_keys: Vec<String> = Vec::new();
        let db_key_iter = db.iterator(rocksdb::IteratorMode::Start);

        for key_value in db_key_iter {
            match key_value {
                Ok((key, _value)) => {
                    let key_str: String = String::from_utf8(key.to_vec()).expect("Invalid UTF-8 for String::from_utf8");

                    if key_str.contains(search_query) {
                        similar_keys.push(key_str);
                    }
                },
                Err(_) => eprintln!("Lookup failed!")
            }
        }
        similar_keys
    }
}