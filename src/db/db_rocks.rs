pub struct Ctx {
    
}

impl Ctx {
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