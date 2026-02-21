import { PRESETS, type Preset } from '../../data/presets';

interface Props {
  onRun: (preset: Preset) => void;
  running: string | null;
}

export default function PresetBar({ onRun, running }: Props) {
  return (
    <div className="flex flex-wrap gap-1.5">
      {PRESETS.map((p) => {
        const isRunning = running === p.name;
        return (
          <button
            key={p.name}
            onClick={() => onRun(p)}
            disabled={running !== null}
            className={`rounded-md border px-2.5 py-1 text-[11px] font-medium transition-all cursor-pointer ${
              isRunning
                ? 'accent-gradient-bg border-transparent text-bg font-semibold'
                : 'border-border text-dim hover:border-border-active hover:text-text bg-bg-elevated/30'
            } disabled:opacity-40 disabled:cursor-not-allowed`}
          >
            {p.name}
          </button>
        );
      })}
    </div>
  );
}
