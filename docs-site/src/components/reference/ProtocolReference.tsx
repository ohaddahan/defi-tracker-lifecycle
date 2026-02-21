import { useState } from 'react';
import { PROTOCOL_LIST, type EventType, type ProtocolId } from '../../data/protocols';
import MappingTable from './MappingTable';

export default function ProtocolReference() {
  const [active, setActive] = useState<ProtocolId>('dca');
  const protocol = PROTOCOL_LIST.find((p) => p.id === active)!;

  return (
    <div className="rounded-xl border border-border bg-panel overflow-hidden glow-border">
      <div className="flex border-b border-border">
        {PROTOCOL_LIST.map((p) => (
          <button
            key={p.id}
            onClick={() => setActive(p.id)}
            className={`flex-1 px-4 py-3 text-sm font-medium transition-all cursor-pointer relative ${
              active === p.id
                ? 'text-accent bg-accent/[0.04]'
                : 'text-dim hover:text-text hover:bg-panel-hover'
            }`}
          >
            {p.label}
            {active === p.id && (
              <span className="absolute bottom-0 left-0 right-0 h-px accent-gradient-bg" />
            )}
          </button>
        ))}
      </div>

      <div className="p-6">
        <div className="flex items-center gap-3 mb-6">
          <h3 className="text-lg font-semibold">{protocol.label}</h3>
          <span className="rounded-md bg-bg-elevated border border-border px-2 py-0.5 font-mono text-[10px] text-dim">
            {protocol.programId}
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <MappingTable
            title="Instructions → EventType"
            entries={Object.entries(protocol.instructions) as [string, EventType][]}
          />
          <MappingTable
            title="Events → EventType"
            entries={Object.entries(protocol.events) as [string, EventType][]}
          />
        </div>

        {protocol.notes.length > 0 && (
          <div className="mt-6">
            <h4 className="text-[10px] text-dim uppercase tracking-widest font-mono font-semibold mb-3">
              Protocol Notes
            </h4>
            <ul className="space-y-1.5">
              {protocol.notes.map((note, i) => (
                <li key={i} className="text-sm text-dim/80 flex gap-2">
                  <span className="text-accent/60 mt-0.5">›</span>
                  {note}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}
