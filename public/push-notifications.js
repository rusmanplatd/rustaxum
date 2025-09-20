/**
 * Push Notifications Client Library for RustAxum
 * Handles service worker registration, subscription management, and push notifications
 */

class PushNotificationManager {
  constructor(options = {}) {
    this.options = {
      serviceWorkerPath: '/sw.js',
      apiBase: '/api/web-push',
      debug: false,
      ...options
    };

    this.serviceWorkerRegistration = null;
    this.pushSubscription = null;
    this.vapidPublicKey = null;
    this.isSupported = this.checkSupport();

    this.log('Initialized PushNotificationManager', {
      supported: this.isSupported,
      options: this.options
    });
  }

  /**
   * Check if push notifications are supported
   */
  checkSupport() {
    if (!('serviceWorker' in navigator)) {
      this.log('Service workers not supported');
      return false;
    }

    if (!('PushManager' in window)) {
      this.log('Push messaging not supported');
      return false;
    }

    if (!('Notification' in window)) {
      this.log('Notifications not supported');
      return false;
    }

    return true;
  }

  /**
   * Initialize push notifications
   */
  async initialize() {
    if (!this.isSupported) {
      throw new Error('Push notifications are not supported in this browser');
    }

    try {
      // Register service worker
      await this.registerServiceWorker();

      // Get VAPID public key
      await this.getVapidPublicKey();

      // Check current subscription status
      await this.checkSubscriptionStatus();

      this.log('Push notifications initialized successfully');
      return true;
    } catch (error) {
      this.log('Failed to initialize push notifications:', error);
      throw error;
    }
  }

  /**
   * Register the service worker
   */
  async registerServiceWorker() {
    try {
      this.serviceWorkerRegistration = await navigator.serviceWorker.register(
        this.options.serviceWorkerPath
      );

      this.log('Service worker registered:', this.serviceWorkerRegistration);

      // Wait for service worker to be ready
      await navigator.serviceWorker.ready;

      return this.serviceWorkerRegistration;
    } catch (error) {
      this.log('Service worker registration failed:', error);
      throw error;
    }
  }

  /**
   * Get VAPID public key from server
   */
  async getVapidPublicKey() {
    try {
      const response = await fetch(`${this.options.apiBase}/vapid-public-key`);

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      this.vapidPublicKey = data.public_key;

      this.log('VAPID public key retrieved:', this.vapidPublicKey);
      return this.vapidPublicKey;
    } catch (error) {
      this.log('Failed to get VAPID public key:', error);
      throw error;
    }
  }

  /**
   * Check current subscription status
   */
  async checkSubscriptionStatus() {
    if (!this.serviceWorkerRegistration) {
      return false;
    }

    try {
      this.pushSubscription = await this.serviceWorkerRegistration.pushManager.getSubscription();

      if (this.pushSubscription) {
        this.log('Existing push subscription found:', this.pushSubscription);
        return true;
      } else {
        this.log('No existing push subscription');
        return false;
      }
    } catch (error) {
      this.log('Failed to check subscription status:', error);
      return false;
    }
  }

  /**
   * Request notification permission
   */
  async requestPermission() {
    if (!('Notification' in window)) {
      throw new Error('Notifications not supported');
    }

    if (Notification.permission === 'granted') {
      this.log('Notification permission already granted');
      return true;
    }

    if (Notification.permission === 'denied') {
      this.log('Notification permission denied');
      return false;
    }

    try {
      const permission = await Notification.requestPermission();
      this.log('Notification permission result:', permission);
      return permission === 'granted';
    } catch (error) {
      this.log('Failed to request notification permission:', error);
      return false;
    }
  }

  /**
   * Subscribe to push notifications
   */
  async subscribe() {
    if (!this.isSupported) {
      throw new Error('Push notifications not supported');
    }

    try {
      // Request permission first
      const hasPermission = await this.requestPermission();
      if (!hasPermission) {
        throw new Error('Notification permission not granted');
      }

      // Ensure we have service worker and VAPID key
      if (!this.serviceWorkerRegistration) {
        await this.registerServiceWorker();
      }

      if (!this.vapidPublicKey) {
        await this.getVapidPublicKey();
      }

      // Subscribe to push
      const subscription = await this.serviceWorkerRegistration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: this.urlBase64ToUint8Array(this.vapidPublicKey)
      });

      this.log('Push subscription created:', subscription);

      // Send subscription to server
      const result = await this.sendSubscriptionToServer(subscription);

