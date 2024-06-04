use spin_sdk::http::{IntoResponse, Json, Response, Request};
use spin_sdk::http::conversions::TryFromIncomingRequest;
use spin_sdk::{http_component, pg::{self, Decode}};
use anyhow::Result;
use serde::Deserialize;
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

impl TryFrom<&pg::Row> for Customer {
    type Error = anyhow::Error;

    fn try_from(row: &pg::Row) -> Result<Self, Self::Error> {
        let name = String::decode(&row[0])?;
        let email = String::decode(&row[1])?;
        let phone = String::decode(&row[2])?;
        let address = String::decode(&row[3])?;
        let city = String::decode(&row[4])?;
        let pin = String::decode(&row[5])?;

        Ok(Self {
            name,
            email,
            phone,
            address,
            city,
            pin,
        })
    }
}

#[derive(serde::Deserialize, Debug)]
struct Cart {
    id: i32,
    name: String,
    image: String,
    price: i32,
    quantity: i32,
}

impl TryFrom<&pg::Row> for Cart {
    type Error = anyhow::Error;

    fn try_from(row: &pg::Row) -> Result<Self, Self::Error> {
        let id = i32::decode(&row[0])?;
        let name = String::decode(&row[1])?;
        let image = String::decode(&row[2])?;
        let price = i32::decode(&row[3])?;
        let quantity = i32::decode(&row[4])?;

        Ok(Self {
            id,
            name,
            image,
            price,
            quantity,
        })
    }
}

#[derive(serde::Deserialize, Debug)]
struct Order {
    customer: Customer,
    checkout: Vec<Cart>,
    orderTotal: String,
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
                .body(response_body)
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

// fn read_from_db<T>(_req: Request<()>, dataset: T) -> Result<Response<String>> {
//     let address = std::env::var(DB_URL_ENV)?;
//     let conn = pg::Connection::open(&address)?;
//
//     let sql = "SELECT id, title, content, authorname, coauthor FROM articletest";
//     let rowset = conn.query(sql, &[])?;
//
//     let column_summary = rowset
//         .columns
//         .iter()
//         .map(format_col)
//         .collect::<Vec<_>>()
//         .join(", ");
//
//     let mut response_lines = vec![];
//
//     for row in rowset.rows {
//         let article = Article::try_from(&row)?;
//
//         println!("article: {:#?}", article);
//         response_lines.push(format!("article: {:#?}", article));
//     }
//
//     // use it in business logic
//
//     let response = format!(
//         "Found {} article(s) as follows:\n{}\n\n(Column info: {})\n",
//         response_lines.len(),
//         response_lines.join("\n"),
//         column_summary,
//     );
//
//     Ok(http::Response::builder().status(200).body(response)?)
// }
//
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