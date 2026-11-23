#[tokio::main]
async fn main() {
    kavionics_cli::run().await.unwrap_or_else(|err| {
        tracing::error!("Application error: {:#}", err);
        std::process::exit(1);
    });
}
