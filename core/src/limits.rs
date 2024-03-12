use crate::context::UpdateContext;

/// Indication of how long execution is allowed to take.
///
/// Execution is limited by *operation limit*, or oplimit,
/// which is a certain number of actions
/// executed, bytes processed, or other things that can be done.
///
/// What constitutes an "operation" is up to the user of this structure. It may
/// cover individual interpreter opcodes, bytes decoded in a data stream, or so
/// on.
pub struct ExecutionLimit {
    /// How many operations remain before the limit is reached.
    ///
    /// If `None`, then an execution limit is not enforced.
    current_oplimit: Option<usize>,
}

impl ExecutionLimit {
    /// Construct an execution limit that allows unlimited execution.
    pub fn none() -> ExecutionLimit {
        Self {
            current_oplimit: None,
        }
    }

    pub fn with_max_ops(ops: usize) -> ExecutionLimit {
        Self {
            current_oplimit: Some(ops),
        }
    }

    /// Create an execution limit that is already exhausted.
    pub fn exhausted() -> ExecutionLimit {
        Self {
            current_oplimit: Some(0),
        }
    }

    /// Check if the execution of a certain number of operations has exceeded
    /// the execution limit.
    ///
    /// This is intended to be called after the given operations have been
    /// executed. The ops will be deducted from the operation limit and, if
    /// that limit has been breached, this returns 'true'
    pub fn did_ops_breach_limit(
        &mut self,
        _context: &mut UpdateContext<'_, '_>,
        ops: usize,
    ) -> bool {
        if let Some(oplimit) = &mut self.current_oplimit {
            *oplimit = oplimit.saturating_sub(ops);
            return *oplimit == 0;
        }

        false
    }
}
