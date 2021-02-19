use crate::display_object::{DisplayObject, TDisplayObject};
use gc_arena::{Collect, MutationContext};
use std::collections::BTreeMap;

pub type LevelId = u32;

#[derive(Clone, Collect, Default)]
#[collect(no_drop)]
pub struct LevelsData<'gc>(BTreeMap<LevelId, Level<'gc>>);

impl<'gc> LevelsData<'gc> {
    pub fn insert(&mut self, gc_context: MutationContext<'gc, '_>, level: Level<'gc>) {
        if let Some(level0) = self.get(0) {
            let prev = level0.root();
            if let Some(next) = prev.next_exec() {
                level.root().set_next_exec(gc_context, Some(next));
                next.set_prev_exec(gc_context, Some(level.root()));
            }
            level.last_child().set_prev_exec(gc_context, Some(prev));
            prev.set_next_exec(gc_context, Some(level.last_child()));
        }
        self.0.insert(level.id(), level);
    }

    pub fn remove(&mut self, gc_context: MutationContext<'gc, '_>, id: LevelId) {
        if let Some(level) = self.0.remove(&id) {
            let prev = level.last_child().prev_exec();
            let next = level.root().next_exec();

            if let Some(prev) = prev {
                prev.set_next_exec(gc_context, next);
            }
            if let Some(next) = next {
                next.set_prev_exec(gc_context, prev);
            }

            level.last_child().set_prev_exec(gc_context, None);
            level.root().set_next_exec(gc_context, None);
        }
    }

    pub fn get(&self, id: LevelId) -> Option<&Level<'gc>> {
        self.0.get(&id)
    }

    pub fn get_mut(&mut self, id: LevelId) -> Option<&mut Level<'gc>> {
        self.0.get_mut(&id)
    }

    // TODO: impl Index?
    pub fn at(&self, id: LevelId) -> Option<DisplayObject<'gc>> {
        self.get(id).map(|level| level.root())
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Level<'gc>> {
        self.0.values()
    }

    pub fn iter_exec(&self) -> ExecIter<'gc> {
        ExecIter {
            head: self.get(0).map(|level0| level0.last_child()),
        }
    }
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Level<'gc> {
    root: DisplayObject<'gc>,
    last_child: DisplayObject<'gc>,
}

impl<'gc> Level<'gc> {
    pub fn new(root: DisplayObject<'gc>) -> Self {
        Self {
            root,
            last_child: root,
        }
    }

    pub fn root(&self) -> DisplayObject<'gc> {
        self.root
    }

    pub fn id(&self) -> LevelId {
        self.root.level_id()
    }

    pub fn last_child(&self) -> DisplayObject<'gc> {
        self.last_child
    }

    pub fn set_last_child(&mut self, child: DisplayObject<'gc>) {
        self.last_child = child;
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
