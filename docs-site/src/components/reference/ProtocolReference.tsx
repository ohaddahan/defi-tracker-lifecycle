import { useState } from 'react';
import { PROTOCOL_LIST, type EventType, type ProtocolId } from '../../data/protocols';
import MappingTable from './MappingTable';

export default function ProtocolReference() {
  const [active, setActive] = useState<ProtocolId>('dca');
  const protocol = PROTOCOL_LIST.find((entry) => entry.id === active);

  if (!protocol) {
    return null;
  }

  return (
    <div className="rounded-2xl border border-border bg-panel shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
      <div className="border-b border-border p-4">
        <div className="flex flex-wrap gap-2">
          {PROTOCOL_LIST.map((entry) => {
            const isActive = entry.id === active;
            return (
              <button
                key={entry.id}
                type="button"
                aria-pressed={isActive}
                onClick={() => setActive(entry.id)}
                className={`focus-ring ui-transition rounded-lg border px-4 py-2 text-sm font-medium ${
                  isActive
                    ? 'border-accent/40 bg-accent/[0.08] text-accent'
                    : 'border-border text-dim hover:border-border-active hover:bg-panel-hover hover:text-text'
                }`}
              >
                {entry.label}
              </button>
            );
          })}
        </div>
      </div>

      <div className="p-6">
        <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_320px]">
          <div className="min-w-0">
            <div className="flex flex-wrap items-start gap-3">
              <div>
                <h3 className="text-xl font-semibold text-text">{protocol.label}</h3>
                <p className="mt-1 text-sm text-dim">{protocol.sub}</p>
              </div>
              <span className="rounded-full border border-border bg-bg-elevated px-3 py-1 text-[10px] font-mono uppercase tracking-widest text-dim">
                {protocol.closedVariants.length > 0
                  ? `${protocol.closedVariants.length} closed variants`
                  : 'No close variants'}
              </span>
            </div>

            <div className="mt-4 rounded-xl border border-border bg-bg-elevated p-4">
              <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
                Program ID
              </div>
              <div className="mt-2 break-all font-mono text-sm text-text">{protocol.programId}</div>
            </div>

            <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
              <MappingTable
                title="Instruction Variants"
                entries={Object.entries(protocol.instructions) as [string, EventType][]}
              />
              <MappingTable
                title="Event Variants"
                entries={Object.entries(protocol.events) as [string, EventType][]}
              />
            </div>
          </div>

          <aside className="rounded-2xl border border-border bg-bg-elevated p-5">
            <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
              Integration Notes
            </div>
            <ul className="mt-4 space-y-3">
              {protocol.notes.map((note) => (
                <li key={note} className="flex gap-2 text-sm leading-relaxed text-dim">
                  <span className="mt-0.5 text-accent/70">•</span>
                  <span>{note}</span>
                </li>
              ))}
            </ul>
          </aside>
        </div>
      </div>
    </div>
  );
}
