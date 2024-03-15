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

function startInactivityTimer(TIMEOUT_DURATION = 300000) {
  // Default to 5 minutes
  let timeout;

  function redirectToHome() {
    window.location.href = "/";
  }

  function resetTimeout() {
    clearTimeout(timeout);
    timeout = setTimeout(redirectToHome, TIMEOUT_DURATION);
  }

  document.addEventListener("mousemove", resetTimeout);
  document.addEventListener("keypress", resetTimeout);
  document.addEventListener("click", resetTimeout);

  resetTimeout();
}

let timeout;
let inputBox;

// Set a timeout to clear the input box if not entered fully within a specified time
function setInputClearTimer(
  inputSelector,
  clearAfter = 5000,
  fieldFocus = true
) {
  // Find the input box
  const inputBox = document.querySelector(inputSelector);
  console.log("inputbox is:" + inputBox);

  if (!inputBox) return; // Exit if the input box is not found

  // Function to clear the input box
  function clearInput() {
    inputBox.value = "";
    // Focus on the input box
    if (fieldFocus) {
      setTimeout(() => inputBox.focus(), 0);
    }
  }

  // Reset the timer whenever the user types
  function resetTimer() {
    clearTimeout(timeout);
    timeout = setTimeout(clearInput, clearAfter);
  }

  // Listen for keypresses in the input box
  inputBox.addEventListener("keyup", resetTimer);

  // Listen for the blur event on the input box
  inputBox.addEventListener("blur", () => {
    if (fieldFocus) {
      setTimeout(() => inputBox.focus(), clearAfter);
    }
  });

  // Initialize the timer
  resetTimer();
}

function cancelInputClearTimer() {
  // Clear the timeout
  clearTimeout(timeout);
  console.log("cancelInputClearTimer called");

  // Remove the event listeners
  if (inputBox) {
    inputBox.removeEventListener("keyup", resetTimer);
    inputBox.removeEventListener("blur", () => {
      if (fieldFocus) {
        setTimeout(() => inputBox.focus(), clearAfter);
      }
    });
  }
}
