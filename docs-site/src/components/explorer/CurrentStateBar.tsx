interface Props {
  currentStatus: string | null;
  fills: number;
  onReset: () => void;
}

const STATE_STYLES: Record<string, string> = {
  none: 'bg-dim/20 text-dim',
  active: 'bg-accent/15 text-accent',
  created: 'bg-accent/15 text-accent',
  completed: 'bg-green/15 text-green',
  cancelled: 'bg-red/15 text-red',
  expired: 'bg-amber/15 text-amber',
};

export default function CurrentStateBar({ currentStatus, fills, onReset }: Props) {
  const state = currentStatus ?? 'none';
  const style = STATE_STYLES[state] ?? STATE_STYLES.none;
  const label = currentStatus
    ? currentStatus.charAt(0).toUpperCase() + currentStatus.slice(1)
    : 'None (new order)';

  return (
    <div className="flex items-center gap-3 px-4 py-3 bg-panel border-b border-border">
      <span className="text-[11px] text-dim uppercase tracking-wider">Current State</span>
      <span className={`inline-block rounded px-2 py-0.5 text-xs font-semibold ${style}`}>
        {label}
      </span>
      <span className="text-[11px] text-dim uppercase tracking-wider ml-4">Fills</span>
      <span className="font-mono text-sm">{fills}</span>
      <button
        onClick={onReset}
        className="ml-auto rounded-md border border-border px-3 py-1 text-[11px] text-dim transition-all hover:text-red hover:border-red cursor-pointer"
      >
        Reset
      </button>
    </div>
  );
}
