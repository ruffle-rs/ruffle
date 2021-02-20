use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use gc_arena::{Collect, MutationContext};
use std::collections::BTreeMap;

pub type LevelId = u32;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
struct Level<'gc> {
    root: DisplayObject<'gc>,
    last_child: DisplayObject<'gc>,
}

#[derive(Clone, Collect, Default)]
#[collect(no_drop)]
pub struct LevelsData<'gc>(BTreeMap<LevelId, Level<'gc>>);

impl<'gc> LevelsData<'gc> {
    pub fn insert(&mut self, gc_context: MutationContext<'gc, '_>, root: DisplayObject<'gc>) {
        if let Some(level0) = self.0.get(&0) {
            let prev = level0.root;
            if let Some(next) = prev.next_exec() {
                root.set_next_exec(gc_context, Some(next));
                next.set_prev_exec(gc_context, Some(root));
            }
            root.set_prev_exec(gc_context, Some(prev));
            prev.set_next_exec(gc_context, Some(root));
        }
        self.0.insert(root.level_id(), Level { root, last_child: root });
    }

    pub fn remove(&mut self, gc_context: MutationContext<'gc, '_>, id: LevelId) {
        if let Some(level) = self.0.remove(&id) {
            let prev = level.last_child.prev_exec();
            let next = level.root.next_exec();

            if let Some(prev) = prev {
                prev.set_next_exec(gc_context, next);
            }
            if let Some(next) = next {
                next.set_prev_exec(gc_context, prev);
            }

            level.last_child.set_prev_exec(gc_context, None);
            level.root.set_next_exec(gc_context, None);
        }
    }

    pub fn add_to_execution_list(&mut self, gc_context: MutationContext<'gc, '_>, node: DisplayObject<'gc>) {
        if let Some(level) = self.0.get_mut(&node.level_id()) {
            let head = level.last_child;
            if let Some(prev) = head.prev_exec() {
                prev.set_next_exec(gc_context, Some(node));
                node.set_prev_exec(gc_context, Some(prev));
            }
            head.set_prev_exec(gc_context, Some(node));
            node.set_next_exec(gc_context, Some(head));
            level.last_child = node;
        } else {
            self.insert(gc_context, node);
        }
    }

    pub fn remove_from_execution_list(&mut self, gc_context: MutationContext<'gc, '_>, node: DisplayObject<'gc>) {
        if let Some(ctr) = node.as_container() {
            for child in ctr.iter_render_list() {
                self.remove_from_execution_list(gc_context, child);
            }
        }

        let prev = node.prev_exec();
        let next = node.next_exec();

        if let Some(prev) = prev {
            prev.set_next_exec(gc_context, next);
        }
        if let Some(next) = next {
            next.set_prev_exec(gc_context, prev);
        }

        node.set_prev_exec(gc_context, None);
        node.set_next_exec(gc_context, None);

        if let Some(level) = self.0.get_mut(&node.level_id()) {
            if DisplayObject::ptr_eq(level.last_child, node) {
                // TODO: can we safely unwrap here?
                level.last_child = next.unwrap();
            }
        }
    }

    // TODO: impl Index?
    pub fn at(&self, id: LevelId) -> Option<DisplayObject<'gc>> {
        self.0.get(&id).map(|level| level.root)
    }

    pub fn iter_roots<'a>(&'a self) -> impl 'a + Iterator<Item = DisplayObject<'gc>> {
        self.0.values().map(|level| level.root)
    }

    pub fn iter_exec(&self) -> ExecIter<'gc> {
        ExecIter {
            head: self.0.get(&0).map(|level0| level0.last_child),
        }
    }
}

pub struct ExecIter<'gc> {
    head: Option<DisplayObject<'gc>>,
}

impl<'gc> Iterator for ExecIter<'gc> {
    type Item = DisplayObject<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.head;
        self.head = ret.and_then(|x| x.next_exec());
        ret
    }
}
