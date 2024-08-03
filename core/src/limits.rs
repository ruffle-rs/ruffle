use crate::context::UpdateContext;
use std::time::Duration;

/// Indication of how long execution is allowed to take.
///
/// Execution is limited by two mechanisms:
///
/// 1. An *operation limit*, or oplimit, which is a certain number of actions
///    executed, bytes processed, or other things that can be done before
///    checking...
/// 2. A *time limit*, which is checked every time the oplimit runs out.
///    If it has not yet expired, then the oplimit is refreshed and we
///    continue.
///
/// This two-tiered system is intended to reduce the overhead of enforcing
/// execution limits.
///
/// What constitutes an "operation" is up to the user of this structure. It may
/// cover individual interpreter opcodes, bytes decoded in a data stream, or so
/// on.
pub struct ExecutionLimit {
    /// How many operations remain before the next timelimit check.
    ///
    /// If `None`, then the execution limit is not enforced.
    current_oplimit: Option<usize>,

    /// The number of operations allowed between timelimit checks.
    ///
    /// If `None`, then the execution limit is not enforced.
    max_ops_per_check: Option<usize>,

    /// The amount of time allowed to be taken regardless of the opcount.
    time_limit: Duration,
}

impl ExecutionLimit {
    /// Construct an execution limit that allows unlimited execution.
    pub fn none() -> ExecutionLimit {
        Self {
            current_oplimit: None,
            max_ops_per_check: None,
            time_limit: Duration::MAX,
        }
    }

    /// Construct an execution limit that checks the current wall-clock time
    /// after a certain number of operations are executed.
    pub fn with_max_ops_and_time(ops: usize, time_limit: Duration) -> ExecutionLimit {
        Self {
            current_oplimit: Some(ops),
            max_ops_per_check: Some(ops),
            time_limit,
        }
    }

    /// Create an execution limit that is already exhausted.
    pub fn exhausted() -> ExecutionLimit {
        Self {
            current_oplimit: Some(0),
            max_ops_per_check: Some(0),
            time_limit: Duration::from_secs(0),
        }
    }

    /// Check if the execution of a certain number of operations has exceeded
    /// the execution limit.
    ///
    /// This is intended to be called after the given operations have been
    /// executed. The ops will be deducted from the operation limit and, if
    /// that limit is zero, the time limit will be checked. If both limits have
    /// been breached, this returns `true`. Otherwise, this returns `false`,
    /// and if the operation limit was exhausted, it will be returned to the
    /// starting maximum.
    pub fn did_ops_breach_limit(&mut self, context: &mut UpdateContext<'_>, ops: usize) -> bool {
        if let Some(oplimit) = &mut self.current_oplimit {
            *oplimit = oplimit.saturating_sub(ops);

            if *oplimit == 0 {
                if context.update_start.elapsed() >= self.time_limit {
                    return true;
                }

                self.current_oplimit = self.max_ops_per_check;
            }
        }

        false
    }
}
