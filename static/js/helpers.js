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

window.addEventListener("pageshow", () => {
  // just to make sure the loader is hidden when the page is shown
  hideLoader();
});

function showLoader() {
  let loader = document.getElementById("loader-container");
  if (loader) {
    loader.style.display = "flex";
  }
}

function hideLoader() {
  let loader = document.getElementById("loader-container");
  if (loader) {
    loader.style.display = "none";
  }
}

function navigateWithLoadingDots(url) {
  // Show the loader container
  let loader = document.getElementById("loader-container");
  if (loader) {
    loader.style.display = "flex";
  }

  // Redirect after a short delay to allow the loader to show
  setTimeout(() => {
    window.location.href = url;
  }, 100); // Short delay
}

function showloaderContainer() {
  // Show the loader container
  let loader = document.getElementById("loader-container");
  if (loader) {
    loader.style.display = "flex";
  }
}

// hide loader
function hideloaderContainer() {
  // Hide the loader container
  let loader = document.getElementById("loader-container");
  if (loader) {
    loader.style.display = "none";
  }
}

// wait for a specified time
function wait(time) {
  return new Promise((resolve) => {
    setTimeout(resolve, time);
  });
}

// db functions
async function checkDatabaseStatus() {
  try {
    const response = await fetch("/db-check");
    const text = await response.text();
    console.log("Database status:", text);
    return text.includes("1");
  } catch (e) {
    console.error(e);
    return false; // assume the database is down
  }
}

function updateDbStatusIndicator(isOnline) {
  const statusElement = document.getElementById("db-status-text");
  if (isOnline) {
    statusElement.textContent = "Online";
    statusElement.style.color = "green";
  } else {
    statusElement.textContent = "Offline";
    statusElement.style.color = "red";
  }
}

// offline checkin
async function performCheckIn(userData) {
  const dbIsOnline = await checkDatabaseStatus();

  if (dbIsOnline) {
    // Proceed with normal check-in process
    sendCheckInToServer(userData);
  } else {
    // Store the check-in data locally for later synchronization
    storeCheckInLocally(userData);
  }
}

// local checkin
function storeCheckInLocally(checkInData) {
  // Example using local storage; for more complex data, use IndexedDB
  const existingData =
    JSON.parse(localStorage.getItem("offlineCheckIns")) || [];
  existingData.push(checkInData);
  localStorage.setItem("offlineCheckIns", JSON.stringify(existingData));
}

// sync when back online
window.addEventListener("online", async () => {
  const dbIsOnline = await checkDatabaseStatus();

  if (dbIsOnline) {
    syncLocalDataWithServer();
  }
});

async function syncLocalDataWithServer() {
  const offlineData = JSON.parse(localStorage.getItem("offlineCheckIns")) || [];
  for (const checkInData of offlineData) {
    await sendCheckInToServer(checkInData);
  }
  localStorage.removeItem("offlineCheckIns"); // Clear the local storage after syncing
}

// send checkin to server
async function sendCheckInToServer(checkInData) {
  try {
    const response = await fetch("/bulk-checkin", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(checkInData),
    });
    const result = await response.json();
    console.log("Check-in result:", result);
  } catch (error) {
    console.error("Error sending check-in to server:", error);
  }
}
