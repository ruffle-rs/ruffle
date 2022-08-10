use std::cmp::min;
use std::ops::Deref;

#[derive(Debug)]
pub struct StackFrame<'a, T> {
    stack: &'a mut Vec<T>,
    depth: usize,
    max: usize,
}

impl<'a, T> StackFrame<'a, T> {
    pub fn new(stack: &'a mut Vec<T>, depth: usize, max: usize) -> Self {
        Self { stack, depth, max }
    }

    pub fn clear(&mut self) {
        self.stack.truncate(self.depth);
    }

    pub fn push(&mut self, value: T) {
        if self.len() > self.max {
            log::warn!("Avm2::StackFrame::push overflow");
            return;
        }
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            self.stack.pop()
        } else {
            None
        }
    }
}

impl<'a, T> Deref for StackFrame<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.stack
            .get(self.depth..)
            .and_then(|v| v.get(..min(v.len(), self.max)))
            .expect("StackFrame out of range")
    }
}
