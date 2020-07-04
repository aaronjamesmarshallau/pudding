#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

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

    rocket::ignite()
        .manage(bucket_metadata)
        .mount(
            "/",
            routes![
                //handlers::files::get_file,
                handlers::files::create_file,
            ],
        )
        .launch();
}
