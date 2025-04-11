use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
};
use include_dir::{Dir, include_dir};
use mime_guess::from_path;

// Include the static directory in the binary
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

// Serve static files from the embedded directory
pub async fn serve_static_file(Path(path): Path<String>) -> impl IntoResponse {
    // Try to find the file in the embedded directory
    if let Some(file) = STATIC_DIR.get_file(&path) {
        // Get the file contents
        let contents = file.contents().to_vec();

        // Guess the MIME type
        let mime_type = from_path(&path).first_or_octet_stream().to_string();

        // Create the response with headers
        (
            [
                (header::CONTENT_TYPE, mime_type),
                (
                    header::CACHE_CONTROL,
                    "public, max-age=31536000".to_string(),
                ),
            ],
            contents,
        )
            .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
