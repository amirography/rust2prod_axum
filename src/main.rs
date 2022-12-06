use rust2prod_amir::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("could not connect to the database");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .expect("failed to bind to port");

    run(listener, connection_pool).await?;
    Ok(())
}
