import { motion } from 'framer-motion';

const LAYERS = [
  { label: 'Lifecycle E2E', desc: 'Raw JSON → adapter → state machine → status', color: '#14f195', width: '40%' },
  { label: 'Fixture Tests', desc: 'Real JSON from defi-tracker', color: '#06d6a0', width: '50%' },
  { label: 'Unit Tests', desc: 'Per-protocol classify/resolve', color: '#3b82f6', width: '60%' },
  { label: 'EventType Reachability', desc: 'All 9 variants hit', color: '#9945ff', width: '70%' },
  { label: 'Mirror Alignment', desc: 'Serde dispatch ↔ Carbon variants', color: '#f59e0b', width: '80%' },
  { label: 'Compile-time', desc: 'Exhaustive match on Carbon enum', color: '#f43f5e', width: '90%' },
];

export default function TestPyramid() {
  return (
    <div className="flex flex-col items-center gap-2">
      {LAYERS.map((layer, i) => (
        <motion.div
          key={layer.label}
          initial={{ opacity: 0, scaleX: 0.3 }}
          whileInView={{ opacity: 1, scaleX: 1 }}
          transition={{ delay: i * 0.08, duration: 0.4 }}
          viewport={{ once: true }}
          className="rounded-lg border px-4 py-2.5 text-center"
          style={{
            width: layer.width,
            borderColor: `${layer.color}30`,
            backgroundColor: `${layer.color}08`,
          }}
        >
          <div className="text-[11px] font-semibold" style={{ color: layer.color }}>
            Layer {LAYERS.length - i}: {layer.label}
          </div>
          <div className="text-[9px] text-dim/60 font-mono mt-0.5">{layer.desc}</div>
        </motion.div>
      ))}
    </div>
  );
}
