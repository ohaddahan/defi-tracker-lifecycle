pub mod adapters;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum TerminalStatus {
    Completed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransition {
    Create,
    FillDelta,
    Close { status: TerminalStatus },
    MetadataOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionDecision {
    Apply,
    IgnoreTerminalViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotDelta {
    pub delta: i64,
    pub regression: bool,
}

pub struct LifecycleEngine;

impl LifecycleEngine {
    pub fn decide_transition(
        current_terminal: Option<TerminalStatus>,
        transition: LifecycleTransition,
    ) -> TransitionDecision {
        if current_terminal.is_none() {
            return TransitionDecision::Apply;
        }

        match transition {
            LifecycleTransition::MetadataOnly => TransitionDecision::Apply,
            LifecycleTransition::Create
            | LifecycleTransition::FillDelta
            | LifecycleTransition::Close { .. } => TransitionDecision::IgnoreTerminalViolation,
        }
    }

    pub fn normalize_snapshot_to_delta(stored_total: i64, snapshot_total: i64) -> SnapshotDelta {
        let delta = snapshot_total.saturating_sub(stored_total).max(0);
        SnapshotDelta {
            delta,
            regression: snapshot_total < stored_total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LifecycleEngine, LifecycleTransition, SnapshotDelta, TerminalStatus, TransitionDecision,
    };

    fn lcg_next(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        *state
    }

    fn random_transition(state: &mut u64) -> LifecycleTransition {
        match lcg_next(state) % 6 {
            0 => LifecycleTransition::Create,
            1 => LifecycleTransition::FillDelta,
            2 => LifecycleTransition::Close {
                status: TerminalStatus::Completed,
            },
            3 => LifecycleTransition::Close {
                status: TerminalStatus::Cancelled,
            },
            4 => LifecycleTransition::Close {
                status: TerminalStatus::Expired,
            },
            _ => LifecycleTransition::MetadataOnly,
        }
    }

    #[test]
    fn terminal_status_roundtrip() {
        assert_eq!(
            "completed".parse::<TerminalStatus>().ok(),
            Some(TerminalStatus::Completed)
        );
        assert_eq!(
            "cancelled".parse::<TerminalStatus>().ok(),
            Some(TerminalStatus::Cancelled)
        );
        assert_eq!(
            "expired".parse::<TerminalStatus>().ok(),
            Some(TerminalStatus::Expired)
        );
        assert_eq!("active".parse::<TerminalStatus>().ok(), None);
        assert_eq!(TerminalStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn terminal_orders_reject_state_mutating_transitions() {
        let current = Some(TerminalStatus::Completed);
        assert_eq!(
            LifecycleEngine::decide_transition(current, LifecycleTransition::Create),
            TransitionDecision::IgnoreTerminalViolation
        );
        assert_eq!(
            LifecycleEngine::decide_transition(current, LifecycleTransition::FillDelta),
            TransitionDecision::IgnoreTerminalViolation
        );
        assert_eq!(
            LifecycleEngine::decide_transition(
                current,
                LifecycleTransition::Close {
                    status: TerminalStatus::Cancelled
                }
            ),
            TransitionDecision::IgnoreTerminalViolation
        );
        assert_eq!(
            LifecycleEngine::decide_transition(current, LifecycleTransition::MetadataOnly),
            TransitionDecision::Apply
        );
    }

    #[test]
    fn snapshot_to_delta_never_regresses() {
        assert_eq!(
            LifecycleEngine::normalize_snapshot_to_delta(300, 450),
            SnapshotDelta {
                delta: 150,
                regression: false
            }
        );
        assert_eq!(
            LifecycleEngine::normalize_snapshot_to_delta(300, 300),
            SnapshotDelta {
                delta: 0,
                regression: false
            }
        );
        assert_eq!(
            LifecycleEngine::normalize_snapshot_to_delta(300, 200),
            SnapshotDelta {
                delta: 0,
                regression: true
            }
        );
    }

    #[test]
    fn snapshot_to_delta_property_holds_for_randomized_inputs() {
        let mut seed = 0x00C0_FFEE_u64;
        for _ in 0..20_000 {
            let stored_total = (lcg_next(&mut seed) as i64 % 2_000_000) - 1_000_000;
            let snapshot_total = (lcg_next(&mut seed) as i64 % 2_000_000) - 1_000_000;
            let normalized =
                LifecycleEngine::normalize_snapshot_to_delta(stored_total, snapshot_total);

            assert!(normalized.delta >= 0);
            assert_eq!(normalized.regression, snapshot_total < stored_total);

            if snapshot_total >= stored_total {
                assert_eq!(normalized.delta, snapshot_total - stored_total);
            } else {
                assert_eq!(normalized.delta, 0);
            }
        }
    }

    #[test]
    fn terminal_immutability_property_holds_for_all_terminals() {
        let terminal_statuses = [
            TerminalStatus::Completed,
            TerminalStatus::Cancelled,
            TerminalStatus::Expired,
        ];
        let mut seed = 0xDEAD_BEEF_u64;

        for status in terminal_statuses {
            for _ in 0..5_000 {
                let transition = random_transition(&mut seed);
                let decision = LifecycleEngine::decide_transition(Some(status), transition);
                match transition {
                    LifecycleTransition::MetadataOnly => {
                        assert_eq!(decision, TransitionDecision::Apply);
                    }
                    LifecycleTransition::Create
                    | LifecycleTransition::FillDelta
                    | LifecycleTransition::Close { .. } => {
                        assert_eq!(decision, TransitionDecision::IgnoreTerminalViolation);
                    }
                }
            }
        }
    }

    #[test]
    fn non_terminal_statuses_do_not_block_transitions() {
        let mut seed = 0xA11CE_u64;

        for _ in 0..12_000 {
            let transition = random_transition(&mut seed);
            let decision = LifecycleEngine::decide_transition(None, transition);
            assert_eq!(decision, TransitionDecision::Apply);
        }
    }

    fn apply_sequence(steps: &[(LifecycleTransition, TransitionDecision)]) {
        let mut current_terminal: Option<TerminalStatus> = None;

        for (i, (transition, expected_decision)) in steps.iter().enumerate() {
            let decision = LifecycleEngine::decide_transition(current_terminal, *transition);
            assert_eq!(
                decision, *expected_decision,
                "step {i}: expected {expected_decision:?} for {transition:?} with terminal {current_terminal:?}"
            );

            if decision == TransitionDecision::Apply {
                if let LifecycleTransition::Close { status } = transition {
                    current_terminal = Some(*status);
                }
            }
        }
    }

    #[test]
    fn lifecycle_sequence_dca_happy_path() {
        apply_sequence(&[
            (LifecycleTransition::Create, TransitionDecision::Apply),
            (LifecycleTransition::FillDelta, TransitionDecision::Apply),
            (LifecycleTransition::FillDelta, TransitionDecision::Apply),
            (
                LifecycleTransition::Close {
                    status: TerminalStatus::Completed,
                },
                TransitionDecision::Apply,
            ),
            (
                LifecycleTransition::FillDelta,
                TransitionDecision::IgnoreTerminalViolation,
            ),
        ]);
    }

    #[test]
    fn lifecycle_sequence_limit_cancel() {
        apply_sequence(&[
            (LifecycleTransition::Create, TransitionDecision::Apply),
            (LifecycleTransition::FillDelta, TransitionDecision::Apply),
            (
                LifecycleTransition::Close {
                    status: TerminalStatus::Cancelled,
                },
                TransitionDecision::Apply,
            ),
            (
                LifecycleTransition::Create,
                TransitionDecision::IgnoreTerminalViolation,
            ),
        ]);
    }

    #[test]
    fn lifecycle_sequence_limit_expired() {
        apply_sequence(&[
            (LifecycleTransition::Create, TransitionDecision::Apply),
            (
                LifecycleTransition::Close {
                    status: TerminalStatus::Expired,
                },
                TransitionDecision::Apply,
            ),
            (
                LifecycleTransition::FillDelta,
                TransitionDecision::IgnoreTerminalViolation,
            ),
        ]);
    }

    #[test]
    fn lifecycle_sequence_terminal_still_accepts_metadata() {
        apply_sequence(&[
            (LifecycleTransition::Create, TransitionDecision::Apply),
            (
                LifecycleTransition::Close {
                    status: TerminalStatus::Completed,
                },
                TransitionDecision::Apply,
            ),
            (LifecycleTransition::MetadataOnly, TransitionDecision::Apply),
            (LifecycleTransition::MetadataOnly, TransitionDecision::Apply),
            (
                LifecycleTransition::Close {
                    status: TerminalStatus::Cancelled,
                },
                TransitionDecision::IgnoreTerminalViolation,
            ),
        ]);
    }
}
