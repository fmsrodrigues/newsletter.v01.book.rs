use std::net::TcpListener;

use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener =
        TcpListener::bind("http://localhost:8000").expect("Failed to bind server to port 8000");
    run(listener)?.await
}
