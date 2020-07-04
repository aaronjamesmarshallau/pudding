use crate::models::state::BucketMetadata;
use crate::models::responses::FileResponse;

use std::io::Cursor;

use rocket::State;
use rocket::Data;
use rocket::http::ContentType;

use s3::bucket::Bucket;

#[get("/api/files/<file_id>")]
pub fn get_file(file_id: i32, bucket_metadata: State<BucketMetadata>) -> FileResponse {
    let metadata = bucket_metadata.inner();
    let bucket_name = metadata.bucket_name;
    let region = metadata.region.clone();
    let credentials = metadata.credentials.clone();

    let mut cursor = Cursor::new(Vec::new());
    let bucket = Bucket::new(bucket_name, region, credentials).unwrap();
    let result = bucket.get_object_stream_blocking(&format!("/{}", file_id), &mut cursor);

    match result {
        Ok(response_code) => {
            println!("Request finished successfully with response code {}", response_code);
            FileResponse::ok(cursor.into())
        },
        Err(error) => {
            println!("An error occurred: {}\n\treturning NotFound", error);
            FileResponse::not_found()
        }
    }
}

#[put("/api/files/<file_id>", data = "<file_data>")]
pub fn update_file(file_id: String, file_data: Data, content_type: &ContentType) {

}

#[post("/api/files", data = "<file_data>")]
pub fn create_file(file_data: Data, content_type: &ContentType) {

}
