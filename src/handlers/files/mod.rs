use crate::models::results::UploadResult;
use crate::models::responses::ApiResponse;
use crate::models::state::BucketMetadata;
use crate::models::responses::FileResponse;
use uuid::Uuid;
use rocket_contrib::json::Json;
use futures::executor::block_on;
use futures::stream;
use itertools::Itertools;
use core::iter::FromIterator;

use std::io::{Cursor, Read};

use rocket::State;
use rocket::Data;
use rocket::http::Status;

use rusoto_core::{HttpClient, Region};
use rusoto_s3::{PutObjectRequest, S3Client, StreamingBody, S3};

// #[get("/api/files/<file_id>")]
// pub fn get_file(file_id: String, bucket_metadata: State<BucketMetadata>) -> FileResponse {
//     let metadata = bucket_metadata.inner();
//     let bucket_name = &metadata.bucket_name;
//     let region = metadata.region.clone();
//     let credentials = metadata.credentials.clone();

//     let mut cursor = Cursor::new(Vec::new());
//     let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
//     let result = bucket.get_object_stream_blocking(&format!("/{}", file_id), &mut cursor);

//     match result {
//         Ok(response_code) => {
//             println!("Request finished successfully with response code {}", response_code);
//             FileResponse::ok(cursor.into())
//         },
//         Err(error) => {
//             println!("An error occurred: {}\n\treturning NotFound", error);
//             FileResponse::not_found()
//         }
//     }
// }

#[post("/api/files", data = "<file_data>")]
pub fn create_file(
    file_data: Data,
    bucket_metadata: State<BucketMetadata>
) -> ApiResponse<UploadResult> {
    let file_id = Uuid::new_v4().to_simple();
    let metadata = bucket_metadata.inner();
    let bucket_name = &metadata.bucket_name;
    let region = metadata.region.clone();
    let credentials = metadata.credentials.clone();

    println!("Data length: {}", file_data.peek().iter().count());

    let mut stream = file_data.open();

    let client = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        credentials,
        Region::UsEast1,
    );

    let bitty_boys = stream::iter(
        stream.bytes()
            .map(Result::unwrap)
            .chunks(4096)
            .into_iter()
            .map(|chunk| Ok(bytes::Bytes::from_iter(chunk)))
    );

    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        body: Some(StreamingBody::new(bitty_boys)),
        ..Default::default()
    };

    let result = block_on(client.put_object(req));

    match result {
        Ok(put_output) => {
            println!("Successfully uploaded file {} with output {:?}", file_id, put_output);
            ApiResponse {
                json: Json(Some(UploadResult { file_id: file_id.to_string() })),
                status: Status::Ok,
            }
        },
        Err(error) => {
            println!("Unable to create file: {}", error);
            ApiResponse {
                json: Json(None),
                status: Status::BadRequest,
            }
        },
    }
}
