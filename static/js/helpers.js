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

// This will keep track of the last known DB status
let dbWasOffline = false;

// db functions
async function checkDatabaseStatus() {
  try {
    const response = await fetch("/db-check");
    const isOnline = response.ok && (await response.text()).includes("1");

    console.log("Checking Database status:", isOnline ? "Online" : "Offline");

    // If the database was offline last time we checked and is now online, trigger a sync
    if (dbWasOffline && isOnline) {
      await syncLocalDataWithServer(); // Ensure this function exists and correctly handles the sync
      dbWasOffline = false; // Reset the flag after syncing
    }

    // If the database is offline, set the flag
    if (!isOnline) {
      dbWasOffline = true;
    }

    return isOnline;
  } catch (e) {
    console.error(e);
    dbWasOffline = true; // assume the database is down and set the flag
    return false;
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

// local checkin
function storeCheckInLocally(checkInData) {
  // Example using local storage; for more complex data, use IndexedDB
  const existingData =
    JSON.parse(localStorage.getItem("offlineCheckIns")) || [];
  existingData.push(checkInData);
  localStorage.setItem("offlineCheckIns", JSON.stringify(existingData));
}

// sync when back online (looper)
async function syncLocalDataWithServer() {
  const offlineData = JSON.parse(localStorage.getItem("offlineCheckIns")) || [];
  for (const checkInData of offlineData) {
    console.log("syncing check-in data:", checkInData);
    try {
      let response = await sendCheckInToServer(checkInData);
      console.log("response from sendCheckIn is:", response);

      // if response not undefined, remove from local storage
      if (response && response.status !== 404) {
        console.log("removing item from local storage:", checkInData);
        // Filter out the sent item instead of splicing by index
        const updatedOfflineData = offlineData.filter(
          (item) => item !== checkInData
        );
        localStorage.setItem(
          "offlineCheckIns",
          JSON.stringify(updatedOfflineData)
        );
      } else {
        console.log(
          `Failed to sync data: ${response.status} ${response.statusText}`
        );
      }
    } catch (error) {
      console.error("Error sending check-in to server:", error);
      // Do not remove anything from local storage if there's an error
    }
  }
}

// send checkin to server
async function sendCheckInToServer(checkInData) {
  console.log("sending checkin to server:", checkInData);
  try {
    // Fetch user details
    let userResponse = await fetch(
      `/user/${encodeURIComponent(checkInData.userId)}`
    );
    if (!userResponse.ok) {
      if (userResponse.status === 400 || userResponse.status === 404) {
        console.error("User not found.");
        // return not found
        return userResponse;
      } else {
        console.error(
          `Error fetching user details: ${userResponse.statusText}`
        );
      }
      return; // Skip to next if user not found or other error
    }

    // Get last punch details
    let newStatus = await getLastPunchAndCalculateNewStatus(checkInData.userId);
    if (!newStatus) return;

    // Update status with no timer
    return await updateStatus(
      newStatus,
      checkInData.userId,
      (showMessage = false)
    );
  } catch (error) {
    console.error("Error during check-in process:", error);
  }
}

// Fetch last punch details and calculate new status
async function getLastPunchAndCalculateNewStatus(userId) {
  let punchResponse = await fetch(
    `/punch/${encodeURIComponent(userId)}/last_punch`
  );
  if (!punchResponse.ok) {
    console.error(`Error fetching last punch: ${punchResponse.statusText}`);
    return null;
  }
  let lastPunchData = await punchResponse.json();

  // Calculate opposite status
  let currentStatus = lastPunchData.in_out;
  let newStatus = currentStatus === "In" ? "Out" : "In";

  return newStatus;
}

// update status
async function updateStatus(status, userId, showMessage = true) {
  const messageDiv = document.getElementById("statusMessage");
  let punchUrl = `/punch/${encodeURIComponent(userId)}`;
  device_id = localStorage.getItem("device_id");

  try {
    let punchResponse = await fetch(punchUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ device_id: device_id, in_out: status }),
    });

    if (punchResponse.ok) {
      if (showMessage) {
        messageDiv.textContent = `Status updated to ${status}`;
        await wait(2000);
        messageDiv.textContent = "";
        window.location.reload();
      } else {
        console.log(`Status updated to ${status}`);
      }
      let responseData = await punchResponse.json();
      return responseData;
    } else {
      if (showMessage) {
        messageDiv.textContent = `Failed to update status: ${punchResponse.statusText}`;
        await wait(2000);
        messageDiv.textContent = "";
      } else {
        console.error(`Failed to update status: ${punchResponse.statusText}`);
      }
      throw new Error(`Failed to update status: ${punchResponse.statusText}`);
    }
  } catch (error) {
    if (showMessage) {
      messageDiv.textContent = `Network error during status update: ${error}`;
      await wait(2000);
      messageDiv.textContent = "";
    } else {
      console.error("Network error during status update:", error);
    }
    throw error;
  }
}

// register service worker
if (navigator.serviceWorker) {
  navigator.serviceWorker.register("static/js/service-worker.js").then(() => {
    console.log("Service Worker Registered");
  });
}
