// Powered by WASM — logic from src/lifecycle/mod.rs

import {
  wasmDecideTransition,
  wasmIsTerminal,
  wasmNormalizeSnapshot,
} from './wasm';

export type TerminalStatus = 'completed' | 'cancelled' | 'expired';

export type LifecycleTransition =
  | { type: 'Create' }
  | { type: 'FillDelta' }
  | { type: 'Close'; status: TerminalStatus }
  | { type: 'MetadataOnly' };

export type TransitionDecision = 'Apply' | 'IgnoreTerminalViolation';

export interface SnapshotDelta {
  delta: number;
  regression: boolean;
}

export function isTerminal(status: string | null): boolean {
  return wasmIsTerminal(status);
}

export function decideTransition(
  currentStatus: string | null,
  transition: LifecycleTransition,
): TransitionDecision {
  const closeStatus =
    transition.type === 'Close' ? transition.status : undefined;
  return wasmDecideTransition(
    currentStatus,
    transition.type,
    closeStatus,
  ) as TransitionDecision;
}

export function normalizeSnapshotToDelta(
  storedTotal: number,
  snapshotTotal: number,
): SnapshotDelta {
  return wasmNormalizeSnapshot(storedTotal, snapshotTotal) as SnapshotDelta;
}
