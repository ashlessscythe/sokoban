use rocket::request::{self, FromRequest, Request};
use rocket::outcome::Outcome;

pub struct ExtractedUserId(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ExtractedUserId {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let id = request.param::<String>(0).unwrap();
        let extractor = UserIdExtractor::new(id.expect("No scannedId found in the request"));
        let extracted_id = extractor.extract_user_id();
        Outcome::Success(ExtractedUserId(extracted_id))
    }
}

pub struct UserIdExtractor {
    scanned_id: String,
}

impl UserIdExtractor {
    pub fn new(scanned_id: String) -> Self {
        Self { scanned_id }
    }

    pub fn extract_user_id(&self) -> String {
        let patterns = vec![
            ("100", 9),
            ("21", 8),
            ("20", 8),
            ("104", 9),
            ("600", 9),
        ];

        for (prefix, length) in patterns {
            if let Some(start_index) = self.scanned_id.find(prefix) {
                return self.scanned_id[start_index..start_index + length].to_string();
            }
        }

        // Log or handle the case where no pattern matches
        println!(
            "Nothing was extracted from scannedId: {}. Returning the original scannedId.",
            self.scanned_id
        );
        self.scanned_id.to_string()
    }

}
