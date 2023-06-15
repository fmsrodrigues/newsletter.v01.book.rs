use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("localhost:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind server to port 8000");
    run(listener, connection)?.await
}
