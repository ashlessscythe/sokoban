const cacheName = "site-static-v1";
const assets = [
  "static/css/main.css", // Your main stylesheet
  "static/js/helpers.js", // Your main JavaScript file
  // Add paths for other scripts you want to cache
  "templates/home.html.tera", // Add cached pages
  "templates/checklist.html.tera", // Add cached pages
  "templates/status_list.html.tera", // Add cached pages
  "static/images/logo.png", // Include your logo and other essential images
  // Other HTML templates and assets as needed
];

// Install service worker
self.addEventListener("install", (evt) => {
  evt.waitUntil(
    caches.open(cacheName).then((cache) => {
      console.log("caching shell assets");
      cache.addAll(assets);
    })
  );
});

// Activate service worker
self.addEventListener("activate", (evt) => {
  evt.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.filter((key) => key !== cacheName).map((key) => caches.delete(key))
      );
    })
  );
});

// Fetch event
self.addEventListener("fetch", (evt) => {
  evt.respondWith(
    caches.match(evt.request).then((cacheRes) => {
      return cacheRes || fetch(evt.request);
    })
  );
});
