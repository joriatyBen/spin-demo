use spin_sdk::http::{Response, Request};
use spin_sdk::{http_component, pg::{self, Decode, ParameterValue}};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::error::Error;


// The environment variable set in `spin.toml` that points to the
// address of the Pg server that the component will write to
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
async fn hello_world(req: Request) -> Result<Response, Box<dyn Error>> {
    match serde_json::from_slice::<Order>(req.body().as_ref()) {
        Ok(order) => {
            let response_body = format!("New order: {:?}", order);
            Ok(Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "http://localhost:5173")
                .header("Access-Control-Allow-Methods", "POST")
                .header("Access-Control-Allow-Headers", "Content-Type")
                //.body(response_body)
                .body(format!("{:?}", check_article_quantity(order).unwrap()))
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

fn get_product_quantity(conn: &pg::Connection, article: Cart) -> Result<i32> {
    println!("ARTICLE NAME: {}", article.name);
    let sql =
        format!("SELECT quantity FROM products.\"products-details\" WHERE name = '{}' AND article_number = '{}'", article.name, article.id); // Welcome SQL-Injection!!!
    //let params = vec![ParameterValue::Str(article.name)]; // ParameterValue is not working - fix later

    let rowset = conn.query(&sql, &[])?;

    match rowset.rows.first() {
        None => Ok(0),
        Some(row) => match row.first() {
            None => Ok(0),
            Some(pg::DbValue::Int32(i)) => Ok(*i),
            Some(other) => Err(anyhow!(
                "Unexpected"
            )),
        },
    }
}

fn check_article_quantity(order: Order) -> Result<Vec<i32>> {
    let address = std::env::var(DB_URL_ENV)?;
    let conn = pg::Connection::open(&address)?;
    let mut ids = Vec::new();

    for article in order.checkout {
        ids.push(get_product_quantity(&conn, article).expect("DID NOT EXPECT THAT"));
    }
    Ok(ids)
}

// fn write_to_db(_req: Request<()>) -> Result<Response<String>> {
//     let address = std::env::var(DB_URL_ENV)?;
//     let conn = pg::Connection::open(&address)?;
//
//     let sql = "INSERT INTO articletest (title, content, authorname) VALUES (?, ?, ?, ?)";
//     let nrow_executed = conn.execute(sql, &[])?;
//
//     println!("nrow_executed: {}", nrow_executed);
//
//     let sql = "SELECT COUNT(id) FROM articletest";
//     let rowset = conn.query(sql, &[])?;
//     let row = &rowset.rows[0];
//     let count = i64::decode(&row[0])?;
//     let response = format!("Count: {}\n", count);
//
//     Ok(http::Response::builder().status(200).body(response)?)
// }