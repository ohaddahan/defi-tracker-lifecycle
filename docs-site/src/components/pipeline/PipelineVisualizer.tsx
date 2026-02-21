import { useState } from 'react';
import PipelineStep from './PipelineStep';

const STAGES = [
  {
    title: 'Raw Input',
    subtitle: 'RawInstruction / RawEvent',
    detail:
      'Transaction data arrives as JSON with program_id, instruction_name, accounts, args (instructions) or event_name and fields (events).',
    color: '#6b7280',
  },
  {
    title: 'from_program_id()',
    subtitle: 'Protocol → Adapter',
    detail:
      'The program ID string is parsed to a Pubkey and compared against Carbon decoder constants to identify the Protocol enum variant.',
    color: '#3b82f6',
  },
  {
    title: 'adapter_for()',
    subtitle: 'Protocol → ProtocolAdapter',
    detail:
      'Each Protocol variant maps to a zero-sized static adapter (DcaAdapter, LimitV1Adapter, etc.) implementing the ProtocolAdapter trait.',
    color: '#a855f7',
  },
  {
    title: 'classify / resolve',
    subtitle: 'EventType + Correlation + Payload',
    detail:
      'Instructions: serde envelope dispatch → EventType. Events: serde envelope → field extraction → (EventType, CorrelationOutcome, EventPayload).',
    color: '#06b6d4',
  },
  {
    title: 'decide_transition()',
    subtitle: 'TransitionDecision',
    detail:
      'Consumer maps EventType → LifecycleTransition, then LifecycleEngine decides: Apply (non-terminal or MetadataOnly) or IgnoreTerminalViolation.',
    color: '#22c55e',
  },
];

export default function PipelineVisualizer() {
  const [active, setActive] = useState(0);

  return (
    <div className="rounded-xl border border-border bg-panel p-6">
      <div className="flex flex-col sm:flex-row gap-3 items-stretch">
        {STAGES.map((stage, i) => (
          <div key={i} className="flex items-center flex-1">
            <PipelineStep
              title={stage.title}
              subtitle={stage.subtitle}
              detail={stage.detail}
              color={stage.color}
              isActive={active === i}
              onClick={() => setActive(i)}
              index={i}
            />
            {i < STAGES.length - 1 && (
              <div className="hidden sm:flex items-center px-1">
                <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                  <path d="M5 10h10M12 7l3 3-3 3" stroke="#6b7280" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
