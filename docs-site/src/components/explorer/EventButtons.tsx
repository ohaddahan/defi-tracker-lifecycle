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
      <div className="text-[11px] text-dim mb-1.5 font-medium">{title}</div>
      <div className="flex flex-wrap gap-1.5">
        {entries.map(([name, type]) => {
          const transition = EVENT_TO_TRANSITION_LABEL[type] ?? 'MetadataOnly';
          const wouldBlock = isTerminal && transition !== 'MetadataOnly';
          return (
            <button
              key={name}
              onClick={() => type !== 'Closed' && onFire(type, null)}
              disabled={type === 'Closed'}
              className={`rounded-2xl border px-2.5 py-1 text-xs transition-all ${
                wouldBlock
                  ? 'border-red opacity-50 cursor-pointer'
                  : type === 'Closed'
                    ? 'border-border opacity-30 cursor-not-allowed'
                    : 'border-border text-text hover:border-green hover:text-green cursor-pointer'
              }`}
            >
              {name}
              <span className="block text-[9px] text-dim mt-px">
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
          <div className="text-[11px] text-dim mb-1.5 font-medium">Close Terminal Status</div>
          <div className="flex flex-wrap gap-1.5">
            {proto.closedVariants.map((variant) => {
              const wouldBlock = isTerminal;
              return (
                <button
                  key={variant}
                  onClick={() => onFire('Closed', variant)}
                  className={`rounded-2xl border px-2.5 py-1 text-xs transition-all cursor-pointer ${
                    wouldBlock
                      ? 'border-red opacity-50'
                      : 'border-border text-text hover:border-green hover:text-green'
                  }`}
                >
                  Close: {variant}
                  <span className="block text-[9px] text-dim mt-px">Close({variant})</span>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
