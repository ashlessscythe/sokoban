async function getDeviceId() {
  // Initialize an agent at application startup.
  const fpPromise = import("https://openfpcdn.io/fingerprintjs/v4").then(
    (FingerprintJS) => FingerprintJS.load()
  );

  // Get the visitor identifier when you need it.
  const fp = await fpPromise;
  const result = await fp.get();
  return result.visitorId;
}

async function storeDeviceId() {
  const deviceId = await getDeviceId();
  localStorage.setItem("device_id", deviceId);
}
