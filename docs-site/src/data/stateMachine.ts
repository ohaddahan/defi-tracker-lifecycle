// Synced from lifecycle-explorer.html NODES/EDGES

export interface NodeDef {
  id: string;
  label: string;
  sub: string;
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  stroke: string;
}

export interface EdgeDef {
  from: string;
  to: string;
  label: string;
  color: string;
  self?: boolean;
  dy?: number;
}

export const NODES: NodeDef[] = [
  { id: 'none', label: 'None', sub: 'new order', x: 100, y: 250, w: 110, h: 50, color: '#374151', stroke: '#6b7280' },
  { id: 'active', label: 'Active', sub: 'in progress', x: 310, y: 250, w: 120, h: 50, color: '#1e3a5f', stroke: '#3b82f6' },
  { id: 'completed', label: 'Completed', sub: 'terminal', x: 570, y: 120, w: 140, h: 50, color: '#14532d', stroke: '#22c55e' },
  { id: 'cancelled', label: 'Cancelled', sub: 'terminal', x: 570, y: 250, w: 140, h: 50, color: '#450a0a', stroke: '#ef4444' },
  { id: 'expired', label: 'Expired', sub: 'terminal', x: 570, y: 380, w: 140, h: 50, color: '#451a03', stroke: '#f59e0b' },
];

export const EDGES: EdgeDef[] = [
  { from: 'none', to: 'active', label: 'Create', color: '#3b82f6' },
  { from: 'active', to: 'active', label: 'FillDelta', color: '#06b6d4', self: true, dy: -40 },
  { from: 'active', to: 'active', label: 'MetadataOnly', color: '#6b7280', self: true, dy: 40 },
  { from: 'active', to: 'completed', label: 'Close(Completed)', color: '#22c55e' },
  { from: 'active', to: 'cancelled', label: 'Close(Cancelled)', color: '#ef4444' },
  { from: 'active', to: 'expired', label: 'Close(Expired)', color: '#f59e0b' },
  { from: 'completed', to: 'completed', label: 'MetadataOnly', color: '#6b7280', self: true, dy: -35 },
  { from: 'cancelled', to: 'cancelled', label: 'MetadataOnly', color: '#6b7280', self: true, dy: -35 },
  { from: 'expired', to: 'expired', label: 'MetadataOnly', color: '#6b7280', self: true, dy: -35 },
];

export const ARROW_COLORS: Record<string, string> = {
  blue: '#3b82f6',
  green: '#22c55e',
  red: '#ef4444',
  amber: '#f59e0b',
  cyan: '#06b6d4',
  gray: '#6b7280',
};

export function colorToName(hex: string): string {
  const entry = Object.entries(ARROW_COLORS).find(([, v]) => v === hex);
  return entry ? entry[0] : 'gray';
}
