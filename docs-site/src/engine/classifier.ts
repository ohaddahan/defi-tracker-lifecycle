// Powered by WASM — logic from src/protocols/ classify functions

import type { EventType, ProtocolId } from '../data/protocols';
import { wasmClassifyJson } from './wasm';

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
  const wasmProtocol = protocolIdToWasm(protocolId);
  const result = wasmClassifyJson(wasmProtocol, json);
  if (result && typeof result === 'object' && 'error' in result) {
    return { error: result.error as string };
  }
  return result as ClassificationResult;
}

function protocolIdToWasm(id: ProtocolId): string {
  switch (id) {
    case 'dca':
      return 'dca';
    case 'limitV1':
      return 'limitV1';
    case 'limitV2':
      return 'limitV2';
    case 'kamino':
      return 'kamino';
  }
}
