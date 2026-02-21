import { motion } from 'framer-motion';

const STATS = [
  { value: '4', label: 'Protocols', color: 'text-accent' },
  { value: '9', label: 'Event Types', color: 'text-purple' },
  { value: '137', label: 'Tests', color: 'text-cyan' },
  { value: '0', label: 'IO / DB', color: 'text-amber' },
];

export default function Hero() {
  return (
    <div className="relative pt-28 pb-24 px-4 sm:px-8 overflow-hidden">
      {/* Gradient orbs */}
      <div className="absolute top-0 left-1/4 w-[600px] h-[600px] rounded-full bg-accent/[0.07] blur-[120px] gradient-orb pointer-events-none" />
      <div
        className="absolute top-20 right-1/4 w-[500px] h-[500px] rounded-full bg-purple/[0.05] blur-[100px] gradient-orb pointer-events-none"
        style={{ animationDelay: '-7s' }}
      />

      {/* Grid background */}
      <div
        className="absolute inset-0 pointer-events-none opacity-[0.03]"
        style={{
          backgroundImage:
            'linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)',
          backgroundSize: '60px 60px',
        }}
      />

      <div className="mx-auto max-w-4xl text-center relative">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="inline-flex items-center gap-2 rounded-full border border-border bg-panel/80 backdrop-blur-sm px-4 py-1.5 mb-8">
            <span className="w-1.5 h-1.5 rounded-full bg-accent pulse-glow" />
            <span className="text-[11px] text-dim font-medium tracking-wide">
              Pure-logic Solana crate
            </span>
          </div>
        </motion.div>

        <motion.h1
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
          className="text-4xl sm:text-6xl font-bold mb-6 tracking-tight"
        >
          <span className="accent-gradient">defi-tracker</span>
          <span className="text-border-active">-</span>
          <span className="text-text">lifecycle</span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2, ease: [0.22, 1, 0.36, 1] }}
          className="text-dim text-lg max-w-xl mx-auto mb-12 leading-relaxed font-light"
        >
          Classification, correlation, and state machine logic for DeFi order
          lifecycle tracking. Zero IO, zero DB.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3, ease: [0.22, 1, 0.36, 1] }}
          className="grid grid-cols-4 gap-4 sm:gap-8 max-w-md mx-auto mb-12"
        >
          {STATS.map((stat, i) => (
            <motion.div
              key={stat.label}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.4, delay: 0.4 + i * 0.08 }}
              className="text-center"
            >
              <div className={`text-2xl sm:text-3xl font-bold font-mono ${stat.color}`}>
                {stat.value}
              </div>
              <div className="text-[10px] text-dim mt-1 font-medium tracking-wider uppercase">
                {stat.label}
              </div>
            </motion.div>
          ))}
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.5, ease: [0.22, 1, 0.36, 1] }}
          className="flex justify-center gap-4"
        >
          <a
            href="#explorer"
            className="group relative rounded-lg px-6 py-3 text-sm font-semibold text-bg overflow-hidden transition-all hover:shadow-[0_0_30px_rgba(20,241,149,0.3)]"
          >
            <span className="absolute inset-0 accent-gradient-bg" />
            <span className="relative flex items-center gap-2">
              Explore State Machine
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" className="transition-transform group-hover:translate-y-0.5">
                <path d="M7 2v10M3 8l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </span>
          </a>
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-border px-6 py-3 text-sm font-semibold text-dim hover:text-text hover:border-border-active transition-all"
          >
            View Source
          </a>
        </motion.div>
      </div>
    </div>
  );
}
