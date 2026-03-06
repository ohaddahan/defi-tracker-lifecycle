import { useState } from 'react';
import { normalizeSnapshotToDelta } from '../../engine/lifecycle';

export default function SnapshotDelta() {
  const [stored, setStored] = useState(300);
  const [snapshot, setSnapshot] = useState(450);
  const result = normalizeSnapshotToDelta(stored, snapshot);

  return (
    <div className="space-y-3">
      <p className="text-sm text-dim leading-relaxed">
        Snapshot deltas never go negative. Regressions clamp the delta to `0` and mark the event
        for review.
      </p>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <label className="block">
          <span className="mb-1.5 block text-[10px] text-dim font-mono uppercase tracking-wider">
            Stored Total
          </span>
          <input
            type="number"
            name="stored_total"
            inputMode="numeric"
            autoComplete="off"
            value={stored}
            onChange={(e) => setStored(Number(e.target.value) || 0)}
            className="focus-ring ui-transition w-full rounded-md border border-border bg-bg-elevated px-3 py-2 font-mono text-sm text-text"
          />
        </label>

        <label className="block">
          <span className="mb-1.5 block text-[10px] text-dim font-mono uppercase tracking-wider">
            Snapshot Total
          </span>
          <input
            type="number"
            name="snapshot_total"
            inputMode="numeric"
            autoComplete="off"
            value={snapshot}
            onChange={(e) => setSnapshot(Number(e.target.value) || 0)}
            className="focus-ring ui-transition w-full rounded-md border border-border bg-bg-elevated px-3 py-2 font-mono text-sm text-text"
          />
        </label>
      </div>

      <div className="rounded-xl border border-border bg-bg-elevated px-4 py-3 font-mono text-sm">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-dim">delta</span>
          <span className="text-accent font-semibold">{result.delta}</span>
          <span className="text-dim/30">|</span>
          <span className="text-dim">regression</span>
          <span className={`font-semibold ${result.regression ? 'text-red' : 'text-green'}`}>
            {String(result.regression)}
          </span>
        </div>
      </div>
    </div>
  );
}
