use std::collections::HashMap;

pub fn parse_query_params(url: &str) -> HashMap<String, String> {
    let mut params: HashMap<String, String> = HashMap::new();
    
    if let Some(query_string) = url.split('?').nth(1) {
        for pair in query_string.split('&') {
            let mut key_value = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
    }
    params
}