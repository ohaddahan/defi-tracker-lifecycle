import { PRESETS, type Preset } from '../../data/presets';

interface Props {
  onRun: (preset: Preset) => void;
  running: string | null;
}

export default function PresetBar({ onRun, running }: Props) {
  return (
    <div className="flex flex-wrap gap-1.5">
      {PRESETS.map((p) => (
        <button
          key={p.name}
          onClick={() => onRun(p)}
          disabled={running !== null}
          className={`rounded-md border px-2.5 py-1 text-xs transition-all cursor-pointer ${
            running === p.name
              ? 'bg-accent border-accent text-white'
              : 'border-border text-text hover:border-accent hover:text-accent'
          } disabled:opacity-50`}
        >
          {p.name}
        </button>
      ))}
    </div>
  );
}
