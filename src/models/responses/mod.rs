use serde::Serialize;
use rocket_contrib::json::Json;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Stream};
use rocket::Request;
use std::io::Cursor;

/// A response for a file stream
pub struct FileResponse {
    file_data: Stream<Cursor<Vec<u8>>>,
    status: Status,
}

fn empty_cursor() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::new())
}

impl FileResponse {
    pub fn not_found() -> FileResponse {
        FileResponse::new(empty_cursor(), Status::NotFound)
    }

    pub fn ok(cur: Cursor<Vec<u8>>) -> FileResponse {
        FileResponse::new(cur, Status::Ok)
    }

    pub fn new(cur: Cursor<Vec<u8>>, status: Status) -> FileResponse {
        FileResponse {
            file_data: cur.into(),
            status: status,
        }
    }
}

impl<'r> Responder<'r> for FileResponse {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        rocket::response::Response::build_from(self.file_data.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

pub struct ApiResponse<T> {
    pub json: Json<Option<T>>,
    pub status: Status,
}

impl<'r, T: Serialize> Responder<'r> for ApiResponse<T> {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        rocket::response::Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
