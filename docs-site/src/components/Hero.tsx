import { motion } from 'framer-motion';

const HIGHLIGHTS = [
  { value: '4', label: 'Protocols', color: 'text-accent' },
  { value: '9', label: 'Event Types', color: 'text-purple' },
  { value: '0', label: 'IO / DB', color: 'text-amber' },
  { value: 'WASM', label: 'Docs Playground', color: 'text-cyan' },
];

export default function Hero() {
  return (
    <section className="relative overflow-hidden px-4 pb-20 pt-28 sm:px-8">
      <div className="gradient-orb absolute left-1/4 top-0 h-[520px] w-[520px] rounded-full bg-accent/[0.07] blur-[120px] pointer-events-none" />
      <div
        className="gradient-orb absolute right-1/4 top-12 h-[440px] w-[440px] rounded-full bg-purple/[0.05] blur-[100px] pointer-events-none"
        style={{ animationDelay: '-7s' }}
      />
      <div
        className="absolute inset-0 pointer-events-none opacity-[0.03]"
        style={{
          backgroundImage:
            'linear-gradient(rgba(255,255,255,0.08) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.08) 1px, transparent 1px)',
          backgroundSize: '60px 60px',
        }}
      />

      <div className="relative mx-auto max-w-6xl">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.45 }}
          className="inline-flex items-center gap-2 rounded-full border border-border bg-panel/80 px-4 py-1.5 backdrop-blur-sm"
        >
          <span className="pulse-glow h-1.5 w-1.5 rounded-full bg-accent" />
          <span className="text-[11px] font-medium tracking-wide text-dim">
            Pure-logic Solana order lifecycle crate
          </span>
        </motion.div>

        <div className="mt-8 grid grid-cols-1 gap-12 lg:grid-cols-[minmax(0,1fr)_320px] lg:items-end">
          <div>
            <motion.h1
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.05 }}
              className="max-w-4xl text-4xl font-bold tracking-tight text-text sm:text-6xl"
            >
              Canonical lifecycle logic for DCA, limit orders, and Kamino.
            </motion.h1>

            <motion.p
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.12 }}
              className="mt-5 max-w-2xl text-lg leading-relaxed text-dim"
            >
              Use one crate to classify raw variants, resolve correlation details, and enforce the
              same terminal-state rules across every supported protocol.
            </motion.p>

            <motion.div
              initial={{ opacity: 0, y: 18 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.45, delay: 0.2 }}
              className="mt-8 flex flex-wrap gap-3"
            >
              <a
                href="#quick-start"
                className="focus-ring ui-transition group relative overflow-hidden rounded-lg px-6 py-3 text-sm font-semibold text-bg hover:shadow-[0_0_30px_rgba(20,241,149,0.28)]"
              >
                <span className="accent-gradient-bg absolute inset-0" />
                <span className="relative flex items-center gap-2">
                  Start With 4 Steps
                  <svg
                    aria-hidden="true"
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    fill="none"
                    className="ui-transition group-hover:translate-y-0.5"
                  >
                    <path
                      d="M7 2v10M3 8l4 4 4-4"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                </span>
              </a>

              <a
                href="#playground"
                className="focus-ring ui-transition rounded-lg border border-border px-6 py-3 text-sm font-semibold text-dim hover:border-border-active hover:text-text"
              >
                Open Playground
              </a>
            </motion.div>
          </div>

          <motion.aside
            initial={{ opacity: 0, y: 28 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="rounded-2xl border border-border bg-panel/80 p-5 backdrop-blur-sm"
          >
            <div className="text-[10px] font-mono font-semibold uppercase tracking-widest text-dim">
              Good Fit If You Need
            </div>
            <ul className="mt-4 space-y-3 text-sm text-dim">
              <li className="flex gap-3">
                <span className="text-accent">01</span>
                <span>One canonical EventType layer across heterogeneous protocols.</span>
              </li>
              <li className="flex gap-3">
                <span className="text-accent">02</span>
                <span>Terminal-state enforcement without wiring protocol rules in multiple places.</span>
              </li>
              <li className="flex gap-3">
                <span className="text-accent">03</span>
                <span>Logic that can run natively and in a WASM-backed docs playground.</span>
              </li>
            </ul>
          </motion.aside>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.45, delay: 0.28 }}
          className="mt-10 grid max-w-2xl grid-cols-2 gap-4 sm:grid-cols-4"
        >
          {HIGHLIGHTS.map((highlight) => (
            <div
              key={highlight.label}
              className="rounded-2xl border border-border bg-panel/60 p-4 text-center backdrop-blur-sm"
            >
              <div className={`font-mono text-2xl font-bold ${highlight.color}`}>
                {highlight.value}
              </div>
              <div className="mt-1 text-[10px] font-medium uppercase tracking-wider text-dim">
                {highlight.label}
              </div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
