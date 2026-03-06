import { useState } from 'react';
import { PROTOCOL_LIST, type ProtocolId } from '../data/protocols';
import { lookupVariant } from '../engine/lookup';

const EXAMPLE_JSONS: Record<ProtocolId, string> = {
  dca: '{"ClosedEvent": {"dca_key": "3nsT…", "user_closed": false, "unfilled_amount": 0}}',
  limitV1:
    '{"TradeEvent": {"order_key": "HkLZ…", "in_amount": 724773829, "out_amount": 51821329}}',
  limitV2: '{"CreateOrderEvent": {"order_key": "ABC123"}}',
  kamino:
    '{"OrderDisplayEvent": {"remaining_input_amount": 0, "filled_output_amount": 11744711, "status": 1}}',
};

export default function VariantLookup() {
  const [protocol, setProtocol] = useState<ProtocolId>('dca');
  const [json, setJson] = useState(EXAMPLE_JSONS.dca);
  const [result, setResult] = useState<ReturnType<typeof lookupVariant> | null>(null);

  function handleLookup() {
    setResult(lookupVariant(json, protocol));
  }

  function handleProtocolChange(nextProtocol: ProtocolId) {
    setProtocol(nextProtocol);
    setJson(EXAMPLE_JSONS[nextProtocol]);
    setResult(null);
  }

  return (
    <div className="rounded-2xl border border-border bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
        <div>
          <h3 className="text-lg font-semibold text-text">Static Variant Lookup</h3>
          <p className="mt-2 text-sm leading-relaxed text-dim">
            This tool matches the first JSON key against the known instruction and event variants
            for the selected protocol. It does not validate payload shape or correlation data.
          </p>

          <div className="mt-5">
            <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
              Protocol
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              {PROTOCOL_LIST.map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  aria-pressed={protocol === entry.id}
                  onClick={() => handleProtocolChange(entry.id)}
                  className={`focus-ring ui-transition rounded-md border px-3 py-1.5 text-xs ${
                    protocol === entry.id
                      ? 'border-accent/40 bg-accent/[0.08] text-accent'
                      : 'border-border text-dim hover:border-border-active hover:text-text'
                  }`}
                >
                  {entry.label}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div>
          <label htmlFor="variant-lookup-json" className="block">
            <span className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
              Sample JSON
            </span>
            <textarea
              id="variant-lookup-json"
              name="variant_lookup_json"
              autoComplete="off"
              spellCheck={false}
              value={json}
              onChange={(event) => setJson(event.target.value)}
              className="focus-ring ui-transition mt-2 h-40 w-full rounded-xl border border-border bg-bg-elevated p-3 font-mono text-sm text-text"
            />
          </label>

          <div className="mt-3 flex flex-wrap items-center gap-3">
            <button
              type="button"
              onClick={handleLookup}
              className="focus-ring ui-transition rounded-lg px-5 py-2 text-sm font-semibold text-bg accent-gradient-bg hover:shadow-[0_0_20px_rgba(20,241,149,0.2)]"
            >
              Lookup Variant
            </button>
            <span className="text-sm text-dim">
              Best for checking names and canonical mappings before you wire real payload handling.
            </span>
          </div>

          <div className="mt-5 rounded-xl border border-border bg-bg-elevated p-4">
            {result === null ? (
              <p className="text-sm text-dim">
                Paste a sample instruction or event payload and run the lookup to see the canonical
                mapping.
              </p>
            ) : 'error' in result ? (
              <div className="rounded-lg border border-red/20 bg-red/5 p-4 text-sm text-red">
                {result.error}
              </div>
            ) : (
              <div className="space-y-4">
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                  <Row label="Variant" value={result.variantName} />
                  <Row label="Source" value={result.source} />
                  <Row label="EventType" value={result.eventType} color="text-accent" />
                  <Row
                    label="Decision From None"
                    value={result.decisionFromNone}
                    color={result.decisionFromNone === 'Apply' ? 'text-green' : 'text-red'}
                  />
                </div>

                <div className="rounded-lg border border-border/60 bg-panel/70 p-3">
                  <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
                    Canonical Transition
                  </div>
                  <div className="mt-1 font-mono text-sm font-semibold text-cyan">
                    {result.transition}
                  </div>
                </div>

                <div>
                  <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
                    Important Notes
                  </div>
                  <ul className="mt-2 space-y-2">
                    {result.notes.map((note) => (
                      <li key={note} className="flex gap-2 text-sm leading-relaxed text-dim">
                        <span className="mt-0.5 text-accent/70">•</span>
                        <span>{note}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function Row({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div className="rounded-lg border border-border/60 bg-panel/60 p-3">
      <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
        {label}
      </div>
      <div className={`mt-1 break-words font-mono text-sm font-semibold ${color ?? 'text-text'}`}>
        {value}
      </div>
    </div>
  );
}
