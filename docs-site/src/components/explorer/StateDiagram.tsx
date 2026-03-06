import { EDGES, NODES, colorToName } from '../../data/stateMachine';
import type { TransitionFlash } from '../../hooks/useLifecycleState';

interface Props {
  currentStatus: string | null;
  lastTransition: TransitionFlash | null;
}

function selfLoopPath(cx: number, cy: number, dy: number) {
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
    <svg
      viewBox="0 0 800 500"
      className="h-full w-full"
      role="img"
      aria-labelledby="state-diagram-title state-diagram-desc"
    >
      <title id="state-diagram-title">Order lifecycle state diagram</title>
      <desc id="state-diagram-desc">
        Create moves an order to Active. FillDelta and MetadataOnly keep it active. Close can move
        it to Completed, Cancelled, or Expired. Terminal states only accept MetadataOnly.
      </desc>

      <defs>
        {Object.entries(ARROW_COLORS).map(([name, color]) => (
          <marker
            key={name}
            id={`arrow-${name}`}
            markerWidth="8"
            markerHeight="6"
            refX="7"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 8 3, 0 6" fill={color} />
          </marker>
        ))}
      </defs>

      <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
        <path
          d="M 40 0 L 0 0 0 40"
          fill="none"
          stroke="rgba(255,255,255,0.02)"
          strokeWidth="0.5"
        />
      </pattern>
      <rect width="800" height="500" fill="url(#grid)" />

      <rect
        x="540"
        y="90"
        width="200"
        height="370"
        rx="16"
        fill="none"
        stroke="#1e2030"
        strokeWidth="1"
        strokeDasharray="6 4"
      />

      {EDGES.map((edge, index) => {
        const fromNode = NODES.find((node) => node.id === edge.from);
        const toNode = NODES.find((node) => node.id === edge.to);

        if (!fromNode || !toNode) {
          return null;
        }

        const markerName = colorToName(edge.color);

        if (edge.self) {
          const cx = fromNode.x + fromNode.w / 2;
          const cy = fromNode.y + ((edge.dy ?? 0) < 0 ? 0 : fromNode.h);
          const d = selfLoopPath(cx, cy, edge.dy ?? -40);

          return (
            <g key={index}>
              <path
                d={d}
                fill="none"
                stroke={edge.color}
                strokeWidth="1.5"
                strokeOpacity="0.7"
                markerEnd={`url(#arrow-${markerName})`}
              />
              <text
                x={cx}
                y={cy + (edge.dy ?? -40) * 1.3}
                textAnchor="middle"
                dominantBaseline="central"
                fill={edge.color}
                fontSize="9"
                opacity="0.8"
                fontFamily="'Lexend', system-ui, sans-serif"
                fontWeight="500"
              >
                {edge.label}
              </text>
            </g>
          );
        }

        const x1 = fromNode.x + fromNode.w;
        const y1 = fromNode.y + fromNode.h / 2;
        const x2 = toNode.x;
        const y2 = toNode.y + toNode.h / 2;
        const midpointX = (x1 + x2) / 2;

        return (
          <g key={index}>
            <path
              d={`M ${x1} ${y1} C ${midpointX} ${y1}, ${midpointX} ${y2}, ${x2} ${y2}`}
              fill="none"
              stroke={edge.color}
              strokeWidth="1.5"
              strokeOpacity="0.7"
              markerEnd={`url(#arrow-${markerName})`}
            />
            <text
              x={midpointX}
              y={(y1 + y2) / 2 - 8}
              textAnchor="middle"
              dominantBaseline="central"
              fill={edge.color}
              fontSize="9"
              opacity="0.8"
              fontFamily="'Lexend', system-ui, sans-serif"
              fontWeight="500"
            >
              {edge.label}
            </text>
          </g>
        );
      })}

      {NODES.map((node) => {
        const isCurrent = activeId === node.id;
        const isFlashing = lastTransition?.target === node.id;

        return (
          <g key={node.id}>
            {isFlashing ? (
              <rect
                key={`${node.id}-${lastTransition.step}`}
                x={node.x - 6}
                y={node.y - 6}
                width={node.w + 12}
                height={node.h + 12}
                rx="16"
                fill="none"
                stroke={lastTransition.decision === 'Apply' ? '#14f195' : '#f43f5e'}
                strokeWidth="2"
                className={
                  lastTransition.decision === 'Apply'
                    ? 'state-flash-apply'
                    : 'state-flash-block'
                }
              />
            ) : null}

            {isCurrent ? (
              <rect
                x={node.x - 3}
                y={node.y - 3}
                width={node.w + 6}
                height={node.h + 6}
                rx="14"
                fill="none"
                stroke={node.stroke}
                strokeWidth="1"
                opacity="0.3"
                className="pulse-glow"
              />
            ) : null}

            <rect
              x={node.x}
              y={node.y}
              width={node.w}
              height={node.h}
              rx="12"
              fill={isCurrent ? `${node.stroke}15` : node.color}
              stroke={node.stroke}
              strokeWidth={isCurrent ? 2 : 1}
              strokeOpacity={isCurrent ? 1 : 0.5}
              style={{
                filter: isCurrent ? `drop-shadow(0 0 16px ${node.stroke}40)` : undefined,
              }}
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

      <text
        x="640"
        y="460"
        textAnchor="middle"
        fill="#5c6078"
        fontSize="9"
        fontFamily="'Lexend', system-ui, sans-serif"
        fontWeight="300"
      >
        Terminal states only accept MetadataOnly
      </text>
    </svg>
  );
}
