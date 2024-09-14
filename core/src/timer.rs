//! Timer handling for `setInterval`/`setTimeout`/`Timer` AVM timers.
//!
//! We tick the timers during our normal frame loop for deterministic operation.
//! The timers are stored in a priority queue, where we check if the nearest timer
//! is ready to tick each frame.

use crate::avm1::ExecutionReason;
use crate::avm1::{
    Activation, ActivationIdentifier, Object as Avm1Object, TObject as _, Value as Avm1Value,
};
use crate::avm2::object::TObject;
use crate::avm2::{Activation as Avm2Activation, Object as Avm2Object, Value as Avm2Value};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::AvmString;
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
    pub fn update_timers(context: &mut UpdateContext<'gc>, dt: f64) -> Option<f64> {
        context.timers.cur_time = context
            .timers
            .cur_time
            .wrapping_add((dt * Self::TIMER_SCALE) as u64);

        if context.timers.is_empty() {
            return None;
        }

        let level0 = context.stage.root_clip();

        let mut tick_count = 0;

        // We have to be careful because the timer list can be mutated while updating;
        // a timer callback could add more timers, clear timers (including itself), etc.
        while context
            .timers
            .peek()
            .map(|timer| timer.tick_time)
            .unwrap_or(context.timers.cur_time)
            < context.timers.cur_time
        {
            let timer = context.timers.peek().unwrap();

            tick_count += 1;
            // SANITY: Only allow so many ticks per update.
            if tick_count > Self::MAX_TICKS {
                // Reset our time to a little bit before the nearest timer.
                context.timers.cur_time = timer.tick_time.wrapping_sub(100);
                break;
            }

            // TODO: Can we avoid these clones?
            let callback = timer.callback.clone();
            let expected_id = timer.id;

            let cancel_timer = match callback {
                TimerCallback::Avm1Function { func, params } => {
                    if let Some(level0) = level0 {
                        let mut avm1_activation = Activation::from_nothing(
                            context,
                            ActivationIdentifier::root("[Timer Callback]"),
                            level0,
                        );
                        let result = func.call(
                            "[Timer Callback]".into(),
                            &mut avm1_activation,
                            Avm1Value::Undefined,
                            &params,
                        );

                        if let Err(e) = result {
                            tracing::error!("Unhandled AVM1 error in timer callback: {}", e);
                        }
                    } else {
                        tracing::warn!("Skipping AVM1 timer as there's no root");
                    }

                    false
                }
                TimerCallback::Avm1Method {
                    this,
                    method_name,
                    params,
                } => {
                    // If you add a timer onto a MovieClip and then remove the clip
                    // The timer should stop firing and be canceled
                    // Because we store an object and not a value, we have to check for a clip like this

                    let mut removed = false;

                    // We can't use as_display_object + as_movie_clip here as we explicitly don't want to convert `SuperObjects`
                    if let Avm1Object::StageObject(s) = this {
                        let d_o = s.as_display_object().unwrap();
                        if let DisplayObject::MovieClip(mc) = d_o {
                            // Note that we don't want to fire the timer here
                            if mc.avm1_removed() {
                                removed = true;
                            }
                        }
                    }

                    if !removed {
                        if let Some(level0) = level0 {
                            let mut avm1_activation = Activation::from_nothing(
                                context,
                                ActivationIdentifier::root("[Timer Callback]"),
                                level0,
                            );
                            let result = this.call_method(
                                method_name,
                                &params,
                                &mut avm1_activation,
                                ExecutionReason::Special,
                            );

                            if let Err(e) = result {
                                tracing::error!("Unhandled AVM1 error in timer callback: {}", e);
                            }
                        } else {
                            tracing::warn!("Skipping AVM1 timer as there's no root");
                        }

                        false
                    } else {
                        true
                    }
                }
                TimerCallback::Avm2Callback { closure, params } => {
                    let domain = context.avm2.stage_domain();
                    let mut avm2_activation = Avm2Activation::from_domain(context, domain);
                    match closure.call(Avm2Value::Null, &params, &mut avm2_activation) {
                        Ok(v) => v.coerce_to_boolean(),
                        Err(e) => {
                            tracing::error!("Unhandled AVM2 error in timer callback: {e:?}",);
                            false
                        }
                    }
                }
            };

            crate::player::Player::run_actions(context);

            if context.timers.is_empty() {
                // Every last timer was cleared during the callback, including
                // the one that just ticked, so there is nothing left to do.
                break;
            }

            // We need a mutable reference to the timer for the rest.
            // Just checked that there is still at least one timer active, so unwrap must succeed.
            let mut timer = context.timers.peek_mut().unwrap();

            if timer.id == expected_id {
                // The timer next in line is still the same, so we need to remove or reset it.
                if timer.is_timeout || cancel_timer {
                    // Timeouts only fire once, and a false return value from the callback cancels intervals.
                    drop(timer);
                    context.timers.pop();
                } else {
                    // Reset setInterval timers. `peek_mut` re-sorts the timer in the priority queue.
                    timer.tick_time = timer.tick_time.wrapping_add(timer.interval);
                }
            } else {
                drop(timer);
                // The timer is no longer the next in line, and this can only happen if
                // it cancelled itself manually, or it set a new timer in the past.
                // Let's make sure it's the former.
                debug_assert!(!context.timers.timer_exists(expected_id));
            }
        }

        // Return estimated time until next timer tick.
        context.timers.peek().map(|timer| {
            (timer.tick_time.wrapping_sub(context.timers.cur_time)) as f64 / Self::TIMER_SCALE
        })
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

    /// Whether there are no timers active.
    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    /// Whether a timer with the given ID exists.
    pub fn timer_exists(&self, id: i32) -> bool {
        self.timers.iter().any(|t| t.id == id)
    }

    /// Registers a new timer and returns the timer ID.
    pub fn add_timer(
        &mut self,
        callback: TimerCallback<'gc>,
        interval: i32,
        is_timeout: bool,
    ) -> i32 {
        // SANITY: Set a minimum interval so we don't spam too much.
        let interval = interval.max(Self::MIN_INTERVAL) as u64 * (Self::TIMER_SCALE as u64);

        // The counter has to be incremented first, because Flash Player never assigns timer ID 0,
        // and neither should we, as some content uses it as an initial "not yet set up" value.
        self.timer_counter = self.timer_counter.wrapping_add(1);
        let id = self.timer_counter;
        let timer = Timer {
            id,
            callback,
            tick_time: self.cur_time + interval,
            interval,
            is_timeout,
        };
        self.timers.push(timer);
        id
    }

    /// Removes a timer.
    pub fn remove(&mut self, id: i32) -> bool {
        let old_len = self.timers.len();
        self.timers.retain(|t| t.id != id);
        let len = self.timers.len();
        // Sanity check: Either we removed a single timer, or none.
        debug_assert!(len == old_len || len == old_len - 1);
        len < old_len
    }

    pub fn remove_all(&mut self) {
        self.timers.clear()
    }

    /// Changes the delay of a timer.
    pub fn set_delay(&mut self, id: i32, interval: i32) {
        // SANITY: Set a minimum interval so we don't spam too much.
        let interval = interval.max(Self::MIN_INTERVAL) as u64 * (Self::TIMER_SCALE as u64);

        // Due to the limitations of `BinaryHeap`, we have to do this in a slightly roundabout way.
        let mut timer = None;
        for t in self.timers.iter() {
            if t.id == id {
                timer = Some(t.clone());
                break;
            }
        }

        if let Some(mut timer) = timer {
            self.remove(id);
            timer.interval = interval;
            timer.tick_time = self.cur_time + interval;
            self.timers.push(timer);
        } else {
            panic!("Changing delay of non-existent timer");
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
    fn trace(&self, cc: &gc_arena::Collection) {
        for timer in &self.timers {
            timer.trace(cc);
        }
    }
}
/// A timer created via `setInterval`/`setTimeout`.
/// Runs a callback when it ticks.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Timer<'gc> {
    /// The ID of the timer.
    id: i32,

    /// The callback that this timer runs when it fires.
    /// A callback is either a function object, or a parent object with a method name.
    callback: TimerCallback<'gc>,

    /// The time when this timer should fire.
    tick_time: u64,

    /// The interval between timer ticks, in microseconds.
    interval: u64,

    /// This timer only fires once if `is_timeout` is true.
    is_timeout: bool,
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
        Some(self.cmp(other))
    }
}

impl Ord for Timer<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.tick_time.cmp(&other.tick_time).reverse()
    }
}

/// A callback fired by a `setInterval`/`setTimeout` timer.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum TimerCallback<'gc> {
    Avm1Function {
        func: Avm1Object<'gc>,
        /// The parameters to pass to the callback function.
        params: Vec<Avm1Value<'gc>>,
    },

    Avm1Method {
        this: Avm1Object<'gc>,
        method_name: AvmString<'gc>,
        params: Vec<Avm1Value<'gc>>,
    },

    Avm2Callback {
        closure: Avm2Object<'gc>,
        params: Vec<Avm2Value<'gc>>,
    },
}
