use crate::models::responses::{ApiResponse, FileResponse, UploadResult};
use crate::models::state::BucketMetadata;
use crate::models::streams::{Chunkable, ContentLength};
use core::iter::FromIterator;
use futures::stream::{self};
use rocket::{Data, State};
use rocket::http::{ContentType, Status};
use rusoto_core::{ByteStream, HttpClient};
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3Client, S3};
use std::io::{Read};
use tokio::runtime::Runtime;
use uuid::Uuid;

fn bytestream_to_read(stream: ByteStream) -> impl Read + Send {
    stream.into_blocking_read()
}

fn read_to_bytestream<R: Read + Send + Sync + 'static>(read: R) -> ByteStream {
    let bytes = read.bytes(); // Iterator<Item = Result<u8>>
    let chunks = bytes.chunks(4096); // Iterator<Item = Vec<Result<u8>>>
    let iter_chunks = chunks.map(|b| Result::from_iter(b.into_iter())); // Iterator<Item = Result<Vec<u8>>>
    let stream = stream::iter(iter_chunks); // Stream<Item = Result<Vec<u8>>>

    ByteStream::new(stream)
}

#[get("/api/files/<file_id>")]
pub fn get_file(file_id: String, bucket_metadata: State<BucketMetadata>, runtime: State<Runtime>) -> FileResponse<impl Read> {
    let metadata = bucket_metadata.inner();
    let bucket_name = &metadata.bucket_name;
    let region = metadata.region.clone();
    let credentials = metadata.credentials.clone();

    let client = S3Client::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        credentials,
        region,
    );

    let request = GetObjectRequest {
        bucket: bucket_name.to_owned(),
        key: file_id.clone(),
        ..Default::default()
    };

    let future = client.get_object(request);
    let result = runtime.handle().block_on(future);

    match result {
        Ok(response_object) => {
            println!("Request for file {} finished successfully: {:?}", file_id, response_object);
            let bytestream = response_object.body;
            let read = bytestream_to_read(bytestream.unwrap());

            FileResponse::ok(
                read,
                response_object.content_type
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(ContentType::Binary),
                ContentLength(response_object.content_length.unwrap_or(0))
            )
        },
        Err(error) => {
            println!("An error occurred: {}\n\treturning NotFound", error);
            FileResponse::not_found()
        }
    }
}

#[post("/api/files", format = "binary", data = "<file_data>")]
pub fn create_file(
    file_data: Data,
    bucket_metadata: State<BucketMetadata>,
    length: ContentLength,
    runtime: State<Runtime>,
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

	let ContentLength(length) = length;

	println!("ContentLength: {}", length);

    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: file_id.to_string(),
        body: Some(stream),
        content_length: Some(length),
        ..Default::default()
    };

	let future = client.put_object(req);
    let result = runtime.handle().block_on(future);

    match result {
        Ok(put_output) => {
            println!("Successfully uploaded file {} with output {:?}", file_id, put_output);
            ApiResponse {
                json: Some(UploadResult { file_id: file_id.to_string() }),
                status: Status::Ok,
            }
        },
        Err(error) => {
            println!("Unable to create file: {}", error);
            ApiResponse {
                json: None,
                status: Status::BadRequest,
            }
        },
    }
}
