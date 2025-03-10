use axum::{routing::get, Router};
use reqwest::Client;
use serde_json::Value;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client as RedisClient};
use shuttle_axum::ShuttleAxum;
use shuttle_secrets::{SecretStore, Secrets};
use std::net::SocketAddr;

#[shuttle_runtime::main]
async fn main(#[Secrets] secrets: SecretStore) -> ShuttleAxum {
    let api_key = secrets.get("WEATHER_API_KEY").expect("API key missing");

    let app = Router::new().route("/weather/:city", get(move |path| get_weather(path, api_key.clone())));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Starting server on {}", addr);

    Ok(app.into())
}

async fn get_weather(axum::extract::Path(city): axum::extract::Path<String>, api_key: String) -> String {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
        city, api_key
    );

    let response = Client::new()
        .get(&url)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let redis_client = RedisClient::open("redis://127.0.0.1/").unwrap();
    let mut con: MultiplexedConnection = redis_client.get_multiplexed_async_connection().await.unwrap();

    let key = format!("weather_requests:{}", city);
    let _: () = con.incr(&key, 1).await.unwrap();
    let _: () = con.expire(&key, 60).await.unwrap();

    response.to_string()
}
