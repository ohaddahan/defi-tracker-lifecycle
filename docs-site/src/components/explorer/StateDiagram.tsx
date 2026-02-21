import { motion, AnimatePresence } from 'framer-motion';
import { NODES, EDGES, colorToName } from '../../data/stateMachine';

interface Props {
  currentStatus: string | null;
  lastTransition: { target: string; decision: string } | null;
}

function SelfLoopPath({ cx, cy, dy }: { cx: number; cy: number; dy: number }) {
  return `M ${cx - 20} ${cy} C ${cx - 30} ${cy + dy * 1.5}, ${cx + 30} ${cy + dy * 1.5}, ${cx + 20} ${cy}`;
}

const ARROW_COLORS: Record<string, string> = {
  blue: '#3b82f6',
  green: '#14f195',
  red: '#f43f5e',
  amber: '#f59e0b',
  cyan: '#06d6a0',
  gray: '#5c6078',
};

export default function StateDiagram({ currentStatus, lastTransition }: Props) {
  const activeId = currentStatus ?? 'none';

  return (
    <svg viewBox="0 0 800 500" className="w-full h-full">
      <defs>
        {Object.entries(ARROW_COLORS).map(([name, color]) => (
          <marker key={name} id={`arrow-${name}`} markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
            <polygon points="0 0, 8 3, 0 6" fill={color} />
          </marker>
        ))}
        <filter id="node-glow">
          <feGaussianBlur stdDeviation="8" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* Subtle grid */}
      <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
        <path d="M 40 0 L 0 0 0 40" fill="none" stroke="rgba(255,255,255,0.02)" strokeWidth="0.5" />
      </pattern>
      <rect width="800" height="500" fill="url(#grid)" />

      {/* Terminal states box */}
      <rect x="540" y="90" width="200" height="370" rx="16" fill="none" stroke="#1e2030" strokeWidth="1" strokeDasharray="6 4" />

      {EDGES.map((edge, i) => {
        const fromNode = NODES.find((n) => n.id === edge.from)!;
        const toNode = NODES.find((n) => n.id === edge.to)!;
        const markerName = colorToName(edge.color);

        if (edge.self) {
          const cx = fromNode.x + fromNode.w / 2;
          const cy = fromNode.y + ((edge.dy ?? 0) < 0 ? 0 : fromNode.h);
          const d = SelfLoopPath({ cx, cy, dy: edge.dy ?? -40 });
          return (
            <g key={i}>
              <path d={d} fill="none" stroke={edge.color} strokeWidth="1.5" strokeOpacity="0.7" markerEnd={`url(#arrow-${markerName})`} />
              <text x={cx} y={cy + (edge.dy ?? -40) * 1.3} textAnchor="middle" dominantBaseline="central" fill={edge.color} fontSize="9" opacity="0.8"
                fontFamily="'Lexend', system-ui, sans-serif" fontWeight="500">
                {edge.label}
              </text>
            </g>
          );
        }

        const x1 = fromNode.x + fromNode.w;
        const y1 = fromNode.y + fromNode.h / 2;
        const x2 = toNode.x;
        const y2 = toNode.y + toNode.h / 2;
        const mx = (x1 + x2) / 2;

        return (
          <g key={i}>
            <path
              d={`M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`}
              fill="none"
              stroke={edge.color}
              strokeWidth="1.5"
              strokeOpacity="0.7"
              markerEnd={`url(#arrow-${markerName})`}
            />
            <text x={mx} y={(y1 + y2) / 2 - 8} textAnchor="middle" dominantBaseline="central" fill={edge.color} fontSize="9" opacity="0.8"
              fontFamily="'Lexend', system-ui, sans-serif" fontWeight="500">
              {edge.label}
            </text>
          </g>
        );
      })}

      {NODES.map((node) => {
        const isCurrent = activeId === node.id;
        const isFlashing = lastTransition && lastTransition.target === node.id;

        return (
          <g key={node.id}>
            <AnimatePresence>
              {isFlashing && (
                <motion.rect
                  x={node.x - 6}
                  y={node.y - 6}
                  width={node.w + 12}
                  height={node.h + 12}
                  rx={16}
                  fill="none"
                  stroke={lastTransition.decision === 'Apply' ? '#14f195' : '#f43f5e'}
                  strokeWidth="2"
                  initial={{ opacity: 0.9 }}
                  animate={{ opacity: 0 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.7 }}
                />
              )}
            </AnimatePresence>

            {/* Active glow ring */}
            {isCurrent && (
              <rect
                x={node.x - 3}
                y={node.y - 3}
                width={node.w + 6}
                height={node.h + 6}
                rx={14}
                fill="none"
                stroke={node.stroke}
                strokeWidth="1"
                opacity="0.3"
                className="pulse-glow"
              />
            )}

            <motion.rect
              x={node.x}
              y={node.y}
              width={node.w}
              height={node.h}
              rx={12}
              fill={isCurrent ? `${node.stroke}15` : node.color}
              stroke={node.stroke}
              strokeWidth={isCurrent ? 2 : 1}
              strokeOpacity={isCurrent ? 1 : 0.5}
              animate={
                isCurrent
                  ? { filter: `drop-shadow(0 0 16px ${node.stroke}40)` }
                  : { filter: 'drop-shadow(0 0 0px transparent)' }
              }
              transition={{ duration: 0.3 }}
            />
            <text
              x={node.x + node.w / 2}
              y={node.y + node.h / 2 - 6}
              textAnchor="middle"
              dominantBaseline="central"
              fill={isCurrent ? node.stroke : '#e8e8f0'}
              fontSize="13"
              fontWeight="600"
              style={{ pointerEvents: 'none' }}
              fontFamily="'Lexend', system-ui, sans-serif"
            >
              {node.label}
            </text>
            <text
              x={node.x + node.w / 2}
              y={node.y + node.h / 2 + 12}
              textAnchor="middle"
              dominantBaseline="central"
              fill="#5c6078"
              fontSize="9"
              style={{ pointerEvents: 'none' }}
              fontFamily="'JetBrains Mono', monospace"
            >
              {node.sub}
            </text>
          </g>
        );
      })}

      <text x="640" y="460" textAnchor="middle" fill="#5c6078" fontSize="9" fontFamily="'Lexend', system-ui, sans-serif" fontWeight="300">
        Terminal states only accept MetadataOnly
      </text>
    </svg>
  );
}
