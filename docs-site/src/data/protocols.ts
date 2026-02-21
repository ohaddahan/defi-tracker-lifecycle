// Synced from src/protocols/dca.rs, limit_v1.rs, limit_v2.rs, kamino.rs

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

export const PROTOCOLS: Record<ProtocolId, ProtocolConfig> = {
  dca: {
    id: 'dca',
    label: 'Jupiter DCA',
    sub: 'Dollar-Cost Averaging',
    programId: 'DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M',
    instructions: {
      OpenDca: 'Created',
      OpenDcaV2: 'Created',
      InitiateFlashFill: 'FillInitiated',
      InitiateDlmmFill: 'FillInitiated',
      FulfillFlashFill: 'FillCompleted',
      FulfillDlmmFill: 'FillCompleted',
      CloseDca: 'Closed',
      EndAndClose: 'Closed',
    },
    events: {
      OpenedEvent: 'Created',
      FilledEvent: 'FillCompleted',
      ClosedEvent: 'Closed',
      CollectedFeeEvent: 'FeeCollected',
      WithdrawEvent: 'Withdrawn',
      DepositEvent: 'Deposited',
    },
    closedVariants: ['Completed', 'Cancelled', 'Expired'],
    notes: [
      'ClosedEvent terminal status derived from user_closed + unfilled_amount fields',
      'Priority: user_closed → Cancelled, unfilled_amount == 0 → Completed, else → Expired',
      'Transfer/Deposit/Withdraw/WithdrawFees instructions are classified as None (irrelevant)',
    ],
  },
  limitV1: {
    id: 'limitV1',
    label: 'Jupiter Limit V1',
    sub: 'Limit Orders V1',
    programId: 'jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu',
    instructions: {
      InitializeOrder: 'Created',
      PreFlashFillOrder: 'FillInitiated',
      FillOrder: 'FillCompleted',
      FlashFillOrder: 'FillCompleted',
      CancelOrder: 'Cancelled',
      CancelExpiredOrder: 'Expired',
    },
    events: {
      CreateOrderEvent: 'Created',
      CancelOrderEvent: 'Cancelled',
      TradeEvent: 'FillCompleted',
    },
    closedVariants: [],
    notes: [
      'CancelExpiredOrder instruction maps to Expired EventType (V2 has no expiry instruction)',
      'TradeEvent uses in_amount/out_amount field names (V1 naming)',
      'WithdrawFee/InitFee/UpdateFee instructions are classified as None',
    ],
  },
  limitV2: {
    id: 'limitV2',
    label: 'Jupiter Limit V2',
    sub: 'Limit Orders V2',
    programId: 'j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X',
    instructions: {
      InitializeOrder: 'Created',
      PreFlashFillOrder: 'FillInitiated',
      FlashFillOrder: 'FillCompleted',
      CancelOrder: 'Cancelled',
    },
    events: {
      CreateOrderEvent: 'Created',
      CancelOrderEvent: 'Cancelled',
      TradeEvent: 'FillCompleted',
    },
    closedVariants: [],
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
    programId: 'LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF',
    instructions: {
      CreateOrder: 'Created',
      TakeOrder: 'FillCompleted',
      FlashTakeOrderStart: 'FillInitiated',
      FlashTakeOrderEnd: 'FillCompleted',
      CloseOrderAndClaimTip: 'Closed',
    },
    events: {
      OrderDisplayEvent: 'FillCompleted',
      UserSwapBalancesEvent: 'FillCompleted',
    },
    closedVariants: ['Completed', 'Cancelled', 'Expired'],
    notes: [
      'OrderDisplayEvent has no order PDA — requires pre_fetched_order_pdas from instruction accounts',
      'Returns Uncorrelated if PDAs missing',
      'UserSwapBalancesEvent is diagnostic-only (NotRequired correlation → MetadataOnly transition)',
      'Status codes: 0=Open, 1=Filled(Completed), 2=Cancelled, 3=Expired',
      'Admin instructions (InitializeGlobalConfig, etc.) are classified as None',
    ],
  },
};

export const PROTOCOL_LIST: ProtocolConfig[] = Object.values(PROTOCOLS);
