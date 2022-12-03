use rust2prod_amir::{configuration::get_configuration, run};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("failed to read configuration");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .expect("failed to bind to port");

    run(listener).await?;
    Ok(())
}
