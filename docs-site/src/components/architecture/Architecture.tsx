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
      {/* Crate Structure */}
      <div className="rounded-xl border border-border bg-panel p-6">
        <h3 className="text-sm font-semibold text-dim uppercase tracking-wider mb-4">
          Crate Structure
        </h3>
        <pre className="rounded-lg bg-bg border border-border p-4 font-mono text-xs text-dim overflow-x-auto">
          {CRATE_TREE}
        </pre>
      </div>

      {/* Test Pyramid */}
      <div className="rounded-xl border border-border bg-panel p-6">
        <h3 className="text-sm font-semibold text-dim uppercase tracking-wider mb-6">
          6-Layer Test Pyramid (137 tests)
        </h3>
        <TestPyramid />
      </div>

      {/* EventType → Transition Table */}
      <div className="rounded-xl border border-border bg-panel p-6">
        <h3 className="text-sm font-semibold text-dim uppercase tracking-wider mb-4">
          EventType → LifecycleTransition Mapping
        </h3>
        <p className="text-xs text-dim mb-4">
          This mapping is defined by the consumer (defi-tracker), not by this crate. Shown here for reference.
        </p>
        <div className="rounded-lg border border-border overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-bg">
                <th className="px-3 py-2 text-left text-xs text-dim font-medium">EventType</th>
                <th className="px-3 py-2 text-left text-xs text-dim font-medium">Transition</th>
                <th className="px-3 py-2 text-left text-xs text-dim font-medium">Description</th>
              </tr>
            </thead>
            <tbody>
              {EVENT_TYPE_MAPPING.map(([et, tr, desc]) => (
                <tr key={et} className="border-t border-border/50">
                  <td className="px-3 py-1.5 font-mono text-xs text-accent">{et}</td>
                  <td className="px-3 py-1.5 font-mono text-xs text-cyan">{tr}</td>
                  <td className="px-3 py-1.5 text-xs text-dim">{desc}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
