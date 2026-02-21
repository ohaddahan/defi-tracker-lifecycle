// Mapping data powered by WASM — UI metadata stays in TS

import { wasmGetAllProtocols } from '../engine/wasm';

export type EventType =
  | 'Created'
  | 'FillInitiated'
  | 'FillCompleted'
  | 'Cancelled'
  | 'Expired'
  | 'Closed'
  | 'FeeCollected'
  | 'Withdrawn'
  | 'Deposited';

export type ProtocolId = 'dca' | 'limitV1' | 'limitV2' | 'kamino';

export interface ProtocolConfig {
  id: ProtocolId;
  label: string;
  sub: string;
  programId: string;
  instructions: Record<string, EventType>;
  events: Record<string, EventType>;
  closedVariants: string[];
  notes: string[];
}

interface WasmProtocolData {
  id: string;
  programId: string;
  instructions: Record<string, string>;
  events: Record<string, string>;
  closedVariants: string[];
}

const UI_METADATA: Record<
  string,
  { id: ProtocolId; label: string; sub: string; notes: string[] }
> = {
  dca: {
    id: 'dca',
    label: 'Jupiter DCA',
    sub: 'Dollar-Cost Averaging',
    notes: [
      'ClosedEvent terminal status derived from user_closed + unfilled_amount fields',
      'Priority: user_closed → Cancelled, unfilled_amount == 0 → Completed, else → Expired',
      'Transfer/Deposit/Withdraw/WithdrawFees instructions are classified as None (irrelevant)',
    ],
  },
  limit_v1: {
    id: 'limitV1',
    label: 'Jupiter Limit V1',
    sub: 'Limit Orders V1',
    notes: [
      'CancelExpiredOrder instruction maps to Expired EventType (V2 has no expiry instruction)',
      'TradeEvent uses in_amount/out_amount field names (V1 naming)',
      'WithdrawFee/InitFee/UpdateFee instructions are classified as None',
    ],
  },
  limit_v2: {
    id: 'limitV2',
    label: 'Jupiter Limit V2',
    sub: 'Limit Orders V2',
    notes: [
      'No CancelExpiredOrder — V2 has no expiry instruction',
      'TradeEvent uses making_amount/taking_amount field names (V2 naming)',
      'Args may be nested in {"params": {...}} wrapper or flat — adapter handles both',
      'UpdateFee/WithdrawFee instructions are classified as None',
    ],
  },
  kamino: {
    id: 'kamino',
    label: 'Kamino',
    sub: 'Kamino Limit Orders',
    notes: [
      'OrderDisplayEvent has no order PDA — requires pre_fetched_order_pdas from instruction accounts',
      'Returns Uncorrelated if PDAs missing',
      'UserSwapBalancesEvent is diagnostic-only (NotRequired correlation → MetadataOnly transition)',
      'Status codes: 0=Open, 1=Filled(Completed), 2=Cancelled, 3=Expired',
      'Admin instructions (InitializeGlobalConfig, etc.) are classified as None',
    ],
  },
};

function buildProtocols(): Record<ProtocolId, ProtocolConfig> {
  const wasmData = wasmGetAllProtocols() as WasmProtocolData[];
  const result = {} as Record<ProtocolId, ProtocolConfig>;

  for (const wp of wasmData) {
    const meta = UI_METADATA[wp.id];
    if (!meta) continue;

    result[meta.id] = {
      id: meta.id,
      label: meta.label,
      sub: meta.sub,
      programId: wp.programId,
      instructions: wp.instructions as Record<string, EventType>,
      events: wp.events as Record<string, EventType>,
      closedVariants: wp.closedVariants,
      notes: meta.notes,
    };
  }

  return result;
}

export const PROTOCOLS: Record<ProtocolId, ProtocolConfig> = buildProtocols();

export const PROTOCOL_LIST: ProtocolConfig[] = Object.values(PROTOCOLS);
