#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate ctrlc;

use tokio::runtime::Runtime;
use crate::models::state::BucketMetadata;
use rusoto_credential::StaticProvider;

pub mod handlers;
pub mod models;

fn initialize_bucket_metadata() -> BucketMetadata {
    let aws_region = dotenv::var("AWS_REGION").expect("AWS_REGION must be set");
    let aws_access_key = dotenv::var("AWS_ACCESS_KEY").expect("AWS_ACCESS_KEY must be set");
    let aws_secret_key = dotenv::var("AWS_SECRET_KEY").expect("AWS_SECRET_KEY must be set");
    let bucket_name = dotenv::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");

    let credentials = StaticProvider::new_minimal(aws_access_key, aws_secret_key);
    let region = aws_region.parse().expect("AWS_REGION must be a valid AWS region");

    BucketMetadata {
        bucket_name,
        credentials: credentials,
        region,
    }
}

fn main() {
    dotenv::dotenv().ok();

    let bucket_metadata = initialize_bucket_metadata();
    let runtime = Runtime::new().expect("Failed to create tokio runtime");

    let rocket_ship = rocket::ignite()
        .manage(bucket_metadata)
        .manage(runtime)
        .mount(
            "/",
            routes![
                handlers::files::get_file,
                handlers::files::create_file,
            ],
        );

    let result = ctrlc::set_handler(move || {
        std::process::exit(0);
    });

    match result {
        Ok(_) => { },
        Err(err) => {
            println!("{}", err);
        }
    }

    rocket_ship.launch();
}
