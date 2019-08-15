use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::event::PlayerEvent;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Button<'gc> {
    base: DisplayObjectBase<'gc>,

    state: ButtonState,

    children: [BTreeMap<Depth, DisplayNode<'gc>>; 4],
    release_actions: Vec<u8>,
}

impl<'gc> Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        library: &crate::library::Library<'gc>,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Self {
        use swf::ButtonState;
        let mut children = [
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        ];
        for record in &button.records {
            let child = library
                .instantiate_display_object(record.id, gc_context)
                .unwrap();
            child
                .write(gc_context)
                .set_matrix(&record.matrix.clone().into());
            child
                .write(gc_context)
                .set_color_transform(&record.color_transform.clone().into());
            for state in &record.states {
                let i = match state {
                    ButtonState::Up => 0,
                    ButtonState::Over => 1,
                    ButtonState::Down => 2,
                    ButtonState::HitTest => continue,
                };
                children[i].insert(record.depth, child);
            }
        }

        let mut release_actions = vec![];
        for actions in &button.actions {
            if actions
                .conditions
                .contains(&swf::ButtonActionCondition::OverDownToOverUp) || actions
                .conditions
                .contains(&swf::ButtonActionCondition::OverUpToOverDown)
            {
                release_actions = actions.action_data.clone();
            }
        }

        Button {
            base: Default::default(),
            children,
            state: self::ButtonState::Up,
            release_actions,
        }
    }

    fn children_in_state(
        &self,
        state: ButtonState,
    ) -> impl std::iter::DoubleEndedIterator<Item = &DisplayNode<'gc>> {
        let i = match state {
            ButtonState::Up => 0,
            ButtonState::Over => 1,
            ButtonState::Down => 2,
        };
        self.children[i].values()
    }

    fn children_in_state_mut(
        &mut self,
        state: ButtonState,
    ) -> impl std::iter::DoubleEndedIterator<Item = &mut DisplayNode<'gc>> {
        let i = match state {
            ButtonState::Up => 0,
            ButtonState::Over => 1,
            ButtonState::Down => 2,
        };
        self.children[i].values_mut()
    }
}

impl<'gc> DisplayObject<'gc> for Button<'gc> {
    impl_display_object!(base);

    fn run_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.children_in_state_mut(self.state) {
            child
                .write(context.gc_context)
                .set_parent(Some(context.active_clip));
            context.active_clip = *child;
            child.write(context.gc_context).run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.children_in_state_mut(self.state) {
            context.active_clip = *child;
            child.write(context.gc_context).run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(self.transform());

        for child in self.children_in_state(self.state) {
            child.read().render(context);
        }
        context.transform_stack.pop();
    }

    fn hit_test(&self, point: (Twips, Twips)) -> bool {
        //if self.world_bounds().contains(point) {
        for child in self.children_in_state(self.state).rev() {
            if child.read().world_bounds().contains(point) {
                return true;
            }
        }
        //}

        false
    }

    fn handle_event(
        &mut self,
        context: &mut crate::player::UpdateContext<'_, 'gc, '_>,
        event: PlayerEvent,
    ) {
        match event {
            PlayerEvent::RollOver => self.state = ButtonState::Over,
            PlayerEvent::Click => {
                let slice = crate::tag_utils::SwfSlice {
                    data: std::sync::Arc::new(self.release_actions.clone()),
                    start: 0,
                    end: self.release_actions.len(),
                };
                context.actions.push((self.parent().unwrap(), slice));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ButtonState {
    Up,
    Over,
    Down,
}

unsafe impl<'gc> gc_arena::Collect for Button<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for state in &self.children {
            for child in state.values() {
                child.trace(cc);
            }
        }
    }
}
