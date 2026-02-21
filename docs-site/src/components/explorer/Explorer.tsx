import { useCallback, useRef, useState } from 'react';
import { useLifecycleState } from '../../hooks/useLifecycleState';
import type { EventType, ProtocolId } from '../../data/protocols';
import type { Preset } from '../../data/presets';
import ProtocolSelector from './ProtocolSelector';
import EventButtons from './EventButtons';
import PresetBar from './PresetBar';
import StateDiagram from './StateDiagram';
import EventLog from './EventLog';
import CurrentStateBar from './CurrentStateBar';
import SnapshotDelta from './SnapshotDelta';

export default function Explorer() {
  const state = useLifecycleState('dca');
  const [runningPreset, setRunningPreset] = useState<string | null>(null);
  const [lastTransition, setLastTransition] = useState<{
    target: string;
    decision: string;
  } | null>(null);
  const flashTimeoutRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const fireWithAnimation = useCallback(
    (eventType: EventType, closedStatus: string | null) => {
      state.fireEvent(eventType, closedStatus);

      const log = state.log;
      const latest = log.length > 0 ? log[log.length - 1] : null;
      const target = latest ? latest.to : state.currentStatus ?? 'none';
      const decision = latest ? latest.decision : 'Apply';

      setLastTransition({ target, decision });
      if (flashTimeoutRef.current) clearTimeout(flashTimeoutRef.current);
      flashTimeoutRef.current = setTimeout(() => setLastTransition(null), 700);
    },
    [state],
  );

  const handlePreset = useCallback(
    async (preset: Preset) => {
      setRunningPreset(preset.name);
      state.reset();
      state.setProtocol(preset.protocol);

      for (const ev of preset.events) {
        await new Promise((r) => setTimeout(r, 400));
        if (ev.includes(':')) {
          const [type, status] = ev.split(':');
          fireWithAnimation(type as EventType, status);
        } else {
          fireWithAnimation(ev as EventType, null);
        }
      }
      setRunningPreset(null);
    },
    [state, fireWithAnimation],
  );

  const handleProtocolChange = useCallback(
    (p: ProtocolId) => {
      state.setProtocol(p);
    },
    [state],
  );

  return (
    <div className="rounded-xl border border-border bg-panel overflow-hidden glow-border">
      <div className="flex flex-col lg:flex-row">
        {/* Sidebar */}
        <div className="w-full lg:w-80 lg:min-w-[320px] lg:border-r border-border flex flex-col bg-panel">
          <SidebarSection label="Presets">
            <PresetBar onRun={handlePreset} running={runningPreset} />
          </SidebarSection>
          <SidebarSection label="Protocol">
            <ProtocolSelector selected={state.protocol} onSelect={handleProtocolChange} />
          </SidebarSection>
          <SidebarSection label="Fire Event">
            <EventButtons
              protocol={state.protocol}
              isTerminal={state.isTerminal}
              onFire={fireWithAnimation}
            />
          </SidebarSection>
          <SidebarSection label="Snapshot Delta Calculator">
            <SnapshotDelta />
          </SidebarSection>
          <SidebarSection label="Event Log" noBorder>
            <div />
          </SidebarSection>
          <EventLog log={state.log} />
        </div>

        {/* Main area */}
        <div className="flex-1 flex flex-col bg-bg-elevated/50">
          <CurrentStateBar
            currentStatus={state.currentStatus}
            fills={state.fills}
            onReset={state.reset}
          />
          <div className="flex-1 p-6 min-h-[400px]">
            <StateDiagram currentStatus={state.currentStatus} lastTransition={lastTransition} />
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarSection({
  label,
  children,
  noBorder = false,
}: {
  label: string;
  children: React.ReactNode;
  noBorder?: boolean;
}) {
  return (
    <div className={`p-4 ${noBorder ? '' : 'border-b border-border'}`}>
      <div className="text-[10px] text-dim uppercase tracking-widest font-semibold mb-2.5 font-mono">
        {label}
      </div>
      {children}
    </div>
  );
}
