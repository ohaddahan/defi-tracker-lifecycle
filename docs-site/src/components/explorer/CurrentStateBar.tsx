interface Props {
  currentStatus: string | null;
  fills: number;
  onReset: () => void;
}

const STATE_STYLES: Record<string, string> = {
  none: 'bg-dim/10 text-dim border-dim/20',
  active: 'bg-accent/10 text-accent border-accent/20',
  created: 'bg-accent/10 text-accent border-accent/20',
  completed: 'bg-green/10 text-green border-green/20',
  cancelled: 'bg-red/10 text-red border-red/20',
  expired: 'bg-amber/10 text-amber border-amber/20',
};

export default function CurrentStateBar({ currentStatus, fills, onReset }: Props) {
  const state = currentStatus ?? 'none';
  const style = STATE_STYLES[state] ?? STATE_STYLES.none;
  const terminal = ['completed', 'cancelled', 'expired'].includes(state);
  const label = currentStatus
    ? currentStatus.charAt(0).toUpperCase() + currentStatus.slice(1)
    : 'None (new order)';

  return (
    <div className="flex flex-wrap items-center gap-3 border-b border-border bg-panel px-5 py-4">
      <span className="text-[10px] text-dim uppercase tracking-widest font-mono">State</span>
      <span
        className={`inline-block rounded-md border px-2.5 py-0.5 text-xs font-semibold ${style}`}
      >
        {label}
      </span>
      <span className="text-[10px] text-dim uppercase tracking-widest font-mono">
        Decision Rule
      </span>
      <span className="text-sm text-dim">
        {terminal ? 'Only MetadataOnly is allowed now.' : 'Any transition can still apply.'}
      </span>
      <span className="text-[10px] text-dim uppercase tracking-widest font-mono sm:ml-auto">
        Fills
      </span>
      <span className="font-mono text-sm text-accent">{fills}</span>
      <button
        type="button"
        onClick={onReset}
        className="focus-ring ui-transition rounded-md border border-border px-3 py-1 text-[10px] text-dim font-mono tracking-wider uppercase hover:border-red/40 hover:bg-red/5 hover:text-red"
      >
        Reset
      </button>
    </div>
  );
}
