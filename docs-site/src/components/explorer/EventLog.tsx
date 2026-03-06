import type { LogEntry } from '../../hooks/useLifecycleState';

interface Props {
  log: LogEntry[];
}

const STATE_STYLES: Record<string, string> = {
  none: 'bg-dim/10 text-dim',
  active: 'bg-accent/10 text-accent',
  created: 'bg-accent/10 text-accent',
  completed: 'bg-green/10 text-green',
  cancelled: 'bg-red/10 text-red',
  expired: 'bg-amber/10 text-amber',
};

export default function EventLog({ log }: Props) {
  if (log.length === 0) {
    return (
      <div className="p-4 text-sm text-dim">
        Run a scenario or fire an event to see the transition log.
      </div>
    );
  }

  return (
    <div className="max-h-80 overflow-y-auto px-4 py-2">
      {[...log].reverse().map((entry) => (
        <div
          key={entry.step}
          className="flex min-w-0 items-center gap-2 border-b border-border/30 py-2 font-mono text-[11px]"
        >
          <span className="text-dim/60 min-w-[20px] text-right text-[10px]">{entry.step}</span>
          <span
            className={`rounded-md px-1.5 py-px text-[10px] font-medium ${STATE_STYLES[entry.from] ?? STATE_STYLES.none}`}
          >
            {entry.from}
          </span>
          <span className="text-dim/40">→</span>
          <span className="min-w-0 break-words font-medium text-text/80">
            {entry.transition}
          </span>
          <span className="text-dim/40">→</span>
          <span
            className={`rounded-md px-1.5 py-px text-[10px] font-medium ${STATE_STYLES[entry.to] ?? STATE_STYLES.none}`}
          >
            {entry.to}
          </span>
          <span
            className={`text-[9px] rounded-md px-1.5 py-px ml-auto font-medium ${
              entry.decision === 'Apply'
                ? 'bg-green/10 text-green'
                : 'bg-red/10 text-red'
            }`}
          >
            {entry.decision === 'Apply' ? 'Applied' : 'Blocked'}
          </span>
        </div>
      ))}
    </div>
  );
}
