import { useState } from 'react';
import { normalizeSnapshotToDelta } from '../../engine/lifecycle';

export default function SnapshotDelta() {
  const [stored, setStored] = useState(300);
  const [snapshot, setSnapshot] = useState(450);
  const result = normalizeSnapshotToDelta(stored, snapshot);

  return (
    <div>
      <div className="flex items-center gap-2 mb-2">
        <span className="text-xs text-dim min-w-[90px]">Stored total</span>
        <input
          type="number"
          value={stored}
          onChange={(e) => setStored(Number(e.target.value) || 0)}
          className="w-24 rounded-md border border-border bg-bg px-2 py-1.5 font-mono text-sm text-text"
        />
      </div>
      <div className="flex items-center gap-2 mb-3">
        <span className="text-xs text-dim min-w-[90px]">Snapshot total</span>
        <input
          type="number"
          value={snapshot}
          onChange={(e) => setSnapshot(Number(e.target.value) || 0)}
          className="w-24 rounded-md border border-border bg-bg px-2 py-1.5 font-mono text-sm text-text"
        />
      </div>
      <div className="rounded-md border border-border bg-bg px-3 py-2 font-mono text-sm">
        delta: <span className="text-green">{result.delta}</span>
        <span className="text-dim"> · </span>
        regression:{' '}
        <span className={result.regression ? 'text-red' : 'text-green'}>
          {String(result.regression)}
        </span>
      </div>
    </div>
  );
}
