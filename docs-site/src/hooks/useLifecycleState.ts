import { useReducer } from 'react';
import type { EventType, ProtocolId } from '../data/protocols';
import { decideTransition, isTerminal, type TransitionDecision } from '../engine/lifecycle';
import {
  eventTypeToTransition,
  transitionTarget,
  transitionToString,
} from '../engine/eventMapping';

export interface LogEntry {
  step: number;
  from: string;
  transition: string;
  to: string;
  decision: TransitionDecision;
  eventType: EventType;
  closedStatus: string | null;
}

export interface TransitionFlash {
  step: number;
  target: string;
  decision: TransitionDecision;
  transition: string;
}

interface LifecycleSnapshot {
  protocol: ProtocolId;
  currentStatus: string | null;
  fills: number;
  log: LogEntry[];
  lastTransition: TransitionFlash | null;
}

interface LifecycleState extends LifecycleSnapshot {
  setProtocol: (protocol: ProtocolId) => void;
  fireEvent: (eventType: EventType, closedStatus: string | null) => void;
  reset: () => void;
  isTerminal: boolean;
}

type Action =
  | { type: 'set_protocol'; protocol: ProtocolId }
  | { type: 'reset' }
  | { type: 'fire_event'; eventType: EventType; closedStatus: string | null };

function createSnapshot(protocol: ProtocolId): LifecycleSnapshot {
  return {
    protocol,
    currentStatus: null,
    fills: 0,
    log: [],
    lastTransition: null,
  };
}

function reduceLifecycleState(
  snapshot: LifecycleSnapshot,
  action: Action,
): LifecycleSnapshot {
  switch (action.type) {
    case 'set_protocol':
      return createSnapshot(action.protocol);
    case 'reset':
      return createSnapshot(snapshot.protocol);
    case 'fire_event': {
      const transition = eventTypeToTransition(
        action.eventType,
        action.closedStatus ?? undefined,
      );
      const transitionLabel = transitionToString(transition);
      const decision = decideTransition(snapshot.currentStatus, transition);
      const from = snapshot.currentStatus ?? 'none';
      let to = from;
      let nextStatus = snapshot.currentStatus;
      let fills = snapshot.fills;

      if (decision === 'Apply') {
        const target = transitionTarget(transition);
        if (target) {
          nextStatus = target;
          to = target;
        } else if (snapshot.currentStatus === null && transition.type !== 'MetadataOnly') {
          nextStatus = 'active';
          to = 'active';
        }

        if (transition.type === 'FillDelta') {
          fills += 1;
        }
      }

      const entry: LogEntry = {
        step: snapshot.log.length + 1,
        from,
        transition: transitionLabel,
        to,
        decision,
        eventType: action.eventType,
        closedStatus: action.closedStatus,
      };

      return {
        ...snapshot,
        currentStatus: nextStatus,
        fills,
        log: [...snapshot.log, entry],
        lastTransition: {
          step: entry.step,
          target: to,
          decision,
          transition: transitionLabel,
        },
      };
    }
  }
}

export function useLifecycleState(initialProtocol: ProtocolId = 'dca'): LifecycleState {
  const [snapshot, dispatch] = useReducer(
    reduceLifecycleState,
    initialProtocol,
    createSnapshot,
  );

  return {
    ...snapshot,
    setProtocol: (protocol) => dispatch({ type: 'set_protocol', protocol }),
    fireEvent: (eventType, closedStatus) =>
      dispatch({ type: 'fire_event', eventType, closedStatus }),
    reset: () => dispatch({ type: 'reset' }),
    isTerminal: isTerminal(snapshot.currentStatus),
  };
}
