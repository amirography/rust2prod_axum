use rust2prod_amir::{configuration::get_configuration, run};
use sqlx::{Connection, PgConnection};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("could not connect to the database");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .expect("failed to bind to port");

    run(listener, connection).await?;
    Ok(())
}
