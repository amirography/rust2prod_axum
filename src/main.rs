use rust2prod_amir::run;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("shiiiiiiiiiiiiiit");
    run(listener).await?;
    Ok(())
}
