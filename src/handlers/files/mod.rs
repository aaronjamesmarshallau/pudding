use crate::models::state::BucketMetadata;

use std::io::Cursor;

use rocket::State;
use rocket::Data;
use rocket::http::ContentType;
use rocket::response::Stream;
use rocket::response::status::NotFound;

use s3::bucket::Bucket;

const CHUNK_SIZE: u64 = 1024 * 1024; // Stream files in 1MiB chunks?

#[get("/api/files/<file_id>")]
pub fn get_file(file_id: i32, bucket_metadata: State<BucketMetadata>) -> Result<Stream<Cursor<Vec<u8>>>, NotFound<String>> {
    let metadata = bucket_metadata.inner();
    let bucket_name = metadata.bucket_name;
    let region = metadata.region.clone();
    let credentials = metadata.credentials.clone();

    let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    let mut cursor = Cursor::new(Vec::new());
    let result = bucket.get_object_stream_blocking(&format!("/{}", file_id), &mut cursor);

    match result {
        Ok(response_code) => {
            Ok(cursor.into())
        },
        Err(error) => {
            println!("An error occurred: {}\n\treturning NotFound", error);
            Err(NotFound("The requested resource could not be retrieved.".into()))
        }
    }
}

#[put("/api/files/<file_id>", data = "<file_data>")]
pub fn update_file(file_id: String, file_data: Data, content_type: &ContentType) {

}

#[post("/api/files", data = "<file_data>")]
pub fn create_file(file_data: Data, content_type: &ContentType) {

}
