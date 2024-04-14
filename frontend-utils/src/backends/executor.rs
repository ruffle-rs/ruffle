mod task;

pub use crate::backends::executor::task::Task;
use async_channel::{unbounded, Receiver, Sender};
use ruffle_core::backend::navigator::OwnedFuture;
use ruffle_core::loader::Error;
use slotmap::{new_key_type, SlotMap};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub trait PollRequester: Clone {
    fn request_poll(&self);
}

new_key_type! {
    // This is what we would call `TaskHandle` everywhere else, but that name is already taken.
    struct TaskKey;
}

/// Executor context passed to event sources.
///
/// All task handles are identical and interchangeable. Cloning a `TaskHandle`
/// does not clone the underlying task.
#[derive(Clone)]
struct TaskHandle<R: PollRequester> {
    /// The arena handle for a given task.
    handle: TaskKey,

    /// The executor the task belongs to.
    ///
    /// Weak reference ensures that the executor along
    /// with its tasks is dropped properly.
    executor: Weak<AsyncExecutor<R>>,
}

impl<R: PollRequester> TaskHandle<R> {
    /// Construct a handle to a given task.
    fn for_task(task: TaskKey, executor: Weak<AsyncExecutor<R>>) -> Self {
        Self {
            handle: task,
            executor,
        }
    }

    /// Construct a new `RawWaker` for this task handle.
    ///
    /// This function clones the underlying task handle.
    fn raw_waker(&self) -> RawWaker {
        let clone = Box::new(self.clone());
        RawWaker::new(Box::into_raw(clone) as *const (), &Self::VTABLE)
    }

    /// Construct a new waker for this task handle.
    fn waker(&self) -> Waker {
        unsafe { Waker::from_raw(self.raw_waker()) }
    }

    /// Wake the task this context refers to.
    fn wake(&self) {
        if let Some(executor) = self.executor.upgrade() {
            executor.wake(self.handle, true);
        }
    }

    /// Convert a voidptr into an `TaskHandle` reference, if non-null.
    ///
    /// This function is unsafe because the pointer can refer to any resource
    /// in memory. It also can belong to any lifetime. Use of this function on
    /// a pointer *not* ultimately derived from an TaskHandle in memory
    /// constitutes undefined behavior.
    unsafe fn from_const_ptr<'a>(almost_self: *const ()) -> Option<&'a Self> {
        if almost_self.is_null() {
            return None;
        }

        Some(&*(almost_self as *const Self))
    }

    /// Convert a voidptr into a mutable `TaskHandle` reference, if
    /// non-null.
    ///
    /// This function is unsafe because the pointer can refer to any resource
    /// in memory. It also can belong to any lifetime. Use of this function on
    /// a pointer *not* ultimately derived from an TaskHandle in memory
    /// constitutes undefined behavior.
    ///
    /// It's also additionally unsound to call this function while other
    /// references to the same `TaskHandle` exist.
    unsafe fn box_from_const_ptr(almost_self: *const ()) -> Option<Box<Self>> {
        if almost_self.is_null() {
            return None;
        }

        Some(Box::from_raw(almost_self as *mut Self))
    }

    /// Construct a new `RawWaker` that wakes the same task.
    ///
    /// This is part of the vtable methods of our `RawWaker` impl.
    unsafe fn clone_as_ptr(almost_self: *const ()) -> RawWaker {
        let selfish = TaskHandle::<R>::from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.raw_waker()
    }

    /// Wake the given task, then drop it.
    unsafe fn wake_as_ptr(almost_self: *const ()) {
        let selfish =
            TaskHandle::<R>::box_from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.wake();
    }

    /// Wake the given task.
    unsafe fn wake_by_ref_as_ptr(almost_self: *const ()) {
        let selfish = TaskHandle::<R>::from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.wake();
    }

    /// Drop the async executor.
    unsafe fn drop_as_ptr(almost_self: *const ()) {
        let _ = TaskHandle::<R>::box_from_const_ptr(almost_self).expect("non-null context ptr");
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_as_ptr,
        Self::wake_as_ptr,
        Self::wake_by_ref_as_ptr,
        Self::drop_as_ptr,
    );
}

pub struct AsyncExecutor<R: PollRequester> {
    /// List of all spawned tasks.
    tasks: Mutex<SlotMap<TaskKey, Task>>,

