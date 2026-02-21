import { PROTOCOLS, type EventType, type ProtocolId } from '../../data/protocols';

interface Props {
  protocol: ProtocolId;
  isTerminal: boolean;
  onFire: (eventType: EventType, closedStatus: string | null) => void;
}

const EVENT_TO_TRANSITION_LABEL: Record<string, string> = {
  Created: 'Create',
  FillInitiated: 'FillDelta',
  FillCompleted: 'FillDelta',
  Cancelled: 'Close(Cancelled)',
  Expired: 'Close(Expired)',
  Closed: 'Close(?)',
  FeeCollected: 'MetadataOnly',
  Withdrawn: 'MetadataOnly',
  Deposited: 'MetadataOnly',
};

function EventGroup({
  title,
  entries,
  isTerminal,
  onFire,
}: {
  title: string;
  entries: [string, EventType][];
  isTerminal: boolean;
  onFire: (eventType: EventType, closedStatus: string | null) => void;
}) {
  return (
    <div className="mt-3">
      <div className="text-[10px] text-dim/70 mb-1.5 font-mono uppercase tracking-wider">{title}</div>
      <div className="flex flex-wrap gap-1.5">
        {entries.map(([name, type]) => {
          const transition = EVENT_TO_TRANSITION_LABEL[type] ?? 'MetadataOnly';
          const wouldBlock = isTerminal && transition !== 'MetadataOnly';
          return (
            <button
              key={name}
              onClick={() => type !== 'Closed' && onFire(type, null)}
              disabled={type === 'Closed'}
              className={`rounded-lg border px-2.5 py-1.5 text-xs transition-all ${
                wouldBlock
                  ? 'border-red/30 bg-red/5 text-red/60 cursor-pointer'
                  : type === 'Closed'
                    ? 'border-border/50 opacity-20 cursor-not-allowed'
                    : 'border-border bg-bg-elevated/30 text-text/80 hover:border-accent/40 hover:text-accent hover:bg-accent/5 cursor-pointer'
              }`}
            >
              <span className="block font-medium">{name}</span>
              <span className="block text-[8px] text-dim/60 mt-0.5 font-mono">
                {type} → {transition}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

export default function EventButtons({ protocol, isTerminal, onFire }: Props) {
  const proto = PROTOCOLS[protocol];
  return (
    <div>
      <EventGroup
        title="Instructions"
        entries={Object.entries(proto.instructions) as [string, EventType][]}
        isTerminal={isTerminal}
        onFire={onFire}
      />
      <EventGroup
        title="Events"
        entries={Object.entries(proto.events) as [string, EventType][]}
        isTerminal={isTerminal}
        onFire={onFire}
      />
      {proto.closedVariants.length > 0 && (
        <div className="mt-3">
          <div className="text-[10px] text-dim/70 mb-1.5 font-mono uppercase tracking-wider">Close Terminal Status</div>
          <div className="flex flex-wrap gap-1.5">
            {proto.closedVariants.map((variant) => {
              const wouldBlock = isTerminal;
              return (
                <button
                  key={variant}
                  onClick={() => onFire('Closed', variant)}
                  className={`rounded-lg border px-2.5 py-1.5 text-xs transition-all cursor-pointer ${
                    wouldBlock
                      ? 'border-red/30 bg-red/5 text-red/60'
                      : 'border-border bg-bg-elevated/30 text-text/80 hover:border-accent/40 hover:text-accent hover:bg-accent/5'
                  }`}
                >
                  <span className="block font-medium">Close: {variant}</span>
                  <span className="block text-[8px] text-dim/60 mt-0.5 font-mono">Close({variant})</span>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
