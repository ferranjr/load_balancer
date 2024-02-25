use crate::domain::models::{Server, Targets};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

pub struct HealthChecker {}

impl HealthChecker {
    pub async fn init(targets: Arc<Mutex<Targets>>) {
        let servers = &targets.lock().await.servers;
        for server in servers.iter() {
            let server = Arc::clone(server);
            tokio::spawn(HealthChecker::healthcheck(server));
        }
    }

    pub async fn healthcheck(server: Arc<Mutex<Server>>) {
        loop {
            let result = {
                let address = server.lock().await.check_status_address();
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(500))
                    .build()
                    .unwrap()
                    .get(address)
                    .send()
                    .await
            };

            let mut server = server.lock().await;
            match result {
                Ok(res) => {
                    server.healthy = res.status().is_success();
                }
                Err(_) => {
                    server.healthy = false;
                }
            }
            time::sleep(Duration::from_millis(200)).await;
        }
    }
}