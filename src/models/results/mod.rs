use serde::Serialize;

#[derive(Serialize)]
pub struct UploadResult {
    pub file_id: String,
}
