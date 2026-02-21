import { motion } from 'framer-motion';

const LAYERS = [
  { label: 'Lifecycle E2E', desc: 'Raw JSON → adapter → state machine → status', color: '#22c55e', width: '40%' },
  { label: 'Fixture Tests', desc: 'Real JSON from defi-tracker', color: '#06b6d4', width: '50%' },
  { label: 'Unit Tests', desc: 'Per-protocol classify/resolve', color: '#3b82f6', width: '60%' },
  { label: 'EventType Reachability', desc: 'All 9 variants hit', color: '#a855f7', width: '70%' },
  { label: 'Mirror Alignment', desc: 'Serde dispatch ↔ Carbon variants', color: '#f59e0b', width: '80%' },
  { label: 'Compile-time', desc: 'Exhaustive match on Carbon enum', color: '#ef4444', width: '90%' },
];

export default function TestPyramid() {
  return (
    <div className="flex flex-col items-center gap-2">
      {LAYERS.map((layer, i) => (
        <motion.div
          key={layer.label}
          initial={{ opacity: 0, scaleX: 0.5 }}
          whileInView={{ opacity: 1, scaleX: 1 }}
          transition={{ delay: i * 0.08 }}
          viewport={{ once: true }}
          className="rounded-lg border px-4 py-2 text-center"
          style={{
            width: layer.width,
            borderColor: layer.color,
            backgroundColor: `${layer.color}15`,
          }}
        >
          <div className="text-xs font-semibold" style={{ color: layer.color }}>
            Layer {LAYERS.length - i}: {layer.label}
          </div>
          <div className="text-[10px] text-dim">{layer.desc}</div>
        </motion.div>
      ))}
    </div>
  );
}
