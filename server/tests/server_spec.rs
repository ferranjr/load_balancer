use server::startup;
use tokio::net::TcpListener;

#[derive(Debug)]
struct TestApp {
    address: String,
    port: u16,
}

async fn spawn_app() -> Result<TestApp, Box<dyn std::error::Error>> {
    let address = format!("{}:{}", "127.0.0.1", 0);
    let listener = TcpListener::bind(address).await?;
    let local_address = listener.local_addr().unwrap();
    let address = local_address.ip().to_string();
    let port = local_address.port();

    println!("Starting app at port {}", &port);

    tokio::task::spawn(async move {
        startup::run(listener, &"R2D2".to_string())
            .await
            .expect("Failed to start server");
    });

    Ok(TestApp { address, port })
}

#[tokio::test]
async fn server_healthcheck_is_accessible() {
    let app = spawn_app().await.expect("Failed to start the app");
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}:{}/private/status",
            &app.address, &app.port
        ))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn server_unknown_url_is_404() {
    let app = spawn_app().await.expect("Failed to start the app");
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}:{}/random/path", &app.address, &app.port))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 404);
}
