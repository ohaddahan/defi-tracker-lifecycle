import { PROTOCOL_LIST, type ProtocolId } from '../../data/protocols';

interface Props {
  selected: ProtocolId;
  onSelect: (id: ProtocolId) => void;
}

export default function ProtocolSelector({ selected, onSelect }: Props) {
  return (
    <div className="grid grid-cols-2 gap-2">
      {PROTOCOL_LIST.map((p) => {
        const isActive = selected === p.id;
        return (
          <button
            key={p.id}
            onClick={() => onSelect(p.id)}
            className={`rounded-lg border px-3 py-2.5 text-left text-sm transition-all cursor-pointer ${
              isActive
                ? 'border-accent/40 bg-accent/[0.08] text-accent'
                : 'border-border bg-bg-elevated/50 text-text hover:border-border-active hover:bg-panel-hover'
            }`}
          >
            <span className="font-semibold block text-xs">{p.label}</span>
            <span className={`text-[10px] block mt-0.5 ${isActive ? 'text-accent/60' : 'text-dim'}`}>
              {p.sub}
            </span>
          </button>
        );
      })}
    </div>
  );
}
