//! Async executor

use crate::custom_event::RuffleEvent;
use crate::task::Task;
use generational_arena::{Arena, Index};
use ruffle_core::backend::navigator::OwnedFuture;
use ruffle_core::loader::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use winit::event_loop::EventLoopProxy;

/// Exeuctor context passed to event sources.
///
/// All task handles are identical and interchangeable. Cloning a `TaskHandle`
/// does not clone the underlying task.
#[derive(Clone)]
struct TaskHandle {
    /// The arena handle for a given task.
    handle: Index,

    /// The executor the task belongs to.
    executor: Arc<Mutex<GlutinAsyncExecutor>>,
}

impl TaskHandle {
    /// Construct a handle to a given task.
    fn for_task(task: Index, executor: Arc<Mutex<GlutinAsyncExecutor>>) -> Self {
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
        self.executor
            .lock()
            .expect("able to lock executor")
            .wake(self.handle);
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

pub struct GlutinAsyncExecutor {
    /// List of all spawned tasks.
    task_queue: Arena<Task>,

    /// Source of tasks sent to us by the `NavigatorBackend`.
    channel: Receiver<OwnedFuture<(), Error>>,

    /// Weak reference to ourselves.
    self_ref: Weak<Mutex<Self>>,

    /// Event injector for the main thread event loop.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// Whether or not we have already queued a `TaskPoll` event.
    waiting_for_poll: bool,
}

impl GlutinAsyncExecutor {
    /// Construct a new executor for the Glutin event loop.
    ///
    /// This function returns the executor itself, plus the `Sender` necessary
    /// to spawn new tasks.
    pub fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> (Arc<Mutex<Self>>, Sender<OwnedFuture<(), Error>>) {
        let (send, recv) = channel();
        let new_self = Arc::new_cyclic(|self_ref| {
            Mutex::new(Self {
                task_queue: Arena::new(),
                channel: recv,
                self_ref: self_ref.clone(),
                event_loop,
                waiting_for_poll: false,
            })
        });
        (new_self, send)
    }

    /// Poll all `Ready` futures.
    pub fn poll_all(&mut self) {
        self.waiting_for_poll = false;

        while let Ok(fut) = self.channel.try_recv() {
            self.task_queue.insert(Task::from_future(fut));
        }

        let self_ref = self.self_ref.upgrade().expect("active self-reference");
        let mut completed_tasks = vec![];

        for (index, task) in self.task_queue.iter_mut() {
            if task.is_ready() {
                let handle = TaskHandle::for_task(index, self_ref.clone());
                let waker = handle.waker();
                let mut context = Context::from_waker(&waker);

                match task.poll(&mut context) {
                    Poll::Pending => {}
                    Poll::Ready(r) => {
                        if let Err(e) = r {
                            log::error!("Async error: {}", e);
                        }

                        completed_tasks.push(index);
                    }
                }
            }
        }

        for index in completed_tasks {
            self.task_queue.remove(index);
        }
    }

    /// Mark a task as ready to proceed.
    fn wake(&mut self, task: Index) {
        if let Some(task) = self.task_queue.get_mut(task) {
            if !task.is_completed() {
                if !self.waiting_for_poll {
                    self.waiting_for_poll = true;
                    task.set_ready();
                    if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
                        log::warn!("A task was queued on an event loop that has already ended. It will not be polled.");
                    }
                } else {
                    log::info!("Double polling");
                }
            } else {
                log::warn!("A Waker was invoked after the task it was attached to was completed.");
            }
        } else {
            log::warn!("Attempted to wake an already-finished task");
        }
    }
}
