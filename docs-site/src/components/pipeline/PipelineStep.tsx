import { motion } from 'framer-motion';

interface Props {
  title: string;
  subtitle: string;
  detail: string;
  color: string;
  isActive: boolean;
  onClick: () => void;
  index: number;
}

export default function PipelineStep({
  title,
  subtitle,
  detail,
  color,
  isActive,
  onClick,
  index,
}: Props) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.1 }}
      viewport={{ once: true }}
      onClick={onClick}
      className={`flex-1 rounded-xl border-2 p-4 cursor-pointer transition-all min-w-[140px] ${
        isActive ? `border-[${color}] bg-[${color}]/10` : 'border-border bg-panel hover:border-dim'
      }`}
      style={isActive ? { borderColor: color, backgroundColor: `${color}15` } : {}}
    >
      <div className="text-xs font-mono text-dim mb-1">Stage {index + 1}</div>
      <div className="font-semibold text-sm mb-1" style={{ color: isActive ? color : undefined }}>
        {title}
      </div>
      <div className="text-[11px] text-dim">{subtitle}</div>
      {isActive && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          className="mt-3 pt-3 border-t border-border text-xs text-dim"
        >
          {detail}
        </motion.div>
      )}
    </motion.div>
  );
}
