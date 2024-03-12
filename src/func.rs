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
