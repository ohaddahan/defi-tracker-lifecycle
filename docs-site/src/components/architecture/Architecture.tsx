import TestPyramid from './TestPyramid';

const FLOW = [
  {
    title: '1. Match the protocol',
    body: 'Resolve the program ID to one of the supported protocols, then grab the corresponding static adapter.',
  },
  {
    title: '2. Normalize the payload',
    body: 'Classify raw variants into canonical EventType values, plus correlation outcomes and structured payload data where available.',
  },
  {
    title: '3. Enforce the lifecycle',
    body: 'Map EventType to a transition and let LifecycleEngine decide whether the change is valid for the current terminal state.',
  },
];

const GUARANTEES = [
  {
    title: 'No hidden IO',
    body: 'The crate stays pure logic. RPC, persistence, retries, and indexing remain your application’s responsibility.',
  },
  {
    title: 'Terminal safety',
    body: 'Once an order is terminal, only MetadataOnly transitions survive. Everything else is ignored explicitly.',
  },
  {
    title: 'Protocol quirks stay local',
    body: 'DCA close derivation, Kamino PDA requirements, and v1/v2 naming differences are handled in protocol-specific adapters.',
  },
];

export default function Architecture() {
  return (
    <div className="grid grid-cols-1 gap-6">
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <section className="rounded-2xl border border-border bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
          <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
            How It Works
          </div>
          <div className="mt-5 grid grid-cols-1 gap-4">
            {FLOW.map((step) => (
              <article
                key={step.title}
                className="rounded-xl border border-border bg-bg-elevated p-4"
              >
                <h3 className="text-base font-semibold text-text">{step.title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-dim">{step.body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
          <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
            Reliability Guarantees
          </div>
          <div className="mt-5 grid grid-cols-1 gap-4">
            {GUARANTEES.map((item) => (
              <article
                key={item.title}
                className="rounded-xl border border-border bg-bg-elevated p-4"
              >
                <h3 className="text-base font-semibold text-text">{item.title}</h3>
                <p className="mt-2 text-sm leading-relaxed text-dim">{item.body}</p>
              </article>
            ))}
          </div>
        </section>
      </div>

      <section className="rounded-2xl border border-border bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
        <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
          Test Layers
        </div>
        <h3 className="mt-2 text-xl font-semibold text-text">
          The crate is checked from mirror-enum drift all the way to end-to-end lifecycle flows.
        </h3>
        <p className="mt-3 max-w-3xl text-sm leading-relaxed text-dim">
          The important part is not the raw test count. It is the spread: compile-time coverage,
          serde alignment, real fixtures, and lifecycle sequences all protect different failure
          modes.
        </p>
        <div className="mt-6">
          <TestPyramid />
        </div>
      </section>
    </div>
  );
}
