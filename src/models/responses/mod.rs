use crate::models::streams::ContentLength;
use rocket_contrib::json::Json;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder};
use rocket::Request;
use serde::Serialize;
use std::io::Read;

/// A response for a file stream
pub struct FileResponse<T: Read + 'static> {
    file_data: Option<T>,
    status: Status,
    content_type: ContentType,
    content_length: ContentLength,
}

impl<T: Read + 'static> FileResponse<T> {
	/// Creates a new file response from the provided optional T, status,
	/// ContentType, and ContentLength
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

	/// Creates a new FileResponse with an Ok Status from the provided T,
	/// ContentType, and ContentLength
    pub fn ok(cur: T, content_type: ContentType, content_length: ContentLength) -> Self {
        FileResponse::new(Some(cur), Status::Ok, content_type, content_length)
    }

	/// Creates a new FileResponse with a NotFound Status.
    pub fn not_found() -> Self {
        FileResponse::new(None, Status::BadRequest, ContentType::default(), ContentLength(0))
    }
}

impl<'r, T: Read> Responder<'r> for FileResponse<T> {
    fn respond_to(self, _: &Request) -> rocket::response::Result<'r> {
        match self.file_data {
            Some(file_data) => {
                let reader = file_data;

                rocket::response::Response::build()
                    .streamed_body(reader)
                    .status(self.status)
                    .header(self.content_type)
                    .header(self.content_length)
                    .ok()
            },
            None => {
                rocket::response::Response::build()
                    .status(self.status)
                    .header(self.content_type)
                    .header(self.content_length)
                    .ok()
            }
        }


    }
}

/// A general API response that wraps a JSON response and status code.
pub struct ApiResponse<T: Serialize> {
    pub json: Option<T>,
    pub status: Status,
}

impl<'r, T: Serialize> Responder<'r> for ApiResponse<T> {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        let json_response_body = Json(self.json);
        rocket::response::Response::build_from(json_response_body.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

/// A data structure that represents the result of uploading a file.
#[derive(Serialize)]
pub struct UploadResult {
    pub file_id: String,
}
