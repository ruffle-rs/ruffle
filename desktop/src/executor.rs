//! Async executor

use crate::custom_event::RuffleEvent;
use crate::task::Task;
use async_channel::{unbounded, Receiver, Sender};
use ruffle_core::backend::navigator::OwnedFuture;
use ruffle_core::loader::Error;
use slotmap::{new_key_type, SlotMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use winit::event_loop::EventLoopProxy;

new_key_type! {
    // This is what we would call `TaskHandle` everywhere else, but that name is already taken.
    struct TaskKey;
}

/// Executor context passed to event sources.
///
/// All task handles are identical and interchangeable. Cloning a `TaskHandle`
/// does not clone the underlying task.
#[derive(Clone)]
struct TaskHandle {
    /// The arena handle for a given task.
    handle: TaskKey,

    /// The executor the task belongs to.
    ///
    /// Weak reference ensures that the executor along
    /// with its tasks is dropped properly.
    executor: Weak<WinitAsyncExecutor>,
}

impl TaskHandle {
    /// Construct a handle to a given task.
    fn for_task(task: TaskKey, executor: Weak<WinitAsyncExecutor>) -> Self {
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
            executor.wake(self.handle);
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
        let selfish = TaskHandle::from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.raw_waker()
    }

    /// Wake the given task, then drop it.
    unsafe fn wake_as_ptr(almost_self: *const ()) {
        let selfish = TaskHandle::box_from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.wake();
    }

    /// Wake the given task.
    unsafe fn wake_by_ref_as_ptr(almost_self: *const ()) {
        let selfish = TaskHandle::from_const_ptr(almost_self).expect("non-null context ptr");

        selfish.wake();
    }

    /// Drop the async executor.
    unsafe fn drop_as_ptr(almost_self: *const ()) {
        let _ = TaskHandle::box_from_const_ptr(almost_self).expect("non-null context ptr");
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_as_ptr,
        Self::wake_as_ptr,
        Self::wake_by_ref_as_ptr,
        Self::drop_as_ptr,
    );
}

pub struct WinitAsyncExecutor {
    /// List of all spawned tasks.
    task_queue: RwLock<SlotMap<TaskKey, Mutex<Task>>>,

    /// Source of tasks sent to us by the `NavigatorBackend`.
    channel: Receiver<OwnedFuture<(), Error>>,

    /// Weak reference to ourselves.
    self_ref: Weak<Self>,

    /// Event injector for the main thread event loop.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// Whether we have already queued a `TaskPoll` event.
    waiting_for_poll: AtomicBool,
}

impl WinitAsyncExecutor {
    /// Construct a new executor for the winit event loop.
    ///
    /// This function returns the executor itself, plus the `Sender` necessary
    /// to spawn new tasks.
    pub fn new(event_loop: EventLoopProxy<RuffleEvent>) -> (Arc<Self>, WinitFutureSpawner) {
        let (send, recv) = unbounded();
        let new_self = Arc::new_cyclic(|self_ref| Self {
            task_queue: RwLock::new(SlotMap::with_key()),
            channel: recv,
            self_ref: self_ref.clone(),
            event_loop: event_loop.clone(),
            waiting_for_poll: AtomicBool::new(false),
        });
        (
            new_self,
            WinitFutureSpawner::send_and_poll(send, event_loop),
        )
    }

    /// Poll all `Ready` futures.
    pub fn poll_all(&self) {
        self.waiting_for_poll.store(false, Ordering::SeqCst);

        self.insert_tasks();

        let mut completed_tasks = vec![];

        for (index, task) in self.read_task_queue().iter() {
            let mut task = task.lock().expect("non-poisoned task");
            if task.is_ready() {
                let handle = TaskHandle::for_task(index, self.self_ref.clone());
                let waker = handle.waker();
                let mut context = Context::from_waker(&waker);

                match task.poll(&mut context) {
                    Poll::Pending => {}
                    Poll::Ready(r) => {
                        if let Err(e) = r {
                            tracing::error!("Async error: {}", e);
                        }

                        completed_tasks.push(index);
                    }
                }
            }
        }

        self.remove_tasks(completed_tasks);
    }

    /// Mark a task as ready to proceed.
    fn wake(&self, task: TaskKey) {
        if let Some(task) = self.read_task_queue().get(task) {
            let mut task = task.lock().expect("non-poisoned task");
            if !task.is_completed() {
                task.set_ready();
                if !self.waiting_for_poll.swap(true, Ordering::SeqCst) {
                    if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
                        tracing::warn!("A task was queued on an event loop that has already ended. It will not be polled.");
                    }
                } else {
                    tracing::info!("Double polling");
                }
            } else {
                tracing::warn!(
                    "A Waker was invoked after the task it was attached to was completed."
                );
            }
        } else {
            tracing::warn!("Attempted to wake an already-finished task");
        }
    }

    fn insert_tasks(&self) {
        let mut queue = self.write_task_queue();
        while let Ok(fut) = self.channel.try_recv() {
            queue.insert(Mutex::new(Task::from_future(fut)));
        }
    }

    fn remove_tasks(&self, completed_tasks: Vec<TaskKey>) {
        let mut queue = self.write_task_queue();
        for index in completed_tasks {
            queue.remove(index);
        }
    }

    fn write_task_queue(&self) -> RwLockWriteGuard<'_, SlotMap<TaskKey, Mutex<Task>>> {
        self.task_queue.write().expect("non-poisoned task queue")
    }

    fn read_task_queue(&self) -> RwLockReadGuard<'_, SlotMap<TaskKey, Mutex<Task>>> {
        self.task_queue.read().expect("non-poisoned task queue")
    }
}

pub trait FutureSpawner {
    fn spawn(&self, future: OwnedFuture<(), Error>);
}

pub struct WinitFutureSpawner {
    channel: Sender<OwnedFuture<(), Error>>,
    event_loop: EventLoopProxy<RuffleEvent>,
}

impl WinitFutureSpawner {
    pub fn send_and_poll(
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> WinitFutureSpawner {
        WinitFutureSpawner {
            channel,
            event_loop,
        }
    }
}

impl FutureSpawner for WinitFutureSpawner {
    fn spawn(&self, future: OwnedFuture<(), Error>) {
        self.channel
            .send_blocking(future)
            .expect("working channel send");
        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            tracing::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }
}
