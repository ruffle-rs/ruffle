//! Timer handling for `setInterval` AVM timers.
//!
//! We tick the timers during our normal frame loop for deterministic operation.
//! The timers are stored in a priority queue, where we check if the nearest timer
//! is ready to tick each frame.
//!
//! TODO: Could we use this for AVM2 timers as well?

use crate::avm1::{Activation, ActivationIdentifier, AvmString, Object, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::Collect;
use std::collections::{binary_heap::PeekMut, BinaryHeap};

/// Manages the collection of timers.
pub struct Timers<'gc> {
    /// The collection of active timers.
    timers: BinaryHeap<Timer<'gc>>,

    /// An increasing ID used for created timers.
    timer_counter: i32,

    /// The current global time.
    cur_time: u64,
}

impl<'gc> Timers<'gc> {
    /// Ticks all timers and runs necessary callbacks.
    pub fn update_timers(context: &mut UpdateContext<'_, 'gc, '_>, dt: f64) -> Option<f64> {
        context.timers.cur_time = context
            .timers
            .cur_time
            .wrapping_add((dt * Self::TIMER_SCALE) as u64);
        let num_timers = context.timers.num_timers();

        if num_timers == 0 {
            return None;
        }

        let version = context.swf.version();
        let globals = context.avm1.global_object_cell();
        let level0 = context.stage.root_clip();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Timer Callback]"),
            version,
            globals,
            level0,
        );

        let mut tick_count = 0;
        let cur_time = activation.context.timers.cur_time;

        // We have to be careful because the timer list can be mutated while updating;
        // a timer callback could add more timers, clear timers, etc.
        while activation
            .context
            .timers
            .peek()
            .map(|timer| timer.tick_time)
            .unwrap_or(cur_time)
            < cur_time
        {
            let timer = activation.context.timers.peek().unwrap();

            // TODO: This is only really necessary because BinaryHeap lacks `remove` or `retain` on stable.
            // We can remove the timers straight away in `clearInterval` once this is stable.
            if !timer.is_alive.get() {
                activation.context.timers.pop();
                continue;
            }

            tick_count += 1;
            // SANITY: Only allow so many ticks per timer per update.
            if tick_count > Self::MAX_TICKS {
                // Reset our time to a little bit before the nearest timer.
                let next_time = activation.context.timers.peek_mut().unwrap().tick_time;
                activation.context.timers.cur_time = next_time.wrapping_sub(100);
                break;
            }

            // TODO: Can we avoid these clones?
            let params = timer.params.clone();
            let callback = timer.callback.clone();

            match callback {
                TimerCallback::Function(function) => {
                    let _ = function.call(
                        "[Timer Callback]".into(),
                        &mut activation,
                        Value::Undefined,
                        &params,
                    );
                }
                TimerCallback::Method { this, method_name } => {
                    let _ = this.call_method(method_name, &params, &mut activation);
                }
            }

            crate::player::Player::run_actions(&mut activation.context);

            let mut timer = activation.context.timers.peek_mut().unwrap();
            if timer.is_timeout {
                // Timeouts only fire once.
                drop(timer);
                activation.context.timers.pop();
            } else {
                // Reset setInterval timers. `peek_mut` re-sorts the timer in the priority queue.
                timer.tick_time = timer.tick_time.wrapping_add(timer.interval);
            }
        }

        // Return estimated time until next timer tick.
        activation
            .context
            .timers
            .peek()
            .map(|timer| (timer.tick_time.wrapping_sub(cur_time)) as f64 / Self::TIMER_SCALE)
    }

    /// The minimum interval we allow for timers.
    const MIN_INTERVAL: i32 = 10;

    /// The maximum timer ticks per call to `update_ticks`, for sanity.
    const MAX_TICKS: i32 = 10;

    /// The scale of the timers (microseconds).
    const TIMER_SCALE: f64 = 1000.0;

    /// Creates a new `Timers` collection.
    pub fn new() -> Self {
        Self {
            timers: Default::default(),
            timer_counter: 0,
            cur_time: 0,
        }
    }

    /// The number of timers currently active.
    pub fn num_timers(&self) -> usize {
        self.timers.len()
    }

    /// Registers a new timer and returns the timer ID.
    pub fn add_timer(
        &mut self,
        callback: TimerCallback<'gc>,
        interval: i32,
        params: Vec<Value<'gc>>,
        is_timeout: bool,
    ) -> i32 {
        // SANITY: Set a minimum interval so we don't spam too much.
        let interval = interval.max(Self::MIN_INTERVAL) as u64 * (Self::TIMER_SCALE as u64);

        self.timer_counter = self.timer_counter.wrapping_add(1);
        let id = self.timer_counter;
        let timer = Timer {
            id,
            callback,
            params,
            tick_time: self.cur_time + interval,
            interval,
            is_timeout,
            is_alive: std::cell::Cell::new(true),
        };
        self.timers.push(timer);
        id
    }

    /// Removes a timer.
    pub fn remove(&mut self, id: i32) -> bool {
        // TODO: When `BinaryHeap::remove` is stable, we can remove it here directly.
        if let Some(timer) = self.timers.iter().find(|timer| timer.id == id) {
            timer.is_alive.set(false);
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&Timer<'gc>> {
        self.timers.peek()
    }

    fn peek_mut(&mut self) -> Option<PeekMut<'_, Timer<'gc>>> {
        self.timers.peek_mut()
    }

    fn pop(&mut self) -> Option<Timer<'gc>> {
        self.timers.pop()
    }
}

impl Default for Timers<'_> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'gc> Collect for Timers<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for timer in &self.timers {
            timer.trace(cc);
        }
    }
}
/// A timer created via `setInterval`/`setTimeout`.
/// Runs a callback when it ticks.
#[derive(Debug, Collect)]
#[collect(no_drop)]
struct Timer<'gc> {
    /// The ID of the timer.
    id: i32,

    /// The callback that this timer runs when it fires.
    /// A callback is either a function object, or a parent object with a method name.
    callback: TimerCallback<'gc>,

    /// The parameters to pass to the callback function.
    params: Vec<Value<'gc>>,

    /// The time when this timer should fire.
    tick_time: u64,

    /// The interval between timer ticks, in microseconds.
    interval: u64,

    /// This timer only fires once if `is_timeout` is true.
    is_timeout: bool,

    /// Whether this timer has been removed.
    is_alive: std::cell::Cell<bool>,
}

// Implement `Ord` so that timers can be stored in the BinaryHeap (as a min-heap).
impl PartialEq for Timer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.tick_time == other.tick_time
    }
}

impl Eq for Timer<'_> {}

impl PartialOrd for Timer<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tick_time
            .partial_cmp(&other.tick_time)
            .map(|o| o.reverse())
    }
}

impl Ord for Timer<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tick_time.cmp(&other.tick_time).reverse()
    }
}

/// A callback fired by a `setInterval`/`setTimeout` timer.
#[derive(Debug, Collect, Clone)]
#[collect(no_drop)]
pub enum TimerCallback<'gc> {
    Function(Object<'gc>),
    Method {
        this: Object<'gc>,
        method_name: AvmString<'gc>,
    },
}
