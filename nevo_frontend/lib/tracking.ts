/**
 * Simple tracking utility for user interactions
 */
export interface TrackingEvent {
  name: string;
  poolId?: string;
  platform?: string;
  timestamp?: number;
}

class TrackingService {
  private events: TrackingEvent[] = [];

  /**
   * Track a share event
   */
  trackShareClick(poolId: string, platform: string) {
    const event: TrackingEvent = {
      name: 'pool_share',
      poolId,
      platform,
      timestamp: Date.now(),
    };
    this.events.push(event);
    console.debug('[Tracking]', event);
  }

  /**
   * Get all tracked events
   */
  getEvents() {
    return this.events;
  }

  /**
   * Clear all tracked events
   */
  clearEvents() {
    this.events = [];
  }
}

export const trackingService = new TrackingService();
