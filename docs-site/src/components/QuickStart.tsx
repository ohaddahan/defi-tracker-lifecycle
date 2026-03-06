const QUICKSTART = `cargo add defi-tracker-lifecycle

use defi_tracker_lifecycle::{
    adapter_for, event_type_to_transition, EventPayload, LifecycleEngine, Protocol, ResolveContext,
};

let protocol = Protocol::from_program_id(program_id).ok_or("unknown program")?;
let adapter = adapter_for(protocol);
let ctx = ResolveContext { pre_fetched_order_pdas: None };

let resolved = adapter
    .classify_and_resolve_event(&raw_event, &ctx)
    .ok_or("unknown event variant")?;

let (event_type, _correlation, payload) = resolved?;

let closed_status = match &payload {
    EventPayload::DcaClosed { status } => Some(*status),
    EventPayload::KaminoDisplay { terminal_status, .. } => *terminal_status,
    _ => None,
};

let transition = event_type_to_transition(&event_type, closed_status);
let decision = LifecycleEngine::decide_transition(None, transition);`;

const GUIDES = [
  {
    title: 'What You Pass In',
    body: 'Decoded instructions or events plus any context you already know, such as Kamino pre-fetched order PDAs.',
  },
  {
    title: 'What You Get Back',
    body: 'Canonical EventType values, correlation outcomes, structured payloads, and a lifecycle decision you can apply directly.',
  },
  {
    title: 'What Stays Outside',
    body: 'Storage, RPC, retries, and any product-specific persistence remain in your application layer.',
  },
];

export default function QuickStart() {
  return (
    <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
      <div className="rounded-2xl border border-border bg-panel p-6 shadow-[0_24px_80px_rgba(0,0,0,0.28)]">
        <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
          Minimal Flow
        </div>
        <h3 className="mt-2 text-xl font-semibold text-text">
          Add the crate, classify the payload, map it to a transition, then ask the lifecycle
          engine what to do.
        </h3>
        <p className="mt-3 max-w-2xl text-sm leading-relaxed text-dim">
          The only tricky part is terminal status: DCA can derive it from `ClosedEvent`, and
          Kamino can surface it from `OrderDisplayEvent`. Everything else stays canonical.
        </p>

        <pre className="mt-5 overflow-x-auto rounded-2xl border border-border bg-bg-elevated p-4 text-sm leading-relaxed text-text">
          <code>{QUICKSTART}</code>
        </pre>
      </div>

      <div className="grid grid-cols-1 gap-4">
        {GUIDES.map((guide) => (
          <article
            key={guide.title}
            className="rounded-2xl border border-border bg-panel p-5 shadow-[0_18px_60px_rgba(0,0,0,0.22)]"
          >
            <h3 className="text-base font-semibold text-text">{guide.title}</h3>
            <p className="mt-2 text-sm leading-relaxed text-dim">{guide.body}</p>
          </article>
        ))}
      </div>
    </div>
  );
}
