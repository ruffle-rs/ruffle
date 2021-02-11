use gc_arena::{Collect, MutationContext};
use crate::display_object::{DisplayObject, TDisplayObject};
use std::collections::BTreeMap;

pub type LevelId = u32;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct LevelsData<'gc> {
    levels: BTreeMap<LevelId, Level<'gc>>,
}

impl<'gc> LevelsData<'gc> {
    pub fn new() -> Self {
        Self { levels: BTreeMap::new() }
    }

    pub fn push(&mut self, gc_context: MutationContext<'gc, '_>, level: &mut Level<'gc>) {
        self.remove(gc_context, level.id());
        let level0 = self.get_mut(0);
        if let Some(level0) = level0 {
            if let Some(next) = level0.next_level_id().and_then(|id| self.get_mut(id)) {
                next.set_prev_level(gc_context, Some(*level));
                level.set_next_level(gc_context, Some(*next));
            }

            level0.set_next_level(gc_context, Some(*level));
            level.set_prev_level(gc_context, Some(*level0));
        }

        self.levels.insert(level.id(), *level);
    }

    pub fn remove(&mut self, gc_context: MutationContext<'gc, '_>, id: LevelId) {
        if let Some(level) = self.get(id) {
            let prev = level.prev_level_id().and_then(|id| self.get_mut(id));
            let next = level.next_level_id().and_then(|id| self.get_mut(id));

            // TODO: cleanup
            if let Some(prev) = prev {
                let next = level.next_level_id().and_then(|id| self.get(id));
                prev.set_next_level(gc_context, next);
            }
            if let Some(next) = next {
                let prev = level.prev_level_id().and_then(|id| self.get(id));
                next.set_prev_level(gc_context, prev);
            }

            level.set_prev_level(gc_context, None);
            level.set_next_level(gc_context, None);
        }
    }

    pub fn get(&self, id: LevelId) -> Option<Level<'gc>> {
        self.levels.get(&id).copied()
    }

    pub fn get_mut(&mut self, id: LevelId) -> Option<&mut Level<'gc>> {
        self.levels.get_mut(&id)
    }

    pub fn level_at(&self, id: LevelId) -> Option<DisplayObject<'gc>> {
        self.get(id).map(|level| level.root())
    }

    pub fn iter(&self) -> LevelIter<'gc> {
        LevelIter { head: self.get(0) }
    }

    pub fn exec_iter(&self) -> ExecIter<'gc> {
        ExecIter { head: Some(self.get(0).unwrap().last_child()) }
    }
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct Level<'gc> {
    root: DisplayObject<'gc>,
    last_child: DisplayObject<'gc>,
    prev_level_id: Option<LevelId>,
    next_level_id: Option<LevelId>,
}

impl<'gc> Level<'gc> {
    pub fn new(root: DisplayObject<'gc>) -> Self {
        Self {
            root,
            last_child: root,
            prev_level_id: None,
            next_level_id: None,
        }
    }

    pub fn root(&self) -> DisplayObject<'gc> {
        self.root
    }

    fn set_root(&mut self, root: DisplayObject<'gc>) {
        self.root = root;
    }

    pub fn id(&self) -> LevelId {
        self.root.level()
    }

    pub fn last_child(&self) -> DisplayObject<'gc> {
        self.last_child
    }

    pub fn set_last_child(&mut self, child: DisplayObject<'gc>) {
        self.last_child = child;
    }

    fn prev_level_id(&self) -> Option<LevelId> {
        self.prev_level_id
    }

    fn next_level_id(&self) -> Option<LevelId> {
        self.next_level_id
    }

    fn set_prev_level(&mut self, gc_context: MutationContext<'gc, '_>, level: Option<Level<'gc>>) {
        self.root.set_prev_exec(gc_context, level.map(|level| level.last_child));
        self.next_level_id = level.map(|level| level.id());
    }

    fn set_next_level(&mut self, gc_context: MutationContext<'gc, '_>, level: Option<Level<'gc>>) {
        self.last_child.set_next_exec(gc_context, level.map(|level| level.root));
        self.next_level_id = level.map(|level| level.id());
    }
}

pub struct LevelIter<'gc> {
    head: Option<Level<'gc>>,
}

impl<'gc> Iterator for LevelIter<'gc> {
    type Item = Level<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO
        None
    }
}

pub struct ExecIter<'gc> {
    head: Option<DisplayObject<'gc>>,
}

impl<'gc> Iterator for ExecIter<'gc> {
    type Item = DisplayObject<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO
        None
    }
}