    /// Task execution queue represented by a channel.
    ///
    /// In order to wake a task it should be sent to this channel,
    /// whereas reading from this channel is used for task polling.
    task_queue: (Sender<TaskKey>, Receiver<TaskKey>),

    /// Source of tasks sent to us by the `NavigatorBackend`.
    task_spawner: Receiver<OwnedFuture<(), Error>>,

    /// Weak reference to ourselves.
    self_ref: Weak<Self>,

    /// Callback to inform main thread that this executor is ready for polling
    poll_requester: R,

    /// Whether we have already queued a `TaskPoll` event.
    waiting_for_poll: AtomicBool,
}

impl<R: PollRequester> AsyncExecutor<R> {
    /// Construct a new executor for the given poll requester.
    ///
    /// This function returns the executor itself, plus the `Sender` necessary
    /// to spawn new tasks.
    pub fn new(poll_requester: R) -> (Arc<Self>, AsyncFutureSpawner<R>) {
        let (send, recv) = unbounded();
        let new_self = Arc::new_cyclic(|self_ref| Self {
            tasks: Mutex::new(SlotMap::with_key()),
            task_queue: unbounded(),
            task_spawner: recv,
            self_ref: self_ref.clone(),
            poll_requester: poll_requester.clone(),
            waiting_for_poll: AtomicBool::new(false),
        });
        (
            new_self,
            AsyncFutureSpawner::send_and_poll(send, poll_requester),
        )
    }

    /// Poll all ready tasks.
    pub fn poll_all(&self) {
        self.waiting_for_poll.store(false, Ordering::SeqCst);

        let mut tasks = self.tasks.lock().expect("non-poisoned tasks");
        self.insert_tasks(&mut tasks);

        // We want only to poll as many tasks as there were at the beginning.
        // Newly added tasks will be polled later at the next iteration.
        let tasks_to_poll = self.task_queue.1.len();

        for _ in 0..tasks_to_poll {
            let Ok(key) = self.task_queue.1.try_recv() else {
                break;
            };

            let Some(task) = tasks.get_mut(key) else {
                // Tried to wake a nonexistent task.
                continue;
            };

            if task.is_completed() {
                // Tried to wake a completed task.
                tasks.remove(key);
                continue;
            }

            let handle = TaskHandle::for_task(key, self.self_ref.clone());
            let waker = handle.waker();
            let mut context = Context::from_waker(&waker);

            match task.poll(&mut context) {
                Poll::Pending => {}
                Poll::Ready(r) => {
                    if let Err(e) = r {
                        tracing::error!("Async error: {}", e);
                    }

                    tasks.remove(key);
                }
            }
        }
    }

    /// Mark a task as ready to proceed.
    fn wake(&self, task: TaskKey, poll: bool) {
        self.task_queue.0.try_send(task).expect("wake a task");
        if poll && !self.waiting_for_poll.swap(true, Ordering::SeqCst) {
            self.poll_requester.request_poll();
        }
    }

    fn insert_tasks(&self, tasks: &mut MutexGuard<SlotMap<TaskKey, Task>>) {
        while let Ok(fut) = self.task_spawner.try_recv() {
            let key = tasks.insert(Task::from_future(fut));

            // Start executing the newly added task by waking it.
            // We do not poll here, as we are inserting tasks during a poll already.
            self.wake(key, false);
        }
    }
}

pub trait FutureSpawner {
    fn spawn(&self, future: OwnedFuture<(), Error>);
}

pub struct AsyncFutureSpawner<R: PollRequester> {
    channel: Sender<OwnedFuture<(), Error>>,
    poll_requester: R,
}

impl<R: PollRequester> AsyncFutureSpawner<R> {
    pub fn send_and_poll(
        channel: Sender<OwnedFuture<(), Error>>,
        poll_requester: R,
    ) -> AsyncFutureSpawner<R> {
        AsyncFutureSpawner {
            channel,
            poll_requester,
        }
    }
}

impl<R: PollRequester> FutureSpawner for AsyncFutureSpawner<R> {
    fn spawn(&self, future: OwnedFuture<(), Error>) {
        self.channel
            .send_blocking(future)
            .expect("working channel send");
        self.poll_requester.request_poll()
    }
}

/// Spawns a new asynchronous task in a tokio runtime, without the current executor needing to belong to tokio
pub async fn spawn_tokio<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move { sender.send(future.await) });
    tokio::task::unconstrained(receiver)
        .await
        .expect("Oneshot should succeed")
}
