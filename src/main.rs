use rust2prod::{configuration::get_configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber(
        String::from("rust2prod"),
        String::from("info"),
        std::io::stdout,
    );

    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let server = Application::build(configuration).await?;
    server.run_until_stopped().await?;
    Ok(())
}
