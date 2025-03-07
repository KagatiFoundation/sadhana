pub fn start_local_server(port: i32) -> tiny_http::Server {
    tiny_http::Server::http(format!("localhost:{port}")).expect("Failed to a start local server.")
}