use crate::context::UpdateContext;
use std::time::Duration;

/// Indication of how long execution is allowed to take.
///
/// Execution is limited by two mechanisms:
///
/// 1. An *action limit*, which is a certain number of *actions* that can be
///    taken before checking...
/// 2. A *time limit*, which is checked every time the action limit runs out.
///    If it has not yet expired, then the action limit is refreshed.
///
/// This two-tiered system is intended to reduce the overhead of enforcing
/// execution limits.
///
/// What constitutes an "action" is up to the user of this structure. It may
/// cover individual interpreter opcodes, bytes decoded in a data stream, or so
/// on.
pub struct ExecutionLimit {
    /// How many actions remain before the next timelimit check.
    ///
    /// If `None`, then the execution limit is not enforced.
    current_action_limit: Option<usize>,

    /// The number of actions allowed between timelimit checks.
    ///
    /// If `None`, then the execution limit is not enforced.
    max_actions_per_check: Option<usize>,

    /// The amount of time allowed to be taken regardless of the action count.
    time_limit: Duration,
}

impl ExecutionLimit {
    /// Construct an execution limit that allows unlimited execution.
    pub fn none() -> ExecutionLimit {
        Self {
            current_action_limit: None,
            max_actions_per_check: None,
            time_limit: Duration::MAX,
        }
    }

    /// Construct an execution limit that checks the current wall-clock time
    /// after a certain number of actions are taken.
    pub fn with_max_actions_and_time(actions: usize, time_limit: Duration) -> ExecutionLimit {
        Self {
            current_action_limit: Some(actions),
            max_actions_per_check: Some(actions),
            time_limit,
        }
    }

    /// Create an execution limit that is already expired.
    ///
    /// This is intended to indicate that only the smallest action possible be
    /// taken before returning. Code that obeys an execution limit must be able
    /// to make forward progress even when restricted to this limit.
    pub fn exhausted() -> ExecutionLimit {
        Self {
            current_action_limit: Some(0),
            max_actions_per_check: Some(0),
            time_limit: Duration::from_secs(0),
        }
    }

    /// Check if the execution of a certain number of actions has exceeded the
    /// execution limit.
    ///
    /// This is intended to be called after the given actions have been
    /// executed. The actions will be deducted from the action limit and, if
    /// that limit is zero, the time limit will be checked. If both limits have
    /// been breached, this returns `true`. Otherwise, this returns `false`,
    /// and if the action limit was exhausted, it will be returned to the
    /// starting maximum.
    pub fn did_actions_breach_limit(
        &mut self,
        context: &mut UpdateContext<'_, '_, '_>,
        actions: usize,
    ) -> bool {
        if let Some(ref mut action_limit) = self.current_action_limit {
            *action_limit = action_limit.saturating_sub(actions);

            if *action_limit == 0 {
                if context.update_start.elapsed() >= self.time_limit {
                    return true;
                }

                self.current_action_limit = self.max_actions_per_check;
            }
        }

        false
    }
}
