use axum::{routing::get, Router};
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::net::SocketAddr;
use redis::{aio::Connection, AsyncCommands, Client as RedisClient};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new().route("/weather/:city", get(get_weather));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_weather(axum::extract::Path(city): axum::extract::Path<String>) -> String {
    let api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY must be set");
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

    #[allow(deprecated)]
    let mut con: Connection = redis_client.get_async_connection().await.unwrap();

    let key = format!("weather_requests:{}", city);
    let _: () = con.incr(&key, 1).await.unwrap();
    let _: () = con.expire(&key, 60).await.unwrap(); // Reset count every 60 seconds

    response.to_string()
}
