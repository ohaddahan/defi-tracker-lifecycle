use crate::lifecycle::{LifecycleTransition, TerminalStatus};
use crate::protocols::EventType;

/// Canonical mapping from [`EventType`] to [`LifecycleTransition`].
///
/// `closed_status` is only used when `event_type` is [`EventType::Closed`] — it provides
/// the terminal status derived from protocol-specific fields (e.g. DCA's `user_closed` + `unfilled_amount`).
/// When `Closed` has no `closed_status`, falls back to `MetadataOnly`.
pub fn event_type_to_transition(
    event_type: &EventType,
    closed_status: Option<TerminalStatus>,
) -> LifecycleTransition {
    match event_type {
        EventType::Created => LifecycleTransition::Create,
        EventType::FillInitiated | EventType::FillCompleted => LifecycleTransition::FillDelta,
        EventType::Cancelled => LifecycleTransition::Close {
            status: TerminalStatus::Cancelled,
        },
        EventType::Expired => LifecycleTransition::Close {
            status: TerminalStatus::Expired,
        },
        EventType::Closed => match closed_status {
            Some(s) => LifecycleTransition::Close { status: s },
            None => LifecycleTransition::MetadataOnly,
        },
        EventType::FeeCollected | EventType::Withdrawn | EventType::Deposited => {
            LifecycleTransition::MetadataOnly
        }
    }
}

/// Human-readable display string for a transition.
pub fn transition_to_display(transition: &LifecycleTransition) -> String {
    match transition {
        LifecycleTransition::Create => "Create".to_string(),
        LifecycleTransition::FillDelta => "FillDelta".to_string(),
        LifecycleTransition::Close { status } => {
            let s = status.as_ref();
            let capitalized = format!(
                "{}{}",
                s.get(..1).unwrap_or_default().to_uppercase(),
                s.get(1..).unwrap_or_default()
            );
            format!("Close({capitalized})")
        }
        LifecycleTransition::MetadataOnly => "MetadataOnly".to_string(),
    }
}

/// Returns the target status string after applying a transition, or `None` for status-preserving transitions.
pub fn transition_target(transition: &LifecycleTransition) -> Option<&'static str> {
    match transition {
        LifecycleTransition::Create => Some("active"),
        LifecycleTransition::Close { status } => match status {
            TerminalStatus::Completed => Some("completed"),
            TerminalStatus::Cancelled => Some("cancelled"),
            TerminalStatus::Expired => Some("expired"),
        },
        LifecycleTransition::FillDelta | LifecycleTransition::MetadataOnly => None,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn event_type_to_transition_covers_all_variants() {
        assert_eq!(
            event_type_to_transition(&EventType::Created, None),
            LifecycleTransition::Create
        );
        assert_eq!(
            event_type_to_transition(&EventType::FillInitiated, None),
            LifecycleTransition::FillDelta
        );
        assert_eq!(
            event_type_to_transition(&EventType::FillCompleted, None),
            LifecycleTransition::FillDelta
        );
        assert_eq!(
            event_type_to_transition(&EventType::Cancelled, None),
            LifecycleTransition::Close {
                status: TerminalStatus::Cancelled
            }
        );
        assert_eq!(
            event_type_to_transition(&EventType::Expired, None),
            LifecycleTransition::Close {
                status: TerminalStatus::Expired
            }
        );
        assert_eq!(
            event_type_to_transition(&EventType::Closed, Some(TerminalStatus::Completed)),
            LifecycleTransition::Close {
                status: TerminalStatus::Completed
            }
        );
        assert_eq!(
            event_type_to_transition(&EventType::Closed, None),
            LifecycleTransition::MetadataOnly
        );
        assert_eq!(
            event_type_to_transition(&EventType::FeeCollected, None),
            LifecycleTransition::MetadataOnly
        );
        assert_eq!(
            event_type_to_transition(&EventType::Withdrawn, None),
            LifecycleTransition::MetadataOnly
        );
        assert_eq!(
            event_type_to_transition(&EventType::Deposited, None),
            LifecycleTransition::MetadataOnly
        );
    }

    #[test]
    fn transition_display_strings() {
        assert_eq!(
            transition_to_display(&LifecycleTransition::Create),
            "Create"
        );
        assert_eq!(
            transition_to_display(&LifecycleTransition::FillDelta),
            "FillDelta"
        );
        assert_eq!(
            transition_to_display(&LifecycleTransition::Close {
                status: TerminalStatus::Completed
            }),
            "Close(Completed)"
        );
        assert_eq!(
            transition_to_display(&LifecycleTransition::Close {
                status: TerminalStatus::Cancelled
            }),
            "Close(Cancelled)"
        );
        assert_eq!(
            transition_to_display(&LifecycleTransition::MetadataOnly),
            "MetadataOnly"
        );
    }

    #[test]
    fn transition_targets() {
        assert_eq!(
            transition_target(&LifecycleTransition::Create),
            Some("active")
        );
        assert_eq!(
            transition_target(&LifecycleTransition::Close {
                status: TerminalStatus::Completed
            }),
            Some("completed")
        );
        assert_eq!(transition_target(&LifecycleTransition::FillDelta), None);
        assert_eq!(transition_target(&LifecycleTransition::MetadataOnly), None);
    }
}
