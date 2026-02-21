// Synced from lifecycle-explorer.html PRESETS

import type { ProtocolId } from './protocols';

export interface Preset {
  name: string;
  protocol: ProtocolId;
  events: string[];
}

export const PRESETS: Preset[] = [
  {
    name: 'Happy Path',
    protocol: 'dca',
    events: ['Created', 'FillCompleted', 'FillCompleted', 'Closed:Completed'],
  },
  {
    name: 'User Cancel',
    protocol: 'limitV1',
    events: ['Created', 'FillCompleted', 'Cancelled'],
  },
  {
    name: 'Expired Order',
    protocol: 'dca',
    events: ['Created', 'FillCompleted', 'Closed:Expired'],
  },
  {
    name: 'Terminal Reject',
    protocol: 'limitV2',
    events: ['Created', 'FillCompleted', 'Cancelled', 'FillCompleted'],
  },
  {
    name: 'Kamino Fill',
    protocol: 'kamino',
    events: ['Created', 'FillCompleted', 'FillCompleted', 'Closed:Completed'],
  },
  {
    name: 'Metadata After Close',
    protocol: 'dca',
    events: ['Created', 'Closed:Completed', 'FeeCollected', 'Deposited'],
  },
  {
    name: 'Double Close',
    protocol: 'limitV1',
    events: ['Created', 'Cancelled', 'Cancelled'],
  },
];
