import { useCallback, useState } from 'react';
import type { EventType, ProtocolId } from '../data/protocols';
import { decideTransition, isTerminal, type TransitionDecision } from '../engine/lifecycle';
import { eventTypeToTransition, transitionToString, transitionTarget } from '../engine/eventMapping';

export interface LogEntry {
  step: number;
  from: string;
  transition: string;
  to: string;
  decision: TransitionDecision;
  eventType: EventType;
  closedStatus: string | null;
}

interface LifecycleState {
  protocol: ProtocolId;
  currentStatus: string | null;
  fills: number;
  log: LogEntry[];
  setProtocol: (p: ProtocolId) => void;
  fireEvent: (eventType: EventType, closedStatus: string | null) => void;
  reset: () => void;
  isTerminal: boolean;
}

export function useLifecycleState(initialProtocol: ProtocolId = 'dca'): LifecycleState {
  const [protocol, setProtocol] = useState<ProtocolId>(initialProtocol);
  const [currentStatus, setCurrentStatus] = useState<string | null>(null);
  const [fills, setFills] = useState(0);
  const [log, setLog] = useState<LogEntry[]>([]);

  const fireEvent = useCallback(
    (eventType: EventType, closedStatus: string | null) => {
      const transition = eventTypeToTransition(eventType, closedStatus ?? undefined);
      const transStr = transitionToString(transition);
      const decision = decideTransition(currentStatus, transition);
      const from = currentStatus ?? 'none';
      let to = from;

      if (decision === 'Apply') {
        const target = transitionTarget(transition);
        if (target) {
          setCurrentStatus(target);
          to = target;
        } else if (currentStatus === null && transition.type !== 'MetadataOnly') {
          setCurrentStatus('active');
          to = 'active';
        } else {
          to = currentStatus ?? 'none';
        }
        if (transition.type === 'FillDelta') {
          setFills((f) => f + 1);
        }
      }

      setLog((prev) => [
        ...prev,
        {
          step: prev.length + 1,
          from,
          transition: transStr,
          to,
          decision,
          eventType,
          closedStatus,
        },
      ]);
    },
    [currentStatus],
  );

  const reset = useCallback(() => {
    setCurrentStatus(null);
    setFills(0);
    setLog([]);
  }, []);

  return {
    protocol,
    currentStatus,
    fills,
    log,
    setProtocol,
    fireEvent,
    reset,
    isTerminal: isTerminal(currentStatus),
  };
}
