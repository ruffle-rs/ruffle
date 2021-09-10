//! Parallel-to-serial iterator bridge

use rayon::prelude::*;

/// Parallel-to-serial iterator bridge trait
///
/// Proposed in and copied from https://github.com/rayon-rs/rayon/issues/858
pub trait SerBridge<T>
where
    T: Send + 'static,
    Self: ParallelIterator<Item = T> + 'static,
{
    fn ser_bridge(self) -> SerBridgeImpl<T> {
        SerBridgeImpl::new(self)
    }
}

impl<PI, T> SerBridge<T> for PI
where
    T: Send + 'static,
    PI: ParallelIterator<Item = T> + 'static,
{
}

/// Parallel-to-serial iterator bridge
///
/// Proposed in and copied from https://github.com/rayon-rs/rayon/issues/858
pub struct SerBridgeImpl<T> {
    rx: crossbeam_channel::Receiver<T>,
}

impl<T: Send + 'static> SerBridgeImpl<T> {
    pub fn new<PI>(par_iterable: impl IntoParallelIterator<Item = T, Iter = PI>) -> Self
    where
        PI: ParallelIterator<Item = T> + 'static,
    {
        let par_iter = par_iterable.into_par_iter();
        let (tx, rx) = crossbeam_channel::bounded(0);
        std::thread::spawn(move || {
            let _ = par_iter.try_for_each(|item| tx.send(item));
        });
        SerBridgeImpl { rx }
    }
}

impl<T> Iterator for SerBridgeImpl<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.rx.recv().ok()
    }
}
