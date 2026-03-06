import {
  EVENT_TYPE_ORDER,
  PROTOCOLS,
  type EventType,
  type ProtocolId,
} from '../../data/protocols';

interface Props {
  protocol: ProtocolId;
  isTerminal: boolean;
  onFire: (eventType: EventType, closedStatus: string | null) => void;
}

const TRANSITION_LABELS: Record<EventType, string> = {
  Created: 'Create',
  FillInitiated: 'FillDelta',
  FillCompleted: 'FillDelta',
  Cancelled: 'Close(Cancelled)',
  Expired: 'Close(Expired)',
  Closed: 'Close(…)/MetadataOnly',
  FeeCollected: 'MetadataOnly',
  Withdrawn: 'MetadataOnly',
  Deposited: 'MetadataOnly',
};

const METADATA_TYPES = new Set<EventType>(['FeeCollected', 'Withdrawn', 'Deposited']);

function buildEventGroups(protocol: ProtocolId) {
  const config = PROTOCOLS[protocol];
  const examplesByType = new Map<EventType, string[]>();

  for (const [name, type] of [
    ...Object.entries(config.instructions),
    ...Object.entries(config.events),
  ] as [string, EventType][]) {
    const current = examplesByType.get(type) ?? [];
    if (!current.includes(name)) {
      current.push(name);
    }
    examplesByType.set(type, current);
  }

  const lifecycle = EVENT_TYPE_ORDER.filter(
    (type) =>
      type !== 'Closed' && !METADATA_TYPES.has(type) && examplesByType.has(type),
  ).map((type) => ({ type, examples: examplesByType.get(type) ?? [] }));

  const metadata = EVENT_TYPE_ORDER.filter(
    (type) => METADATA_TYPES.has(type) && examplesByType.has(type),
  ).map((type) => ({ type, examples: examplesByType.get(type) ?? [] }));

  return {
    lifecycle,
    metadata,
    closeExamples: examplesByType.get('Closed') ?? [],
    closedVariants: config.closedVariants,
  };
}

function EventGroup({
  title,
  entries,
  isTerminal,
  onFire,
}: {
  title: string;
  entries: { type: EventType; examples: string[] }[];
  isTerminal: boolean;
  onFire: (eventType: EventType, closedStatus: string | null) => void;
}) {
  if (entries.length === 0) {
    return null;
  }

  return (
    <div className="mt-4">
      <div className="text-[10px] text-dim/70 mb-2 font-mono uppercase tracking-wider">
        {title}
      </div>
      <div className="grid grid-cols-1 gap-2">
        {entries.map(({ type, examples }) => {
          const wouldBlock = isTerminal && !METADATA_TYPES.has(type);

          return (
            <button
              key={type}
              type="button"
              onClick={() => onFire(type, null)}
              className={`focus-ring ui-transition rounded-xl border px-3 py-2.5 text-left ${
                wouldBlock
                  ? 'border-red/30 bg-red/5 text-red'
                  : 'border-border bg-bg-elevated/40 text-text hover:border-accent/40 hover:bg-panel-hover'
              }`}
            >
              <span className="flex items-center justify-between gap-3">
                <span className="text-sm font-semibold">{type}</span>
                <span className="font-mono text-[10px] text-dim/80">
                  {TRANSITION_LABELS[type]}
                </span>
              </span>
              <span className="mt-1 block text-[11px] text-dim break-words">
                {examples.join(' / ')}
              </span>
              {wouldBlock ? (
                <span className="mt-2 block text-[10px] text-red/80">
                  Still clickable: the explorer will log the terminal-state rejection.
                </span>
              ) : null}
            </button>
          );
        })}
      </div>
    </div>
  );
}

export default function EventButtons({ protocol, isTerminal, onFire }: Props) {
  const { lifecycle, metadata, closeExamples, closedVariants } = buildEventGroups(protocol);

  return (
    <div>
      <p className="text-sm text-dim leading-relaxed">
        Fire canonical event types here. Raw instruction and event names stay in the protocol
        reference below.
      </p>

      <EventGroup
        title="Lifecycle Events"
        entries={lifecycle}
        isTerminal={isTerminal}
        onFire={onFire}
      />

      {closedVariants.length > 0 ? (
        <div className="mt-4">
          <div className="text-[10px] text-dim/70 mb-2 font-mono uppercase tracking-wider">
            Close Terminal Status
          </div>
          <div className="grid grid-cols-1 gap-2">
            {closedVariants.map((variant) => {
              const wouldBlock = isTerminal;
              return (
                <button
                  key={variant}
                  type="button"
                  onClick={() => onFire('Closed', variant)}
                  className={`focus-ring ui-transition rounded-xl border px-3 py-2.5 text-left ${
                    wouldBlock
                      ? 'border-red/30 bg-red/5 text-red'
                      : 'border-border bg-bg-elevated/40 text-text hover:border-accent/40 hover:bg-panel-hover'
                  }`}
                >
                  <span className="flex items-center justify-between gap-3">
                    <span className="text-sm font-semibold">Closed → {variant}</span>
                    <span className="font-mono text-[10px] text-dim/80">
                      Close({variant})
                    </span>
                  </span>
                  {closeExamples.length > 0 ? (
                    <span className="mt-1 block text-[11px] text-dim break-words">
                      {closeExamples.join(' / ')}
                    </span>
                  ) : null}
                  <span className="mt-2 block text-[10px] text-dim">
                    Needed for protocols whose terminal status is not fixed by the variant name.
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      ) : null}

      <EventGroup
        title="Metadata Events"
        entries={metadata}
        isTerminal={isTerminal}
        onFire={onFire}
      />
    </div>
  );
}
