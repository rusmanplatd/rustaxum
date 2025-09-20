// Service Worker for Web Push Notifications
// This file handles push notifications in the background

const CACHE_NAME = 'rustaxum-v1';
const urlsToCache = [
  '/',
  '/static/css/app.css',
  '/static/js/app.js',
  // Add other important assets to cache
];

// Install event - cache important resources
self.addEventListener('install', event => {
  console.log('Service Worker installing...');

  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('Opened cache');
        return cache.addAll(urlsToCache);
      })
      .catch(error => {
        console.error('Cache installation failed:', error);
      })
  );

  // Force the waiting service worker to become the active service worker
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  console.log('Service Worker activating...');

  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName !== CACHE_NAME) {
            console.log('Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );

  // Ensure the service worker takes control of all pages immediately
  self.clients.claim();
});

// Fetch event - serve cached content when offline
self.addEventListener('fetch', event => {
  // Only handle GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  event.respondWith(
    caches.match(event.request)
      .then(response => {
        // Return cached version or fetch from network
        return response || fetch(event.request);
      })
      .catch(error => {
        console.error('Fetch failed:', error);
        // Optionally return a fallback page
      })
  );
});

// Push event - handle incoming push notifications
self.addEventListener('push', event => {
  console.log('Push notification received:', event);

  // Default notification options
  const defaultOptions = {
    icon: '/static/images/icon-192x192.png',
    badge: '/static/images/badge-72x72.png',
    vibrate: [100, 50, 100],
    data: {
      dateOfArrival: Date.now(),
      primaryKey: '1'
    },
    actions: [
      {
        action: 'explore',
        title: 'View',
        icon: '/static/images/checkmark.png'
      },
      {
        action: 'close',
        title: 'Close',
        icon: '/static/images/xmark.png'
      }
    ],
    requireInteraction: false,
    silent: false
  };

  let notificationData = {};

  // Parse notification data from push event
  if (event.data) {
    try {
      notificationData = event.data.json();
      console.log('Notification data:', notificationData);
    } catch (e) {
      console.error('Error parsing notification data:', e);
      notificationData = {
        title: 'New Notification',
        body: event.data.text() || 'You have a new notification from RustAxum'
      };
    }
  } else {
    notificationData = {
      title: 'New Notification',
      body: 'You have a new notification from RustAxum'
    };
  }

  // Merge default options with notification data
  const options = {
    ...defaultOptions,
    ...notificationData,
    body: notificationData.body || notificationData.message,
    tag: notificationData.tag || 'rustaxum-notification',
    icon: notificationData.icon || defaultOptions.icon,
    badge: notificationData.badge || defaultOptions.badge,
    image: notificationData.image,
    data: {
      ...defaultOptions.data,
      ...notificationData.data,
      url: notificationData.action_url || notificationData.url || '/'
    }
  };

  // Show the notification
  event.waitUntil(
    self.registration.showNotification(notificationData.title || 'RustAxum', options)
      .then(() => {
        console.log('Notification displayed successfully');
      })
      .catch(error => {
        console.error('Error displaying notification:', error);
      })
  );
});

// Notification click event - handle user interaction
self.addEventListener('notificationclick', event => {
  console.log('Notification clicked:', event);

  const notification = event.notification;
  const action = event.action;
  const data = notification.data || {};

  // Close the notification
  notification.close();

  // Handle different actions
  if (action === 'close') {
    console.log('Notification dismissed by user');
    return;
  }

  // Default action or 'explore' action - open the app
  const urlToOpen = data.url || '/';

  event.waitUntil(
    clients.matchAll({
      type: 'window',
      includeUncontrolled: true
    }).then(clientList => {
      // Check if there's already a window/tab open with the target URL
      for (const client of clientList) {
        if (client.url === urlToOpen && 'focus' in client) {
          return client.focus();
        }
      }

      // If no existing window, open a new one
      if (clients.openWindow) {
        return clients.openWindow(urlToOpen);
      }
    }).catch(error => {
      console.error('Error handling notification click:', error);
    })
  );
});

// Notification close event - handle when user dismisses notification
self.addEventListener('notificationclose', event => {
  console.log('Notification closed:', event);

  // Optional: Track dismissal analytics
  const notification = event.notification;
  const data = notification.data || {};

  // Send analytics or tracking data if needed
  // trackNotificationDismissal(data);
});

// Push subscription change event - handle subscription updates
self.addEventListener('pushsubscriptionchange', event => {
  console.log('Push subscription changed:', event);

  event.waitUntil(
    // Re-subscribe with new subscription
    self.registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: getApplicationServerKey()
    }).then(subscription => {
      console.log('Re-subscribed with new subscription:', subscription);

      // Send new subscription to server
      return fetch('/api/web-push/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          endpoint: subscription.endpoint,
          keys: {
            p256dh: arrayBufferToBase64(subscription.getKey('p256dh')),
            auth: arrayBufferToBase64(subscription.getKey('auth'))
          }
        })
      });
    }).catch(error => {
      console.error('Failed to re-subscribe:', error);
    })
  );
});

// Helper function to get application server key (VAPID public key)
function getApplicationServerKey() {
  // This should be set when the service worker is first registered
  // For now, we'll fetch it from the server
  return fetch('/api/web-push/vapid-public-key')
    .then(response => response.json())
    .then(data => urlBase64ToUint8Array(data.public_key))
    .catch(error => {
      console.error('Failed to get VAPID public key:', error);
      return null;
    });
}

// Helper function to convert array buffer to base64
function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

// Helper function to convert URL-safe base64 to Uint8Array
function urlBase64ToUint8Array(base64String) {
  const padding = '='.repeat((4 - base64String.length % 4) % 4);
  const base64 = (base64String + padding)
    .replace(/\-/g, '+')
    .replace(/_/g, '/');

  const rawData = atob(base64);
  const outputArray = new Uint8Array(rawData.length);

  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

// Background sync for offline actions (optional)
self.addEventListener('sync', event => {
  console.log('Background sync triggered:', event.tag);

  if (event.tag === 'background-sync') {
    event.waitUntil(
      // Handle background sync tasks
      handleBackgroundSync()
    );
  }
});

function handleBackgroundSync() {
  // Implement background sync logic here
  // For example, send queued notifications or sync data
  return Promise.resolve();
}