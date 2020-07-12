use crate::handlers::files::ContentLength;
use std::io::Read;
use serde::Serialize;
use rocket_contrib::json::Json;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder};
use rocket::Request;

/// A response for a file stream
pub struct FileResponse<T: Read + 'static> {
    file_data: Option<T>,
    status: Status,
    content_type: ContentType,
    content_length: ContentLength,
}

impl<T: Read + 'static> FileResponse<T> {
    pub fn new(
        read: Option<T>,
        status: Status,
        content_type: ContentType,
        content_length: ContentLength
    ) -> Self {
        FileResponse {
            file_data: read,
            status: status,
            content_length,
            content_type,
        }
    }

    pub fn ok(cur: T, content_type: ContentType, content_length: ContentLength) -> Self {
        FileResponse::new(Some(cur), Status::Ok, content_type, content_length)
    }

    pub fn not_found() -> Self {
        FileResponse::new(None, Status::BadRequest, ContentType::default(), ContentLength(0))
    }
}

impl<'r, T: Read> Responder<'r> for FileResponse<T> {
    fn respond_to(self, _: &Request) -> rocket::response::Result<'r> {
        rocket::response::Response::build()
            .streamed_body(self.file_data.unwrap())
            .status(self.status)
            .header(self.content_type)
            .header(self.content_length)
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
