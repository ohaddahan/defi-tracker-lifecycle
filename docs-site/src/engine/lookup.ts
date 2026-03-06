import type { EventType, ProtocolId } from '../data/protocols';
import { wasmLookupVariant } from './wasm';

export interface VariantLookupResult {
  variantName: string;
  source: 'event' | 'instruction';
  eventType: EventType;
  transition: string;
  decisionFromNone: string;
  notes: string[];
}

export function lookupVariant(
  json: string,
  protocolId: ProtocolId,
): VariantLookupResult | { error: string } {
  const wasmProtocol = protocolIdToWasm(protocolId);
  const result = wasmLookupVariant(wasmProtocol, json);

  if (result && typeof result === 'object' && 'error' in result) {
    return { error: result.error as string };
  }

  return result as VariantLookupResult;
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
