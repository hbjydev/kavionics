use std::sync::LazyLock;

use tracing_subscriber::{
    EnvFilter, Layer, Registry, filter::Filtered, layer::SubscriberExt, util::SubscriberInitExt,
};

pub const DEFAULT_LOG_FILTER: &str = "info";
pub static LOG_FORMAT: LazyLock<&str> = LazyLock::new(|| {
    let format = std::env::var("KAVIONICS_LOG_FORMAT").unwrap_or_else(|_| "plain".to_string());

    match format.as_str() {
        "json" => "json",
        "pretty" => "pretty",
        _ => "plain",
    }
});

pub type FmtLayer = Filtered<Box<dyn Layer<Registry> + Send + Sync>, EnvFilter, Registry>;

pub fn init_tracing() {
    let log_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));

    let layer = match LOG_FORMAT.as_ref() {
        "json" => tracing_subscriber::fmt::layer().json().boxed(),
        "pretty" => tracing_subscriber::fmt::layer().pretty().boxed(),
        _ => tracing_subscriber::fmt::layer().boxed(),
    }
    .with_filter(log_filter);

    tracing_subscriber::Registry::default().with(layer).init();
}
