// Powered by WASM — logic from src/lifecycle/mapping.rs

import type { EventType } from '../data/protocols';
import type { LifecycleTransition } from './lifecycle';
import {
  wasmEventTypeToTransition,
  wasmTransitionToString,
  wasmTransitionTarget,
} from './wasm';

export function eventTypeToTransition(
  eventType: EventType,
  closedStatus?: string,
): LifecycleTransition {
  return wasmEventTypeToTransition(eventType, closedStatus) as LifecycleTransition;
}

export function transitionToString(transition: LifecycleTransition): string {
  const closeStatus =
    transition.type === 'Close' ? transition.status : undefined;
  return wasmTransitionToString(transition.type, closeStatus);
}

export function transitionTarget(transition: LifecycleTransition): string | null {
  const closeStatus =
    transition.type === 'Close' ? transition.status : undefined;
  return wasmTransitionTarget(transition.type, closeStatus) ?? null;
}
