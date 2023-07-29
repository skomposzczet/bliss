use serde::Deserialize;

#[derive(Deserialize)]
pub struct UploadImageResponse {
    pub msg: String,
    pub files: Vec<ImageFile>,
}

#[derive(Deserialize)]
pub struct ImageFile {
    pub file: String,
    #[allow(dead_code)]
    delete_token: String,
}
