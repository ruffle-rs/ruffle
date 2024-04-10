//! Task state information

use ruffle_core::backend::navigator::OwnedFuture;
use ruffle_core::loader::Error;
use std::task::{Context, Poll};

/// Indicates the state of a given task.
#[derive(Eq, PartialEq)]
enum TaskState {
    /// Indicates that a task is ready to be polled to make progress.
    Ready,

    /// Indicates that a task is blocked on another event source.
    Blocked,

    /// Indicates that a task is complete and should not be awoken again.
    Completed,
}

/// Wrapper type for futures in our executor.
pub struct Task {
    /// The state of the task.
    state: TaskState,

    /// The future to poll in order to progress the task.
    future: OwnedFuture<(), Error>,
}

impl Task {
    /// Box an owned future into a task structure.
    pub fn from_future(future: OwnedFuture<(), Error>) -> Self {
        Self {
            state: TaskState::Ready,
            future,
        }
    }

    /// Returns `true` if the task is ready to be polled.
    pub fn is_ready(&self) -> bool {
        self.state == TaskState::Ready
    }

    /// Marks this task to as ready to make progress.
    pub fn set_ready(&mut self) {
        self.state = TaskState::Ready
    }

    /// Returns `true` if the task is awaiting further progress.
    #[allow(dead_code)]
    pub fn is_blocked(&self) -> bool {
        self.state == TaskState::Blocked
    }

    /// Returns `true` if the task has completed and should not be polled again.
    pub fn is_completed(&self) -> bool {
        self.state == TaskState::Completed
    }

    /// Poll the underlying future.
    ///
    /// This wrapper function ensures that futures cannot be polled after they
    /// have completed. Future polls will return `Ok(())`.
    pub fn poll(&mut self, context: &mut Context) -> Poll<Result<(), Error>> {
        if self.is_completed() {
            return Poll::Ready(Ok(()));
        }

        let poll = self.future.as_mut().poll(context);

        self.state = match poll {
            Poll::Pending => TaskState::Blocked,
            Poll::Ready(_) => TaskState::Completed,
        };

        poll
    }
}
