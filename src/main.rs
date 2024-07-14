// main.rs
use config::Config;
use reqwest::Client;
use serde_json::Value;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use prettytable::{Table, row, cell};

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
    let settings = Config::builder()
        // Add in `src/Settings.toml`
        .add_source(config::File::with_name("src/Settings"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let api_key = settings.get_string("api_key")?;
    let api_url = settings.get_string("api_url")?;
    println!("API Key: {}", api_key);
    println!("API URL: {}", api_url);

    let client = Client::new();

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
                visualize_order_book(&order_book);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn fetch_order_book(client: &Client) -> Result<Value, Box<dyn Error>> {
    let response = client
        .get("https://api.binance.com/api/v3/depth?symbol=BNBUSDC")
        .send()
        .await?;
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

async fn place_order(
    client: &Client,
    price: f64,
    side: &str,
    amount: &str,
) -> Result<(), Box<dyn Error>> {
    let order = client
        .post("YOUR_ORDER_PLACEMENT_URL")
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

fn visualize_order_book(order_book: &OrderBook) {
    let mut table = Table::new();
    table.add_row(row!["BIDS", "ASKS"]);
    
    let max_len = std::cmp::max(order_book.bids.len(), order_book.asks.len());
    for i in 0..max_len {
        let bid = order_book.bids.get(i).map(|o| format!("{} @ {}", o.quantity, o.price)).unwrap_or_default();
        let ask = order_book.asks.get(i).map(|o| format!("{} @ {}", o.quantity, o.price)).unwrap_or_default();
        table.add_row(row![bid, ask]);
    }

    table.printstd(); // Print the table to standard output
}
