// main.rs

use reqwest::Client;
use serde_json::Value;
use std::{error::Error, time::Duration};
use tokio::time::sleep;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match config::AppConfig::new() {
        Ok(config) => {
            println!("Configuration loaded: {:?}", config);
            // Here, you can proceed with using the configuration in your application logic
        },
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1); // Exit if configuration cannot be loaded
        }
    }
    
    let client = Client::new();

    loop { 
        match fetch_order_book(&client).await {
            Ok(order_book) => {
                println!("{:?}", order_book);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn fetch_order_book(client: &Client) -> Result<Value, Box<dyn Error>> {
    let response = client.get("https://api.binance.com/api/v3/depth?symbol=BNBUSDC").send().await?;
    if response.status().is_success() {
        let order_book = response.json::<Value>().await?;
        Ok(order_book)
    } else {
        Err("Failed to fetch order book".into())
    }
}

async fn execute_trades(client: &Client) -> Result<(), Box<dyn Error>> {
    let order_book = fetch_order_book(client).await?;

    // Example logic for determining your bid and ask prices
    let bid_price = order_book["bids"][0]["price"].as_f64().unwrap() * 0.999; // 0.1% below the best bid
    let ask_price = order_book["asks"][0]["price"].as_f64().unwrap() * 1.001; // 0.1% above the best ask

    // Example functions to place orders
    place_order(client, bid_price, "buy", "0.01 BTC").await?;
    place_order(client, ask_price, "sell", "0.01 BTC").await?;

    Ok(())
}

async fn place_order(client: &Client, price: f64, side: &str, amount: &str) -> Result<(), Box<dyn Error>> {
    let order = client.post("YOUR_ORDER_PLACEMENT_URL")
        .json(&serde_json::json!({
            "price": price,
            "side": side,
            "amount": amount,
            "type": "limit"
        }))
        .send()
        .await?;

    println!("Order placed: {:?}", order);
    Ok(())
}

