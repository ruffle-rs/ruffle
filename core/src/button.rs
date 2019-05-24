use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Button<'gc> {
    base: DisplayObjectBase,

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
                .contains(&swf::ButtonActionCondition::OverDownToOverUp)
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

    fn children_in_state(&self, state: ButtonState) -> impl Iterator<Item = &DisplayNode<'gc>> {
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
    ) -> impl Iterator<Item = &mut DisplayNode<'gc>> {
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
        if self.state == ButtonState::Down {
            // let mut action_context = crate::avm1::ActionContext {
            //     global_time: context.global_time,
            //     active_clip: &mut crate::movie_clip::MovieClip::new(),
            //     audio: context.audio,
            // };
            // context
            //     .avm1
            //     .do_action(&mut action_context, &self.release_actions[..]);
            self.state = ButtonState::Up;
        } else if self.state == ButtonState::Up {
            let dx = self.get_matrix().tx - context.mouse_pos.0;
            let dy = self.get_matrix().ty - context.mouse_pos.1;
            let len = f32::sqrt(dx * dx + dy * dy);
            self.state = if len > 20.0 {
                ButtonState::Up
            } else {
                ButtonState::Over
            };
        }
        for child in self.children_in_state_mut(self.state) {
            child.write(context.gc_context).run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.children_in_state_mut(self.state) {
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

    fn handle_click(&mut self, _pos: (f32, f32)) {
        if self.state == ButtonState::Over {
            self.state = ButtonState::Down;
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