      if (result.success) {
        this.pushSubscription = subscription;
        this.log('Successfully subscribed to push notifications');
        this.dispatchEvent('subscribed', { subscription, result });
        return subscription;
      } else {
        throw new Error(result.message || 'Failed to save subscription');
      }
    } catch (error) {
      this.log('Failed to subscribe to push notifications:', error);
      this.dispatchEvent('subscription-error', { error });
      throw error;
    }
  }

  /**
   * Unsubscribe from push notifications
   */
  async unsubscribe() {
    if (!this.pushSubscription) {
      this.log('No active subscription to unsubscribe from');
      return true;
    }

    try {
      // Unsubscribe from browser
      const result = await this.pushSubscription.unsubscribe();

      if (result) {
        // Remove subscription from server
        await this.removeSubscriptionFromServer(this.pushSubscription);

        this.pushSubscription = null;
        this.log('Successfully unsubscribed from push notifications');
        this.dispatchEvent('unsubscribed');
        return true;
      } else {
        throw new Error('Failed to unsubscribe from browser');
      }
    } catch (error) {
      this.log('Failed to unsubscribe from push notifications:', error);
      this.dispatchEvent('unsubscription-error', { error });
      throw error;
    }
  }

  /**
   * Send subscription to server
   */
  async sendSubscriptionToServer(subscription) {
    const subscriptionData = {
      endpoint: subscription.endpoint,
      keys: {
        p256dh: this.arrayBufferToBase64(subscription.getKey('p256dh')),
        auth: this.arrayBufferToBase64(subscription.getKey('auth'))
      }
    };

    try {
      const response = await fetch(`${this.options.apiBase}/subscribe`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': this.getAuthHeader()
        },
        body: JSON.stringify(subscriptionData)
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      this.log('Subscription sent to server:', result);
      return result;
    } catch (error) {
      this.log('Failed to send subscription to server:', error);
      throw error;
    }
  }

  /**
   * Remove subscription from server
   */
  async removeSubscriptionFromServer(subscription) {
    try {
      const response = await fetch(`${this.options.apiBase}/unsubscribe`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': this.getAuthHeader()
        },
        body: JSON.stringify({
          endpoint: subscription.endpoint
        })
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      this.log('Subscription removed from server:', result);
      return result;
    } catch (error) {
      this.log('Failed to remove subscription from server:', error);
      throw error;
    }
  }

  /**
   * Send test notification
   */
  async sendTestNotification(title = 'Test Notification', message = 'This is a test!') {
    try {
      const response = await fetch(`${this.options.apiBase}/test?title=${encodeURIComponent(title)}&message=${encodeURIComponent(message)}`, {
        method: 'POST',
        headers: {
          'Authorization': this.getAuthHeader()
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      this.log('Test notification sent:', result);
      return result;
    } catch (error) {
      this.log('Failed to send test notification:', error);
      throw error;
    }
  }

  /**
   * Get subscription status
   */
  getSubscriptionStatus() {
    return {
      supported: this.isSupported,
      permission: Notification.permission,
      subscribed: !!this.pushSubscription,
      subscription: this.pushSubscription
    };
  }

  /**
   * Get authorization header (should be implemented based on your auth system)
   */
  getAuthHeader() {
    // This should return the appropriate authorization header
    // For example: 'Bearer ' + token
    const token = localStorage.getItem('auth_token') || sessionStorage.getItem('auth_token');
    return token ? `Bearer ${token}` : '';
  }

  /**
   * Dispatch custom events
   */
  dispatchEvent(eventName, detail = {}) {
    const event = new CustomEvent(`push-notification-${eventName}`, { detail });
    window.dispatchEvent(event);
    this.log(`Event dispatched: ${eventName}`, detail);
  }

  /**
   * Utility: Convert URL-safe base64 to Uint8Array
   */
  urlBase64ToUint8Array(base64String) {
    const padding = '='.repeat((4 - base64String.length % 4) % 4);
    const base64 = (base64String + padding)
      .replace(/\-/g, '+')
      .replace(/_/g, '/');

    const rawData = window.atob(base64);
    const outputArray = new Uint8Array(rawData.length);

    for (let i = 0; i < rawData.length; ++i) {
      outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
  }

  /**
   * Utility: Convert ArrayBuffer to base64
   */
  arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
  }

  /**
   * Debug logging
   */
  log(...args) {
    if (this.options.debug) {
      console.log('[PushNotificationManager]', ...args);
    }
  }
}

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  // Create global instance
  window.pushNotificationManager = new PushNotificationManager({
    debug: true // Set to false in production
  });

  // Auto-initialize if supported
  if (window.pushNotificationManager.isSupported) {
    window.pushNotificationManager.initialize().catch(error => {
      console.warn('Failed to auto-initialize push notifications:', error);
    });
  }
});

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
  module.exports = PushNotificationManager;
}