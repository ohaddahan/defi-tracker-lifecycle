pub mod adapters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStatus {
    Completed,
    Cancelled,
    Expired,
}

impl TerminalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }

    pub fn from_status(status: &str) -> Option<Self> {
        match status {
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
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
    pub fn is_terminal_status(status: &str) -> bool {
        matches!(status, "cancelled" | "completed" | "expired")
    }

    pub fn decide_transition(
        current_status: Option<&str>,
        transition: LifecycleTransition,
    ) -> TransitionDecision {
        let current_is_terminal = current_status.is_some_and(Self::is_terminal_status);
        if !current_is_terminal {
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
            TerminalStatus::from_status("completed"),
            Some(TerminalStatus::Completed)
        );
        assert_eq!(
            TerminalStatus::from_status("cancelled"),
            Some(TerminalStatus::Cancelled)
        );
        assert_eq!(
            TerminalStatus::from_status("expired"),
            Some(TerminalStatus::Expired)
        );
        assert_eq!(TerminalStatus::from_status("active"), None);
        assert_eq!(TerminalStatus::Completed.as_str(), "completed");
    }

    #[test]
    fn terminal_statuses_are_detected() {
        assert!(LifecycleEngine::is_terminal_status("cancelled"));
        assert!(LifecycleEngine::is_terminal_status("completed"));
        assert!(LifecycleEngine::is_terminal_status("expired"));
        assert!(!LifecycleEngine::is_terminal_status("active"));
    }

    #[test]
    fn terminal_orders_reject_state_mutating_transitions() {
        let current = Some("completed");
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
        let terminal_statuses = ["completed", "cancelled", "expired"];
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
        let non_terminal_statuses = [None, Some("created"), Some("active"), Some("unknown")];

        for status in non_terminal_statuses {
            for _ in 0..3_000 {
                let transition = random_transition(&mut seed);
                let decision = LifecycleEngine::decide_transition(status, transition);
                assert_eq!(decision, TransitionDecision::Apply);
            }
        }
    }

    fn apply_sequence(steps: &[(LifecycleTransition, TransitionDecision)]) {
        let mut current_status: Option<&str> = None;

        for (i, (transition, expected_decision)) in steps.iter().enumerate() {
            let decision = LifecycleEngine::decide_transition(current_status, *transition);
            assert_eq!(
                decision, *expected_decision,
                "step {i}: expected {expected_decision:?} for {transition:?} with status {current_status:?}"
            );

            if decision == TransitionDecision::Apply {
                current_status = match transition {
                    LifecycleTransition::Create => Some("created"),
                    LifecycleTransition::FillDelta => current_status,
                    LifecycleTransition::Close { status } => Some(status.as_str()),
                    LifecycleTransition::MetadataOnly => current_status,
                };
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
