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
      <h4 className="text-xs text-dim uppercase tracking-wider font-semibold mb-2">{title}</h4>
      <div className="rounded-lg border border-border overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-bg">
              <th className="px-3 py-2 text-left text-xs text-dim font-medium">Name</th>
              <th className="px-3 py-2 text-left text-xs text-dim font-medium">EventType</th>
            </tr>
          </thead>
          <tbody>
            {entries.map(([name, type]) => (
              <tr key={name} className="border-t border-border/50">
                <td className="px-3 py-1.5 font-mono text-xs">{name}</td>
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
