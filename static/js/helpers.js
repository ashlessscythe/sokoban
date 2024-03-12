// helper function to extract userId from scannedId
function extractUserId(scannedId) {
  const patterns = [
    { prefix: "100", length: 9 },
    { prefix: "21", length: 8 },
    { prefix: "20", length: 8 },
    { prefix: "104", length: 9 },
    { prefix: "600", length: 9 },
  ];

  let bestMatch = { match: "", startIndex: Infinity, length: 0 };

  for (const pattern of patterns) {
    const startIndex = scannedId.indexOf(pattern.prefix);
    if (startIndex !== -1 && startIndex < bestMatch.startIndex) {
      const possibleMatch = scannedId.substr(startIndex, pattern.length);
      // Check if the extracted match is closer to the start and longer than the current best match
      if (
        startIndex < bestMatch.startIndex ||
        (startIndex === bestMatch.startIndex &&
          pattern.length > bestMatch.length)
      ) {
        bestMatch = {
          match: possibleMatch,
          startIndex: startIndex,
          length: pattern.length,
        };
      }
    }
  }

  if (bestMatch.match) {
    console.log(`Extracted ID: ${bestMatch.match}`);
    return bestMatch.match;
  } else {
    console.log(
      `Nothing was extracted from scannedId: ${scannedId}. Returning the original scannedId.`
    );
    return scannedId;
  }
}
