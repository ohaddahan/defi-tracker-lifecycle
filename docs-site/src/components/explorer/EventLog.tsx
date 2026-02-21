import type { LogEntry } from '../../hooks/useLifecycleState';

interface Props {
  log: LogEntry[];
}

const STATE_STYLES: Record<string, string> = {
  none: 'bg-dim/20 text-dim',
  active: 'bg-accent/15 text-accent',
  created: 'bg-accent/15 text-accent',
  completed: 'bg-green/15 text-green',
  cancelled: 'bg-red/15 text-red',
  expired: 'bg-amber/15 text-amber',
};

export default function EventLog({ log }: Props) {
  if (log.length === 0) {
    return <div className="text-dim text-xs p-4 italic">Fire events to see the log...</div>;
  }

  return (
    <div className="overflow-y-auto max-h-64 px-4 py-2">
      {[...log].reverse().map((entry) => (
        <div
          key={entry.step}
          className="flex items-center gap-2 py-1 border-b border-border/50 font-mono text-xs"
        >
          <span className="text-dim min-w-[20px] text-right">{entry.step}</span>
          <span
            className={`rounded px-1.5 py-px text-[10px] font-semibold ${STATE_STYLES[entry.from] ?? STATE_STYLES.none}`}
          >
            {entry.from}
          </span>
          <span className="text-dim">→</span>
          <span className="font-semibold">{entry.transition}</span>
          <span className="text-dim">→</span>
          <span
            className={`rounded px-1.5 py-px text-[10px] font-semibold ${STATE_STYLES[entry.to] ?? STATE_STYLES.none}`}
          >
            {entry.to}
          </span>
          <span
            className={`text-[10px] rounded px-1.5 py-px ${
              entry.decision === 'Apply'
                ? 'bg-green/15 text-green'
                : 'bg-red/15 text-red'
            }`}
          >
            {entry.decision === 'Apply' ? 'Applied' : 'Blocked'}
          </span>
        </div>
      ))}
    </div>
  );
}
