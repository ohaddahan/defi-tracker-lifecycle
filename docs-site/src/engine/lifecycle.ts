// Synced from src/lifecycle/mod.rs

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

const TERMINAL_STATUSES: ReadonlySet<string> = new Set([
  'completed',
  'cancelled',
  'expired',
]);

export function isTerminal(status: string | null): boolean {
  return status !== null && TERMINAL_STATUSES.has(status);
}

export function decideTransition(
  currentStatus: string | null,
  transition: LifecycleTransition,
): TransitionDecision {
  if (!isTerminal(currentStatus)) return 'Apply';
  if (transition.type === 'MetadataOnly') return 'Apply';
  return 'IgnoreTerminalViolation';
}

export function normalizeSnapshotToDelta(
  storedTotal: number,
  snapshotTotal: number,
): SnapshotDelta {
  const raw = snapshotTotal - storedTotal;
  return {
    delta: Math.max(0, raw),
    regression: snapshotTotal < storedTotal,
  };
}
