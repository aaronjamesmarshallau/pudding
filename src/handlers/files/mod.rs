use rocket::Data;
use rocket::http::ContentType;
use rocket::response::Stream;
use rocket::response::status::NotFound;

const CHUNK_SIZE: u64 = 1024 * 1024; // Stream files in 1MiB chunks?

#[get("/api/files/<file_id>")]
pub fn get_file(file_id: i32) -> Result<Stream<Cursor<Vec<u8>>>, NotFound<String>> {
    // gimme file reader

    Ok(Stream::chunked(reader, CHUNK_SIZE))
}

#[put("/api/files/<file_id>", data = "<file_data>")]
pub fn update_file(file_id: String, file_data: Data, content_type: &ContentType) {

}

#[post("/api/files", data = "<file_data>")]
pub fn create_file(file_data: Data, content_type: &ContentType) {

}
