import TestPyramid from './TestPyramid';

const CRATE_TREE = `src/
  lib.rs                    # Public API, cfg_attr deny
  error.rs                  # Error enum (Parse, Protocol, Json)
  types.rs                  # RawInstruction, RawEvent, ResolveContext
  lifecycle/
    mod.rs                  # LifecycleEngine, TerminalStatus, SnapshotDelta
    adapters.rs             # ProtocolAdapter trait, adapter_for()
  protocols/
    mod.rs                  # Protocol/EventType enums, shared helpers
    dca.rs                  # Jupiter DCA adapter
    limit_v1.rs             # Jupiter Limit V1 adapter
    limit_v2.rs             # Jupiter Limit V2 adapter
    kamino.rs               # Kamino adapter`;

const EVENT_TYPE_MAPPING = [
  ['Created', 'Create', 'Initializes order'],
  ['FillInitiated', 'FillDelta', 'Flash-fill started'],
  ['FillCompleted', 'FillDelta', 'Partial/full fill'],
  ['Cancelled', 'Close(Cancelled)', 'User cancelled'],
  ['Expired', 'Close(Expired)', 'Time expired'],
  ['Closed', 'Close(?)', 'Protocol-level close'],
  ['FeeCollected', 'MetadataOnly', 'Fee event'],
  ['Withdrawn', 'MetadataOnly', 'Withdrawal event'],
  ['Deposited', 'MetadataOnly', 'Deposit event'],
];

export default function Architecture() {
  return (
    <div className="space-y-8">
      <div className="rounded-xl border border-border bg-panel p-6 glow-border">
        <h3 className="text-[10px] text-dim uppercase tracking-widest font-mono font-semibold mb-4">
          Crate Structure
        </h3>
        <pre className="rounded-lg bg-bg-elevated border border-border p-4 font-mono text-xs text-dim/80 overflow-x-auto leading-relaxed">
          {CRATE_TREE}
        </pre>
      </div>

      <div className="rounded-xl border border-border bg-panel p-6 glow-border">
        <h3 className="text-[10px] text-dim uppercase tracking-widest font-mono font-semibold mb-6">
          6-Layer Test Pyramid (137 tests)
        </h3>
        <TestPyramid />
      </div>

      <div className="rounded-xl border border-border bg-panel p-6 glow-border">
        <h3 className="text-[10px] text-dim uppercase tracking-widest font-mono font-semibold mb-2">
          EventType → LifecycleTransition Mapping
        </h3>
        <p className="text-[11px] text-dim/60 mb-4">
          Defined by consumer (defi-tracker), not by this crate. Shown for reference.
        </p>
        <div className="rounded-lg border border-border overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-bg-elevated">
                <th className="px-3 py-2 text-left text-[10px] text-dim font-mono font-medium uppercase tracking-wider">EventType</th>
                <th className="px-3 py-2 text-left text-[10px] text-dim font-mono font-medium uppercase tracking-wider">Transition</th>
                <th className="px-3 py-2 text-left text-[10px] text-dim font-mono font-medium uppercase tracking-wider">Description</th>
              </tr>
            </thead>
            <tbody>
              {EVENT_TYPE_MAPPING.map(([et, tr, desc]) => (
                <tr key={et} className="border-t border-border/30 hover:bg-panel-hover/50 transition-colors">
                  <td className="px-3 py-1.5 font-mono text-xs text-accent">{et}</td>
                  <td className="px-3 py-1.5 font-mono text-xs text-cyan">{tr}</td>
                  <td className="px-3 py-1.5 text-xs text-dim/70">{desc}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
