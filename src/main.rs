// main.rs
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use prettytable::{cell, format, row, Table};
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::{error::Error, time::Duration};
use tokio::time::sleep;
extern crate chrono;

struct Order {
    price: f64,
    quantity: f64,
}

struct OrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let num_secs = 10;
    let now: DateTime<Utc> = Utc::now();
    let now_str = now.to_rfc3339();

    // Create a new file for each run
    let file_name = format!("order_book_{}.txt", now_str);
    let mut file = BufWriter::new(File::create(file_name)?);

    // Load the configuration from .env
    dotenv().expect("Failed to load .env file");
    let environment = env::var("ENV").expect("ENV must be set");
    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let api_url = env::var("API_URL").expect("API_URL must be set");
    println!("Environment: {}", environment);
    println!("API Key: {}", api_key);
    println!("API URL: {}", api_url);

    let client = Client::new();

    // Get the current UTC datetime
    let divider = "-------------------------";
    let now_str = now.to_rfc3339();
    println!("\n\n\n{}", divider);
    println!("{}", now_str);
    write!(file, "{}\n", now_str)?;

    loop {
        match fetch_order_book(&client).await {
            Ok(order_book) => {
                // println!("{:?}", order_book);
                let bids = order_book["bids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| Order {
                        price: o[0].as_str().unwrap().parse().unwrap(),
                        quantity: o[1].as_str().unwrap().parse().unwrap(),
                    })
                    .collect();
                let asks = order_book["asks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| Order {
                        price: o[0].as_str().unwrap().parse().unwrap(),
                        quantity: o[1].as_str().unwrap().parse().unwrap(),
                    })
                    .collect();
                let order_book = OrderBook { bids, asks };
                visualize_order_book(&order_book, &mut file)?;

                // now mark down market price and date time
                let market_price = fetch_market_price(&client).await?;
                let now: DateTime<Utc> = Utc::now();
                println!("{} Price: {}", now.to_rfc3339(), market_price);
                write!(file, "{} Price: {}\n", now.to_rfc3339(), market_price)?;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        sleep(Duration::from_secs(num_secs)).await;
    }
}

async fn fetch_market_price(client: &Client) -> Result<f64, Box<dyn Error>> {
    let api_url = "https://api.binance.com/api/v3/ticker/price?symbol=BNBUSDC";
    let response = client.get(api_url).send().await?;
    if response.status().is_success() {
        let price = response.json::<Value>().await?["price"]
            .as_str()
            .unwrap()
            .parse()?;
        Ok(price)
    } else {
        Err("Failed to fetch market price".into())
    }
}

async fn fetch_order_book(client: &Client) -> Result<Value, Box<dyn Error>> {
    let api_url = "https://api.binance.com/api/v3/depth?symbol=BNBUSDC&limit=10";
    let response = client.get(api_url).send().await?;
    if response.status().is_success() {
        let order_book = response.json::<Value>().await?;
        Ok(order_book)
    } else {
        Err("Failed to fetch order book".into())
    }
}

async fn _execute_trades(client: &Client, url: &str) -> Result<(), Box<dyn Error>> {
    let order_book = fetch_order_book(client).await?;

    // Example logic for determining your bid and ask prices
    let bid_price = order_book["bids"][0]["price"].as_f64().unwrap() * 0.999; // 0.1% below the best bid
    let ask_price = order_book["asks"][0]["price"].as_f64().unwrap() * 1.001; // 0.1% above the best ask

    // Example functions to place orders
    _place_order(client, url, bid_price, "buy", "0.01 BTC").await?;
    _place_order(client, url, ask_price, "sell", "0.01 BTC").await?;

    Ok(())
}

async fn _place_order(
    client: &Client,
    url: &str,
    price: f64,
    side: &str,
    amount: &str,
) -> Result<(), Box<dyn Error>> {
    let order = client
        .post(url)
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

fn visualize_order_book(order_book: &OrderBook, file: &mut BufWriter<File>) -> io::Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.add_row(row!["BIDS", "ASKS"]);

    let max_len = std::cmp::max(order_book.bids.len(), order_book.asks.len());
    for i in 0..max_len {
        let bid = order_book
            .bids
            .get(i)
            .map(|o| format!("{} @ {}", o.quantity, o.price))
            .unwrap_or_default();
        let ask = order_book
            .asks
            .get(i)
            .map(|o| format!("{} @ {}", o.quantity, o.price))
            .unwrap_or_default();
        table.add_row(row![bid, ask]);
    }

    table.printstd(); // Print the table to standard output

    let table_string = table.to_string();
    writeln!(file, "{}", table_string)?; // Write the formatted table to the file
    file.flush()?;

    Ok(())
}
