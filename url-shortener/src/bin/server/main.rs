use std::net::TcpListener;
use url_shortener::config::Config;
use url_shortener::domain::urls::service::{Service, ServiceConfig};
use url_shortener::inbound::http::HttpServer;
use url_shortener::outbound::mongo::{MongoClient, MongoDatabase};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;

    // Set Up Mongo Client
    let mongo = MongoClient::new(config.mongo_uri.as_str()).await?;

    // Create Our Service
    let mongo_repository = MongoDatabase::new(mongo, config.mongo_database.as_str());
    let service_config = ServiceConfig::new(3, config.server_base_url.as_str());
    let urls_service = Service::new(mongo_repository, service_config);

    // Create HttpServer
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", config.server_port))?;
    let http_server = HttpServer::new(urls_service, tcp_listener).await?;

    http_server.run_until_stopped().await?;

    Ok(())
}
