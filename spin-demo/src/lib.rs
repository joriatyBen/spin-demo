use spin_sdk::http::{Response, Request};
use spin_sdk::{http_component, pg::{self, Decode, ParameterValue}};
use anyhow::{anyhow, Result};
use std::error::Error;
use std::collections::HashMap;
use chrono::prelude::*;

// defined in spin.toml
const DB_URL_ENV: &str = "DB_URL";

#[derive(serde::Deserialize, Debug)]
struct Customer {
    name: String,
    email: String,
    phone: String,
    address: String,
    city: String,
    pin: String,
}

#[derive(serde::Deserialize, Debug)]
struct Cart {
    id: i32,
    name: String,
    image: String,
    price: i32,
    quantity: i32,
}

#[derive(serde::Deserialize, Debug)]
struct Order {
    customer: Customer,
    checkout: Vec<Cart>,
    #[serde(rename(deserialize = "orderTotal"))]
    order_total: String,
}

#[http_component]
async fn order_request(req: Request) -> Result<Response, Box<dyn Error>> {
    let start_time: DateTime<Utc> = Utc::now();
    let mut ordered_products: HashMap<String, i32> = HashMap::new();
    match serde_json::from_slice::<Order>(req.body().as_ref()) {
        Ok(order) => {
            //let response_body = format!("New order: {:?}", order);
            let result = handle_order(order, start_time, &mut ordered_products).expect("Handling order failed");
            let end_time: DateTime<Utc> = Utc::now();
            Ok(Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "http://localhost:5173")
                .header("Access-Control-Allow-Methods", "POST")
                .header("Access-Control-Allow-Headers", "Content-Type")
                //.body(response_body)
                .body(format!("Execution time: {}. Successfully ordered: {:?}", end_time - start_time, ordered_products))
                .build())
        }
        Err(e) => {
            eprintln!("Failed to parse request body: {:?}", e);
            Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "http://localhost:5173")
                .header("Access-Control-Allow-Methods", "POST")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(format!("Invalid request body: {}", e))
                .build())
        }
    }
}

fn handle_order(
    order: Order,
    start_time: DateTime<Utc>,
    ordered_products: &mut HashMap<String, i32>
) -> Result<i32> {
    let address = std::env::var(DB_URL_ENV)?;
    let conn = pg::Connection::open(&address)?;
    let mut current_quantity = 0;

    for article in &order.checkout {
        current_quantity =
            get_product_quantity(&conn, article).expect("Product quantity query failed") - article.quantity;
        if current_quantity <= 0 {
            println!("Insufficient stock for product: {:?}", article.name)
        } else {
            let sql = format!("UPDATE products.\"products-details\" \
            SET quantity = '{}' WHERE name = '{}'", current_quantity, article.name);
            let sql_execute = conn.query(&sql, &[])?;
            println!("Product quantity updated: {:?}", sql_execute);
            ordered_products.insert(article.name.clone(), article.quantity);
        }
    }
    insert_customer_data(&conn, order, start_time).expect("Customer data insert failed");
    Ok(0)
}

fn get_product_quantity(conn: &pg::Connection, article: &Cart) -> Result<i32> {
    println!("ARTICLE NAME: {}", article.name);
    // Welcome to SQL-Injection its cool, its fun for everybody!!!
    let sql =
        format!("SELECT quantity FROM products.\"products-details\" \
        WHERE name = '{}' AND article_number = '{}'", article.name, article.id);
    // ParameterValue is not working - fix later (maybe)
    //let params = vec![ParameterValue::Str(article.name)];
    let rowset = conn.query(&sql, &[])?;

    match rowset.rows.first() {
        None => Ok(0),
        Some(row) => match row.first() {
            None => Ok(0),
            Some(pg::DbValue::Int32(i)) => Ok(*i),
            Some(other) => Err(anyhow!("Unexpected error: {:?}", other)),
        },
    }
}

fn insert_customer_data(
    conn: &pg::Connection,
    order: Order,
    start_time: DateTime<Utc>
) -> Result<i32> {
    let sql = format!("INSERT INTO products.\"customers\" \
    (name, email, phone, address, city, pin, last_ordered) \
    VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}') \
    ON CONFLICT (name, email) DO UPDATE SET last_ordered = EXCLUDED.last_ordered",
                      order.customer.name,
                      order.customer.email,
                      order.customer.phone,
                      order.customer.address,
                      order.customer.city,
                      order.customer.pin,
                      start_time
    );
    let sql_execute = conn.execute(&sql, &[])?;
    println!("Customer data added: {:?}", sql_execute);
    Ok(0)
}

// Muss ich noch machen
// fn insert_order_data(conn: &pg::Connection, order: Order, start_time: DateTime<Utc>) {
//     let sql = format!("INSERT INTO products.\"orders\" (name, email, phone, address, city, pin, last_ordered) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}') \
//     ON DUPLICATE KEY UPDATE last_ordered = VALUES(last_ordered)",
//                       order.customer.name,
//                       order.customer.email,
//                       order.customer.phone,
//                       order.customer.address,
//                       order.customer.city,
//                       order.customer.pin,
//                       start_time
//     );
//     let sql_execute = conn.execute(&sql, &[])?;
//     println!("CUSTOMER DATA ADDED: {:?}", sql_execute);
//}
