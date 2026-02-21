import { PROTOCOL_LIST, type ProtocolId } from '../../data/protocols';

interface Props {
  selected: ProtocolId;
  onSelect: (id: ProtocolId) => void;
}

export default function ProtocolSelector({ selected, onSelect }: Props) {
  return (
    <div className="grid grid-cols-2 gap-2">
      {PROTOCOL_LIST.map((p) => (
        <button
          key={p.id}
          onClick={() => onSelect(p.id)}
          className={`rounded-lg border px-3 py-2 text-left text-sm transition-all ${
            selected === p.id
              ? 'border-accent bg-accent/15 text-accent'
              : 'border-border bg-transparent text-text hover:border-accent'
          }`}
        >
          <span className="font-semibold block">{p.label}</span>
          <span className="text-[10px] text-dim block mt-0.5">{p.sub}</span>
        </button>
      ))}
    </div>
  );
}
