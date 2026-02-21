import { motion } from 'framer-motion';
import type { ReactNode } from 'react';

interface Props {
  id: string;
  title: string;
  children: ReactNode;
  className?: string;
  index?: number;
}

export default function Section({ id, title, children, className = '', index = 0 }: Props) {
  return (
    <section id={id} className={`relative py-20 px-4 sm:px-8 ${className}`}>
      {index % 2 === 0 && (
        <div className="absolute inset-0 bg-bg-elevated pointer-events-none" />
      )}
      <div className="mx-auto max-w-6xl relative">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          className="mb-10"
        >
          <div className="flex items-center gap-3 mb-1">
            <span className="w-8 h-px accent-gradient-bg" />
            <span className="text-[10px] text-dim font-mono uppercase tracking-widest">
              {String(index + 1).padStart(2, '0')}
            </span>
          </div>
          <h2 className="text-2xl font-bold tracking-tight">{title}</h2>
        </motion.div>
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-40px' }}
          transition={{ duration: 0.6, delay: 0.1 }}
        >
          {children}
        </motion.div>
      </div>
    </section>
  );
}
