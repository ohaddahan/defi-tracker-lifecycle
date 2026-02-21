import type { EventType } from '../../data/protocols';

interface Props {
  title: string;
  entries: [string, EventType][];
}

const TYPE_COLORS: Record<EventType, string> = {
  Created: 'text-accent',
  FillInitiated: 'text-cyan',
  FillCompleted: 'text-cyan',
  Cancelled: 'text-red',
  Expired: 'text-amber',
  Closed: 'text-purple',
  FeeCollected: 'text-dim',
  Withdrawn: 'text-dim',
  Deposited: 'text-dim',
};

export default function MappingTable({ title, entries }: Props) {
  return (
    <div className="mb-4">
      <h4 className="text-[10px] text-dim uppercase tracking-widest font-mono font-semibold mb-2.5">{title}</h4>
      <div className="rounded-lg border border-border overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-bg-elevated">
              <th className="px-3 py-2 text-left text-[10px] text-dim font-mono font-medium uppercase tracking-wider">Name</th>
              <th className="px-3 py-2 text-left text-[10px] text-dim font-mono font-medium uppercase tracking-wider">EventType</th>
            </tr>
          </thead>
          <tbody>
            {entries.map(([name, type]) => (
              <tr key={name} className="border-t border-border/30 hover:bg-panel-hover/50 transition-colors">
                <td className="px-3 py-1.5 font-mono text-xs text-text/80">{name}</td>
                <td className={`px-3 py-1.5 font-mono text-xs font-semibold ${TYPE_COLORS[type]}`}>
                  {type}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
