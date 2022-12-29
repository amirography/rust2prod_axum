use rust2prod::{configuration::get_configuration, startup::run, telemetry};
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber(
        String::from("rust2prod"),
        String::from("info"),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");
    let connection_pool = PgPool::connect_lazy(&configuration.database.connection_string())
        .expect("could not connect to the database");

    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("failed to bind to port");

    run(listener, connection_pool).await?;
    Ok(())
}
