// Synced from tests/adapter_fixtures.rs lines 538-563

import type { EventType } from '../data/protocols';
import type { LifecycleTransition } from './lifecycle';

const EVENT_TO_TRANSITION: Record<EventType, LifecycleTransition | null> = {
  Created: { type: 'Create' },
  FillInitiated: { type: 'FillDelta' },
  FillCompleted: { type: 'FillDelta' },
  Cancelled: { type: 'Close', status: 'cancelled' },
  Expired: { type: 'Close', status: 'expired' },
  Closed: null,
  FeeCollected: { type: 'MetadataOnly' },
  Withdrawn: { type: 'MetadataOnly' },
  Deposited: { type: 'MetadataOnly' },
};

export function eventTypeToTransition(
  eventType: EventType,
  closedStatus?: string,
): LifecycleTransition {
  if (eventType === 'Closed' && closedStatus) {
    return {
      type: 'Close',
      status: closedStatus.toLowerCase() as 'completed' | 'cancelled' | 'expired',
    };
  }
  return EVENT_TO_TRANSITION[eventType] ?? { type: 'MetadataOnly' };
}

export function transitionToString(transition: LifecycleTransition): string {
  if (transition.type === 'Close') {
    return `Close(${transition.status.charAt(0).toUpperCase() + transition.status.slice(1)})`;
  }
  return transition.type;
}

export function transitionTarget(transition: LifecycleTransition): string | null {
  if (transition.type === 'Close') return transition.status;
  if (transition.type === 'Create') return 'active';
  return null;
}
