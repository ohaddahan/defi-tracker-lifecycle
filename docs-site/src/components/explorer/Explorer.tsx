import { useState } from 'react';
import type { Preset } from '../../data/presets';
import type { EventType, ProtocolId } from '../../data/protocols';
import { useLifecycleState } from '../../hooks/useLifecycleState';
import CurrentStateBar from './CurrentStateBar';
import EventButtons from './EventButtons';
import EventLog from './EventLog';
import PresetBar from './PresetBar';
import ProtocolSelector from './ProtocolSelector';
import SnapshotDelta from './SnapshotDelta';
import StateDiagram from './StateDiagram';

export default function Explorer() {
  const state = useLifecycleState('dca');
  const [runningPreset, setRunningPreset] = useState<string | null>(null);

  async function handlePreset(preset: Preset) {
    setRunningPreset(preset.name);
    state.setProtocol(preset.protocol);

    for (const event of preset.events) {
      await new Promise((resolve) => setTimeout(resolve, 350));

      if (event.includes(':')) {
        const [type, status] = event.split(':');
        state.fireEvent(type as EventType, status);
      } else {
        state.fireEvent(event as EventType, null);
      }
    }

    setRunningPreset(null);
  }

  function handleProtocolChange(protocol: ProtocolId) {
    state.setProtocol(protocol);
  }

  return (
    <div className="overflow-hidden rounded-2xl border border-border bg-panel shadow-[0_24px_80px_rgba(0,0,0,0.35)]">
      <div className="grid grid-cols-1 xl:grid-cols-[360px_minmax(0,1fr)]">
        <aside className="border-b border-border bg-panel xl:border-b-0 xl:border-r">
          <SidebarSection
            label="Scenarios"
            description="Replay common flows to see which transitions apply and which ones are blocked."
          >
            <PresetBar onRun={handlePreset} running={runningPreset} />
          </SidebarSection>

          <SidebarSection
            label="Protocol"
            description="Switching protocols resets the current run so the diagram and log stay honest."
          >
            <ProtocolSelector
              selected={state.protocol}
              onSelect={handleProtocolChange}
            />
          </SidebarSection>

          <SidebarSection
            label="Fire Canonical Events"
            description="Use the canonical EventType buttons here. Raw variant names stay in the protocol reference."
          >
            <EventButtons
              protocol={state.protocol}
              isTerminal={state.isTerminal}
              onFire={state.fireEvent}
            />
          </SidebarSection>

          <SidebarSection
            label="Snapshot Delta"
            description="Useful when you consume cumulative balance snapshots and need the crate’s non-negative delta rule."
            noBorder
          >
            <SnapshotDelta />
          </SidebarSection>
        </aside>

        <section className="min-w-0 bg-bg-elevated/40">
          <CurrentStateBar
            currentStatus={state.currentStatus}
            fills={state.fills}
            onReset={state.reset}
          />

          <div className="grid grid-cols-1 gap-0 lg:grid-cols-[minmax(0,1fr)_320px]">
            <div className="min-w-0 border-b border-border lg:border-b-0 lg:border-r">
              <div className="border-b border-border px-6 py-4">
                <h3 className="text-lg font-semibold text-text">State Diagram</h3>
                <p className="mt-1 text-sm text-dim">
                  The diagram reflects the exact decision log below. Nothing here is inferred from
                  stale local state anymore.
                </p>
              </div>
              <div className="min-h-[440px] p-4 sm:p-6">
                <StateDiagram
                  currentStatus={state.currentStatus}
                  lastTransition={state.lastTransition}
                />
              </div>
            </div>

            <div className="min-w-0">
              <div className="border-b border-border px-6 py-4">
                <h3 className="text-lg font-semibold text-text">Decision Log</h3>
                <p className="mt-1 text-sm text-dim">
                  Each row shows the before state, mapped transition, after state, and whether the
                  lifecycle engine applied or ignored it.
                </p>
              </div>
              <EventLog log={state.log} />
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

function SidebarSection({
  label,
  description,
  children,
  noBorder = false,
}: {
  label: string;
  description: string;
  children: React.ReactNode;
  noBorder?: boolean;
}) {
  return (
    <section className={`p-5 ${noBorder ? '' : 'border-b border-border'}`}>
      <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
        {label}
      </div>
      <p className="mt-2 text-sm leading-relaxed text-dim">{description}</p>
      <div className="mt-4">{children}</div>
    </section>
  );
}
