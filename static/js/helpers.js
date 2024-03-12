// helper function to extract userId from scannedId
function extractUserId(scannedId) {
  const patterns = [
    { prefix: "100", length: 9 },
    { prefix: "21", length: 8 },
    { prefix: "20", length: 8 },
    { prefix: "104", length: 9 },
    { prefix: "600", length: 9 },
  ];

  for (const pattern of patterns) {
    const startIndex = scannedId.indexOf(pattern.prefix);
    if (startIndex !== -1) {
      extracted_id = scannedId.substr(startIndex, pattern.length);
      console.log("Extracted userId:", extracted_id);
      return extracted_id;
    }
  }
  // Return the original scannedId if no pattern matches
  console.log(
    "Nothing was extracted from scannedId:",
    scannedId,
    ". Returning the original scannedId."
  );
  return scannedId;
}
