import { motion, AnimatePresence } from 'framer-motion';
import { NODES, EDGES, colorToName } from '../../data/stateMachine';

interface Props {
  currentStatus: string | null;
  lastTransition: { target: string; decision: string } | null;
}

function SelfLoopPath({ cx, cy, dy }: { cx: number; cy: number; dy: number }) {
  return `M ${cx - 20} ${cy} C ${cx - 30} ${cy + dy * 1.5}, ${cx + 30} ${cy + dy * 1.5}, ${cx + 20} ${cy}`;
}

export default function StateDiagram({ currentStatus, lastTransition }: Props) {
  const activeId = currentStatus ?? 'none';

  return (
    <svg viewBox="0 0 800 500" className="w-full h-full">
      <defs>
        {Object.entries({ blue: '#3b82f6', green: '#22c55e', red: '#ef4444', amber: '#f59e0b', cyan: '#06b6d4', gray: '#6b7280' }).map(
          ([name, color]) => (
            <marker key={name} id={`arrow-${name}`} markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
              <polygon points="0 0, 8 3, 0 6" fill={color} />
            </marker>
          ),
        )}
      </defs>

      <rect x="540" y="90" width="200" height="370" rx="16" fill="none" stroke="#2a2d36" strokeWidth="1" strokeDasharray="6 4" />

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
              <path d={d} fill="none" stroke={edge.color} strokeWidth="2" markerEnd={`url(#arrow-${markerName})`} />
              <text x={cx} y={cy + (edge.dy ?? -40) * 1.3} textAnchor="middle" dominantBaseline="central" fill={edge.color} fontSize="10"
                fontFamily="system-ui, -apple-system, sans-serif">
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
              strokeWidth="2"
              markerEnd={`url(#arrow-${markerName})`}
            />
            <text x={mx} y={(y1 + y2) / 2 - 8} textAnchor="middle" dominantBaseline="central" fill={edge.color} fontSize="10"
              fontFamily="system-ui, -apple-system, sans-serif">
              {edge.label}
            </text>
          </g>
        );
      })}

      {NODES.map((node) => {
        const isCurrent = activeId === node.id;
        const isFlashing =
          lastTransition && lastTransition.target === node.id;

        return (
          <g key={node.id}>
            <AnimatePresence>
              {isFlashing && (
                <motion.rect
                  x={node.x - 4}
                  y={node.y - 4}
                  width={node.w + 8}
                  height={node.h + 8}
                  rx={14}
                  fill="none"
                  stroke={lastTransition.decision === 'Apply' ? '#22c55e' : '#ef4444'}
                  strokeWidth="2"
                  initial={{ opacity: 0.8 }}
                  animate={{ opacity: 0 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.6 }}
                />
              )}
            </AnimatePresence>
            <motion.rect
              x={node.x}
              y={node.y}
              width={node.w}
              height={node.h}
              rx={12}
              fill={node.color}
              stroke={node.stroke}
              strokeWidth={isCurrent ? 3 : 2}
              animate={
                isCurrent
                  ? { filter: `drop-shadow(0 0 12px ${node.stroke})` }
                  : { filter: 'drop-shadow(0 0 0px transparent)' }
              }
              transition={{ duration: 0.3 }}
            />
            <text
              x={node.x + node.w / 2}
              y={node.y + node.h / 2 - 6}
              textAnchor="middle"
              dominantBaseline="central"
              fill="#e0e0e6"
              fontSize="14"
              fontWeight="600"
              style={{ pointerEvents: 'none' }}
              fontFamily="system-ui, -apple-system, sans-serif"
            >
              {node.label}
            </text>
            <text
              x={node.x + node.w / 2}
              y={node.y + node.h / 2 + 12}
              textAnchor="middle"
              dominantBaseline="central"
              fill="#6b7280"
              fontSize="10"
              style={{ pointerEvents: 'none' }}
              fontFamily="system-ui, -apple-system, sans-serif"
            >
              {node.sub}
            </text>
          </g>
        );
      })}

      <text x="640" y="460" textAnchor="middle" fill="#6b7280" fontSize="11" fontFamily="system-ui, -apple-system, sans-serif">
        Terminal states only accept MetadataOnly
      </text>
    </svg>
  );
}
