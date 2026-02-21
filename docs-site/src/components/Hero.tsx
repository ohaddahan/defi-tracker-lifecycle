import { motion } from 'framer-motion';

const STATS = [
  { value: '4', label: 'Protocols' },
  { value: '9', label: 'Event Types' },
  { value: '137', label: 'Tests' },
  { value: '0', label: 'IO / DB' },
];

export default function Hero() {
  return (
    <div className="pt-24 pb-16 px-4 sm:px-8">
      <div className="mx-auto max-w-6xl text-center">
        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-4xl sm:text-5xl font-bold mb-4"
        >
          <span className="text-accent">defi-tracker</span>
          <span className="text-dim">-</span>
          <span className="text-text">lifecycle</span>
        </motion.h1>
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="text-dim text-lg max-w-2xl mx-auto mb-8"
        >
          Pure-logic Rust crate for DeFi order lifecycle tracking on Solana.
          <br />
          Zero IO/DB — classification, correlation, and state machine logic only.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="flex justify-center gap-4 sm:gap-8 mb-8"
        >
          {STATS.map((stat) => (
            <div key={stat.label} className="text-center">
              <div className="text-2xl sm:text-3xl font-bold text-accent">{stat.value}</div>
              <div className="text-xs text-dim mt-1">{stat.label}</div>
            </div>
          ))}
        </motion.div>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
          className="flex justify-center gap-3"
        >
          <a
            href="#explorer"
            className="rounded-lg bg-accent px-5 py-2.5 text-sm font-semibold text-white hover:bg-accent/80 transition-all"
          >
            Explore State Machine
          </a>
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-border px-5 py-2.5 text-sm font-semibold text-text hover:border-accent hover:text-accent transition-all"
          >
            View Source
          </a>
        </motion.div>
      </div>
    </div>
  );
}
