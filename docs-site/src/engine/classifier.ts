// Client-side JSON key-name classifier for the JSON panel

import { PROTOCOLS, type EventType, type ProtocolId } from '../data/protocols';
import { eventTypeToTransition, transitionToString } from './eventMapping';
import { decideTransition } from './lifecycle';

export interface ClassificationResult {
  variantName: string;
  source: 'event' | 'instruction';
  eventType: EventType;
  transition: string;
  decision: string;
}

export function classifyJson(
  json: string,
  protocolId: ProtocolId,
): ClassificationResult | { error: string } {
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(json);
  } catch {
    return { error: 'Invalid JSON' };
  }

  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    return { error: 'Expected a JSON object with a variant key, e.g. {"FilledEvent": {...}}' };
  }

  const keys = Object.keys(parsed);
  if (keys.length === 0) {
    return { error: 'Empty JSON object — expected a variant key' };
  }

  const protocol = PROTOCOLS[protocolId];
  const variantName = keys[0];

  if (variantName in protocol.events) {
    const eventType = protocol.events[variantName];
    const transition = eventTypeToTransition(eventType);
    return {
      variantName,
      source: 'event',
      eventType,
      transition: transitionToString(transition),
      decision: decideTransition(null, transition),
    };
  }

  if (variantName in protocol.instructions) {
    const eventType = protocol.instructions[variantName];
    const transition = eventTypeToTransition(eventType);
    return {
      variantName,
      source: 'instruction',
      eventType,
      transition: transitionToString(transition),
      decision: decideTransition(null, transition),
    };
  }

  return {
    error: `Unknown variant "${variantName}" for ${protocol.label}. Known events: ${Object.keys(protocol.events).join(', ')}. Known instructions: ${Object.keys(protocol.instructions).join(', ')}.`,
  };
}
