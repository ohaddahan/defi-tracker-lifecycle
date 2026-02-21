import { useState } from 'react';
import { PROTOCOL_LIST, type ProtocolId } from '../data/protocols';
import { classifyJson } from '../engine/classifier';

const EXAMPLE_JSONS: Record<ProtocolId, string> = {
  dca: '{"FilledEvent": {"dca_key": "3nsT...", "in_amount": 21041666667, "out_amount": 569529644}}',
  limitV1: '{"TradeEvent": {"order_key": "HkLZ...", "in_amount": 724773829, "out_amount": 51821329, "remaining_in_amount": 89147181051, "remaining_out_amount": 6374023074}}',
  limitV2: '{"CreateOrderEvent": {"order_key": "ABC123"}}',
  kamino: '{"OrderDisplayEvent": {"remaining_input_amount": 0, "filled_output_amount": 11744711, "number_of_fills": 1, "status": 1}}',
};

export default function JsonClassifier() {
  const [protocol, setProtocol] = useState<ProtocolId>('dca');
  const [json, setJson] = useState(EXAMPLE_JSONS.dca);
  const [result, setResult] = useState<ReturnType<typeof classifyJson> | null>(null);

  const handleClassify = () => {
    setResult(classifyJson(json, protocol));
  };

  const handleProtocolChange = (p: ProtocolId) => {
    setProtocol(p);
    setJson(EXAMPLE_JSONS[p]);
    setResult(null);
  };

  return (
    <div className="rounded-xl border border-border bg-panel p-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Input */}
        <div>
          <div className="text-[11px] text-dim uppercase tracking-wider font-semibold mb-2">
            Protocol
          </div>
          <div className="flex flex-wrap gap-2 mb-4">
            {PROTOCOL_LIST.map((p) => (
              <button
                key={p.id}
                onClick={() => handleProtocolChange(p.id)}
                className={`rounded-md border px-3 py-1 text-xs cursor-pointer transition-all ${
                  protocol === p.id
                    ? 'border-accent bg-accent/15 text-accent'
                    : 'border-border text-text hover:border-accent'
                }`}
              >
                {p.label}
              </button>
            ))}
          </div>

          <div className="text-[11px] text-dim uppercase tracking-wider font-semibold mb-2">
            Raw Event JSON
          </div>
          <textarea
            value={json}
            onChange={(e) => setJson(e.target.value)}
            className="w-full h-40 rounded-lg border border-border bg-bg p-3 font-mono text-sm text-text resize-none focus:outline-none focus:border-accent"
            spellCheck={false}
          />
          <button
            onClick={handleClassify}
            className="mt-3 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-white transition-all hover:bg-accent/80 cursor-pointer"
          >
            Classify
          </button>
        </div>

        {/* Output */}
        <div>
          <div className="text-[11px] text-dim uppercase tracking-wider font-semibold mb-2">
            Classification Result
          </div>
          {result === null ? (
            <div className="rounded-lg border border-border bg-bg p-4 text-dim text-sm italic">
              Paste JSON and click Classify to see the result...
            </div>
          ) : 'error' in result ? (
            <div className="rounded-lg border border-red/30 bg-red/5 p-4 text-red text-sm">
              {result.error}
            </div>
          ) : (
            <div className="rounded-lg border border-border bg-bg p-4 space-y-3">
              <Row label="Variant" value={result.variantName} />
              <Row label="Source" value={result.source} />
              <Row label="EventType" value={result.eventType} color="text-accent" />
              <Row label="Transition" value={result.transition} color="text-cyan" />
              <Row
                label="Decision (from None)"
                value={result.decision}
                color={result.decision === 'Apply' ? 'text-green' : 'text-red'}
              />
              <div className="border-t border-border pt-3 text-dim text-xs">
                Pipeline: <span className="text-text">{result.variantName}</span> →{' '}
                <span className="text-accent">{result.eventType}</span> →{' '}
                <span className="text-cyan">{result.transition}</span> →{' '}
                <span className={result.decision === 'Apply' ? 'text-green' : 'text-red'}>
                  {result.decision}
                </span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function Row({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-dim text-xs">{label}</span>
      <span className={`font-mono text-sm font-semibold ${color ?? 'text-text'}`}>{value}</span>
    </div>
  );
}
