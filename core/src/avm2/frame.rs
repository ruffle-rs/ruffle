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
    pub fn len(&self) -> usize {
        self.stack.len() - self.depth
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.stack.truncate(self.depth);
    }

    pub fn push(&mut self, value: T) {
        // TODO: Currently we push anyways when the stack frame exceeds its max size.
        // This is fine because the worst that can happen is the stack reallocates, but it might
        // be better to return an error.
        if self.len() > self.max {
            log::warn!("Avm2::StackFrame::push overflow");
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
