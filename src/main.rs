mod server;
use axum::Router;
use dify_client::{Client as DifyClient, Config as DifyConfig};
use std::env;
use std::time::Duration;
use tokio::{net::TcpListener, runtime};

fn main() {
    let _ = dotenvy::dotenv();
    env_logger::init();

    let workers_num = env::var("WORKERS_NUM")
        .ok()
        .and_then(|f| f.parse::<usize>().ok())
        .unwrap_or(num_cpus::get());
    runtime::Builder::new_multi_thread()
        .worker_threads(workers_num)
        .enable_all()
        .build()
        .unwrap()
        .block_on(init_server());
}

async fn init_server() {
    let host = env::var("HOST").unwrap_or("127.0.0.1".into());
    let port = env::var("PORT").unwrap_or("3000".into());
    let server_url = format!("{host}:{port}");
    let dify_base_url = env::var("DIFY_BASE_URL").unwrap_or("https://api.dify.ai".into());
    let dify_api_key = env::var("DIFY_API_KEY").unwrap_or("your_api_key".into());
    let dify_timeout = env::var("DIFY_TIMEOUT")
        .ok()
        .and_then(|f| f.parse::<u64>().ok())
        .unwrap_or(10);

    // dify client
    let dify = DifyClient::new_with_config(DifyConfig {
        base_url: dify_base_url.clone(),
        api_key: dify_api_key.clone(),
        timeout: Duration::from_secs(dify_timeout),
    });

    // shared state
    let state = server::AppState { dify };
    let app = Router::new().merge(server::app_routes()).with_state(state);

    let listener = TcpListener::bind(&server_url)
        .await
        .expect("Failed to bind to address");

    show_welcome(&server_url, &dify_base_url, &dify_api_key, dify_timeout);

    axum::serve(listener, app).await.expect("Server Error");
}

fn show_welcome(server_url: &str, dify_base_url: &str, dify_api_key: &str, dify_timeout: u64) {
    println!(
        r#"Welcome to the Dify OpenAI API Server!

- Address Listen: {}
- Dify Base URL:  {}
- Dify API Key:   {}
- Dify Timeout:   {} seconds"#,
        server_url, dify_base_url, dify_api_key, dify_timeout
    )
}
