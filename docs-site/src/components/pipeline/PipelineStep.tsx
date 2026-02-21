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
      transition={{ delay: index * 0.08 }}
      viewport={{ once: true }}
      onClick={onClick}
      className={`flex-1 rounded-xl border p-4 cursor-pointer transition-all min-w-[130px] ${
        isActive ? 'border-border-active' : 'border-border bg-bg-elevated/30 hover:border-border-active'
      }`}
      style={isActive ? { borderColor: `${color}40`, backgroundColor: `${color}08` } : {}}
    >
      <div className="text-[9px] font-mono text-dim/50 mb-1 uppercase tracking-wider">Stage {index + 1}</div>
      <div className="font-semibold text-xs mb-0.5" style={{ color: isActive ? color : undefined }}>
        {title}
      </div>
      <div className="text-[10px] text-dim/70 font-mono">{subtitle}</div>
      {isActive && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          className="mt-3 pt-3 border-t border-border/50 text-[11px] text-dim/80 leading-relaxed"
        >
          {detail}
        </motion.div>
      )}
    </motion.div>
  );
}
