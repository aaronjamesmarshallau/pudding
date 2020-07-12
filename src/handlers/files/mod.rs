use rocket::request::Outcome;
use std::error::Error;
use rocket::Request;
use rocket::request::FromRequest;
use crate::models::results::UploadResult;
use crate::models::responses::ApiResponse;
use crate::models::state::BucketMetadata;
use crate::models::responses::FileResponse;
use core::iter::FromIterator;
use futures::stream;
use rocket_contrib::json::Json;
use rusoto_core::ByteStream;
use tokio::runtime::Runtime;
use uuid::Uuid;

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

pub trait Chunks: Sized {
    fn chunks(self, size: usize) -> Chunk<Self> {
        Chunk { iter: self, size }
    }
}

impl<T: Iterator> Chunks for T {}

pub struct Chunk<I> {
    iter: I,
    size: usize,
}

impl<I> Iterator for Chunk<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::new();

        for _ in 0..self.size {
            if let Some(v) = self.iter.next() {
                chunk.push(v);
            } else {
                break;
            }
        }

        if chunk.len() > 0 {
            Some(chunk)
        } else {
            None
        }
    }
}

fn read_to_bytestream<R: Read + Send + Sync + 'static>(read: R) -> ByteStream {
    let bytes = read.bytes(); // Iterator<Item = Result<u8>>
    let chunks = bytes.chunks(4096); // Iterator<Item = Vec<Result<u8>>>
    let iter_chunks = chunks.map(|b| Result::from_iter(b.into_iter())); // Iterator<Item = Result<Vec<u8>>>
    let stream = stream::iter(iter_chunks); // Stream<Item = Result<Vec<u8>>>

    ByteStream::new(stream)
}

pub struct ContentLength(i64);

#[derive(Debug)]
pub enum ContentLengthError {
    BadCount,
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for ContentLength {
    type Error = ContentLengthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Content-Length").collect();

        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ContentLengthError::Missing)),
            1 if keys[0].parse().unwrap_or(-1) >= 0 => Outcome::Success(ContentLength(keys[0].parse().unwrap_or(0))),
            1 => Outcome::Failure((Status::BadRequest, ContentLengthError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ContentLengthError::BadCount)),
        }
    }
}

#[post("/api/files", format = "binary", data = "<file_data>")]
pub fn create_file(
    file_data: Data,
    bucket_metadata: State<BucketMetadata>,
    length: ContentLength,
) -> ApiResponse<UploadResult> {
    let file_id = Uuid::new_v4().to_simple();
    let metadata = bucket_metadata.inner();
    let bucket_name = &metadata.bucket_name;
    let region = metadata.region.clone();
    let credentials = metadata.credentials.clone();

    let read = file_data.open();
    let stream = read_to_bytestream(read);

    let client = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        credentials,
        region,
    );

    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: file_id.to_string(),
        body: Some(stream),
        content_length: Some(length.0),
        ..Default::default()
    };

    let future = client.put_object(req);
    let mut runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(error) => {
            println!("Failed to create tokio runtime: {}", error);
            return ApiResponse {
                json: Json(None),
                status: Status::InternalServerError,
            };
        }
    };

    let result = runtime.block_on(future);

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
