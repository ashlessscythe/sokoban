use chrono::{Date, Utc};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use sha2::{Digest, Sha256};

pub struct ExtractedUserId(String);

pub fn get_drill_id(drill_id: Option<i32>) -> i32 {
    match drill_id {
        Some(id) => id,
        None => {
            let today = Utc::now().format("%Y%m%d").to_string();
            today.parse::<i32>().unwrap_or_default()
        }
    }
}

// generate_temp_id generates a temporary ID for the user.
pub fn generate_temp_id(user_id: &str) -> String {
    let drill_id = get_drill_id(None);
    let mut hasher = Sha256::new();
    hasher.update(user_id);
    hasher.update(&drill_id.to_ne_bytes());
    format!("{:x}", hasher.finalize())
}

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
        let patterns = vec![("100", 9), ("21", 8), ("20", 8), ("104", 9), ("600", 9)];

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
