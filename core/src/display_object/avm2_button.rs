use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, ClassObject as Avm2ClassObject, Error as Avm2Error,
    Object as Avm2Object, StageObject as Avm2StageObject, Value as Avm2Value,
};
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::avm1_button::{ButtonState, ButtonTracking};
use crate::display_object::container::{dispatch_added_event, dispatch_removed_event};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, MovieClip, TDisplayObject};
use crate::events::{ClipEvent, ClipEventResult};
use crate::frame_lifecycle::catchup_display_object_to_frame;
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::{Collect, Gc, MutationContext};
use ruffle_gc_extra::lock::{Lock, RefLock};
use ruffle_gc_extra::{unlock, GcExt as _};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::sync::Arc;

use super::interactive::Avm2MousePick;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Avm2Button<'gc>(Gc<'gc, Avm2ButtonData<'gc>>);

impl fmt::Debug for Avm2Button<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Avm2Button")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Avm2ButtonData<'gc> {
    base: RefLock<InteractiveObjectBase<'gc>>,

    static_data: Gc<'gc, ButtonStatic>,

    /// The current button state to render.
    state: Cell<ButtonState>,

    /// The display object tree to render when the button is in the UP state.
    up_state: Lock<Option<DisplayObject<'gc>>>,

    /// The display object tree to render when the button is in the OVER state.
    over_state: Lock<Option<DisplayObject<'gc>>>,

    /// The display object tree to render when the button is in the DOWN state.
    down_state: Lock<Option<DisplayObject<'gc>>>,

    /// The display object tree to use for mouse hit checks.
    hit_area: Lock<Option<DisplayObject<'gc>>>,

    /// The current tracking mode of this button.
    tracking: Cell<ButtonTracking>,

    /// The class of this button.
    ///
    /// If not specified in `SymbolClass`, this will be
    /// `flash.display.SimpleButton`.
    class: Lock<Avm2ClassObject<'gc>>,

    /// The AVM2 representation of this button.
    // TODO(moulins): this could be a OnceLock
    object: Lock<Option<Avm2Object<'gc>>>,

    has_focus: Cell<bool>,
    enabled: Cell<bool>,
    use_hand_cursor: Cell<bool>,

    /// If this button needs to have it's AVM2 side initialized, or not.
    ///
    /// All buttons start out not needing AVM2 initialization.
    needs_avm2_initialization: Cell<bool>,

    /// If this button needs to have it's child states constructed, or not.
    ///
    /// All buttons start out unconstructed and have this flag set `true`.
    /// This flag is consumed during frame construction.
    needs_frame_construction: Cell<bool>,

    /// Skip the next `run_frame` call.
    ///
    /// This flag exists due to a really odd feature of buttons: they run their
    /// children for one frame before parents can run. Then they go back to the
    /// normal AVM2 execution order for future frames.
    skip_current_frame: Cell<bool>,
}

impl<'gc> Avm2Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        source_movie: &SwfSlice,
        context: &mut UpdateContext<'_, 'gc>,
        construct_blank_states: bool,
    ) -> Self {
        let static_data = ButtonStatic {
            swf: source_movie.movie.clone(),
            id: button.id,
            cell: RefCell::new(ButtonStaticMut {
                records: button.records.clone(),
                up_to_over_sound: None,
                over_to_down_sound: None,
                down_to_over_sound: None,
                over_to_up_sound: None,
            }),
        };

        Avm2Button(Gc::allocate(
            context.gc_context,
            Avm2ButtonData {
                base: Default::default(),
                static_data: Gc::allocate(context.gc_context, static_data),
                state: Cell::new(ButtonState::Up),
                hit_area: Lock::new(None),
                up_state: Lock::new(None),
                over_state: Lock::new(None),
                down_state: Lock::new(None),
                class: Lock::new(context.avm2.classes().simplebutton),
                object: Lock::new(None),
                needs_frame_construction: Cell::new(construct_blank_states),
                needs_avm2_initialization: Cell::new(false),
                tracking: Cell::new(if button.is_track_as_menu {
                    ButtonTracking::Menu
                } else {
                    ButtonTracking::Push
                }),
                has_focus: Cell::new(false),
                enabled: Cell::new(true),
                use_hand_cursor: Cell::new(true),
                skip_current_frame: Cell::new(false),
            },
        ))
    }

    pub fn empty_button(context: &mut UpdateContext<'_, 'gc>) -> Self {
        let movie = context.swf.clone();
        let button_record = swf::Button {
            id: 0,
            is_track_as_menu: false,
            records: Vec::new(),
            actions: Vec::new(),
        };

        Self::from_swf_tag(&button_record, &movie.into(), context, false)
    }

    pub fn set_sounds(self, sounds: swf::ButtonSounds) {
        let mut static_data = self.0.static_data.cell.borrow_mut();
        static_data.up_to_over_sound = sounds.up_to_over_sound;
        static_data.over_to_down_sound = sounds.over_to_down_sound;
        static_data.down_to_over_sound = sounds.down_to_over_sound;
        static_data.over_to_up_sound = sounds.over_to_up_sound;
    }

    /// Handles the ancient DefineButtonCxform SWF tag.
    /// Set the color transform for all children of each state.
    pub fn set_colors(self, color_transforms: &[swf::ColorTransform]) {
        let mut static_data = self.0.static_data.cell.borrow_mut();

        // This tag isn't documented well in SWF19. It is only used in very old SWF<=2 content.
        // It applies color transforms to every character in a button, in sequence(?).
        for (record, color_transform) in static_data.records.iter_mut().zip(color_transforms.iter())
        {
            record.color_transform = *color_transform;
        }
    }

    /// Construct a given state of the button and return it's containing
    /// DisplayObject.
    ///
    /// In contrast to AVM1Button, the AVM2 variety constructs all of it's
    /// children once and stores them in four named slots, either on their own
    /// (if they are singular) or in `Sprite`s created specifically to store
    /// button children. This means that, for example, a child that exists in
    /// multiple states in the SWF will actually be instantiated multiple
    /// times.
    ///
    /// If the boolean parameter is `true`, then the caller of this function
    /// should signal events on all children of the returned display object.
    fn create_state(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        swf_state: swf::ButtonState,
    ) -> (DisplayObject<'gc>, bool) {
        let movie = self.movie();
        let sprite_class = context.avm2.classes().sprite;

        let mut children = Vec::new();
        let static_data = self.0.static_data;

        for record in static_data.cell.borrow().records.iter() {
            if record.states.contains(swf_state) {
                match context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .instantiate_by_id(record.id, context.gc_context)
                {
                    Ok(child) => {
                        child.set_matrix(context.gc_context, record.matrix.into());
                        child.set_depth(context.gc_context, record.depth.into());

                        if swf_state != swf::ButtonState::HIT_TEST {
                            child.set_color_transform(context.gc_context, record.color_transform);
                        }

                        children.push((child, record.depth));
                    }
                    Err(error) => {
                        tracing::error!(
                            "Button ID {}: could not instantiate child ID {}: {}",
                            static_data.id,
                            record.id,
                            error
                        );
                    }
                };
            }
        }

        if children.len() == 1 {
            let child = children.first().cloned().unwrap().0;

            child.set_parent(context, Some(self.into()));
            child.post_instantiation(context, None, Instantiator::Movie, false);
            catchup_display_object_to_frame(context, child);

            (child, false)
        } else {
            let state_sprite = MovieClip::new(movie, context.gc_context);

            state_sprite.set_avm2_class(context.gc_context, Some(sprite_class));
            state_sprite.set_parent(context, Some(self.into()));
            catchup_display_object_to_frame(context, state_sprite.into());

            for (child, depth) in children {
                // `parent` returns `null` for these grandchildren during construction time, even though
                // `stage` and `root` will be defined. Set the parent temporarily to the button itself so
                // that `parent` is `null` (`DisplayObject::avm2_parent` checks that the parent is a container),
                // and then properly set the parent to the state Sprite afterwards.
                state_sprite.replace_at_depth(context, child, depth.into());
                child.set_parent(context, Some(self.into()));
                child.post_instantiation(context, None, Instantiator::Movie, false);
                catchup_display_object_to_frame(context, child);
                child.set_parent(context, Some(state_sprite.into()));
            }

            (state_sprite.into(), true)
        }
    }

    /// Get the rendered state of the button.
    pub fn state(self) -> ButtonState {
        self.0.state.get()
    }

    /// Change the rendered state of the button.
    pub fn set_state(self, context: &mut UpdateContext<'_, 'gc>, state: ButtonState) {
        self.0.state.set(state);
        if let Some(state) = self.0.up_state.get() {
            state.set_parent(context, None);
        }
        if let Some(state) = self.0.over_state.get() {
            state.set_parent(context, None);
        }
        if let Some(state) = self.0.down_state.get() {
            state.set_parent(context, None);
        }
        if let Some(state) = self.0.hit_area.get() {
            state.set_parent(context, None);
        }
        if let Some(state) = self.get_state_child(state.into()) {
            state.set_parent(context, Some(self.into()));
        }
    }

    /// Get the display object that represents a particular button state.
    pub fn get_state_child(self, state: swf::ButtonState) -> Option<DisplayObject<'gc>> {
        match state {
            swf::ButtonState::UP => self.0.up_state.get(),
            swf::ButtonState::OVER => self.0.over_state.get(),
            swf::ButtonState::DOWN => self.0.down_state.get(),
            swf::ButtonState::HIT_TEST => self.0.hit_area.get(),
            _ => None,
        }
    }

    /// Set the display object that represents a particular button state.
    pub fn set_state_child(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        state: swf::ButtonState,
        child: Option<DisplayObject<'gc>>,
    ) {
        let child_was_on_stage = child.map(|c| c.is_on_stage(context)).unwrap_or(false);
        let old_state_child = self.get_state_child(state);
        let is_cur_state = swf::ButtonState::from(self.0.state.get()) == state;

        let write = Gc::write(context.gc_context, self.0);
        match state {
            swf::ButtonState::UP => unlock!(write, Avm2ButtonData, up_state).set(child),
            swf::ButtonState::OVER => unlock!(write, Avm2ButtonData, over_state).set(child),
            swf::ButtonState::DOWN => unlock!(write, Avm2ButtonData, down_state).set(child),
            swf::ButtonState::HIT_TEST => unlock!(write, Avm2ButtonData, hit_area).set(child),
            _ => (),
        }

        if let Some(child) = child {
            if let Some(mut parent) = child.parent().and_then(|parent| parent.as_container()) {
                parent.remove_child(context, child);
            }

            if is_cur_state {
                child.set_parent(context, Some(self.into()));
            }
        }

        if let Some(old_state_child) = old_state_child {
            old_state_child.set_parent(context, None);
        }

        if is_cur_state {
            if let Some(child) = child {
                dispatch_added_event(self.into(), child, child_was_on_stage, context);
            }

            if let Some(old_state_child) = old_state_child {
                dispatch_removed_event(old_state_child, context);
            }

            if let Some(child) = child {
                child.frame_constructed(context);
            }
        }

        if is_cur_state {
            if let Some(child) = child {
                child.run_frame_scripts(context);
                child.exit_frame(context);
            }
        }
    }

    pub fn enabled(self) -> bool {
        self.0.enabled.get()
    }

    pub fn set_enabled(self, context: &mut UpdateContext<'_, 'gc>, enabled: bool) {
        self.0.enabled.set(enabled);
        if !enabled {
            self.set_state(context, ButtonState::Up);
        }
    }

    pub fn use_hand_cursor(self) -> bool {
        self.0.use_hand_cursor.get()
    }

    pub fn set_use_hand_cursor(self, use_hand_cursor: bool) {
        self.0.use_hand_cursor.set(use_hand_cursor);
    }

    pub fn button_tracking(self) -> ButtonTracking {
        self.0.tracking.get()
    }

    pub fn set_button_tracking(self, tracking: ButtonTracking) {
        self.0.tracking.set(tracking);
    }

    pub fn set_avm2_class(self, mc: MutationContext<'gc, '_>, class: Avm2ClassObject<'gc>) {
        unlock!(Gc::write(mc, self.0), Avm2ButtonData, class).set(class);
    }
}

impl<'gc> TDisplayObject<'gc> for Avm2Button<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.base.borrow(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        let data = unlock!(Gc::write(mc, self.0), Avm2ButtonData, base);
        RefMut::map(data.borrow_mut(), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(Gc::allocate(gc_context, (*self.0).clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        Gc::as_ptr(self.0) as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.static_data.id
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.static_data.swf.clone()
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        self.set_default_instance_name(context);
    }

    fn enter_frame(&self, context: &mut UpdateContext<'_, 'gc>) {
        if let Some(hit_area) = self.0.hit_area.get() {
            hit_area.enter_frame(context);
        }

        if let Some(up_state) = self.0.up_state.get() {
            up_state.enter_frame(context);
        }

        if let Some(down_state) = self.0.down_state.get() {
            down_state.enter_frame(context);
        }

        if let Some(over_state) = self.0.over_state.get() {
            over_state.enter_frame(context);
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc>) {
        if let Some(hit_area) = self.0.hit_area.get() {
            hit_area.construct_frame(context);
        }

        if let Some(up_state) = self.0.up_state.get() {
            up_state.construct_frame(context);
        }

        if let Some(down_state) = self.0.down_state.get() {
            down_state.construct_frame(context);
        }

        if let Some(over_state) = self.0.over_state.get() {
            over_state.construct_frame(context);
        }

        let needs_avm2_construction = self.0.object.get().is_none();
        let class = self.0.class.get();
        if needs_avm2_construction {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            match Avm2StageObject::for_display_object(&mut activation, (*self).into(), class) {
                Ok(object) => unlock!(
                    Gc::write(context.gc_context, self.0),
                    Avm2ButtonData,
                    object
                )
                .set(Some(object.into())),
                Err(e) => tracing::error!("Got {} when constructing AVM2 side of button", e),
            };

            self.on_construction_complete(context);
        }

        if self.0.needs_frame_construction.get() {
            let (up_state, up_should_fire) = self.create_state(context, swf::ButtonState::UP);
            let (over_state, over_should_fire) = self.create_state(context, swf::ButtonState::OVER);
            let (down_state, down_should_fire) = self.create_state(context, swf::ButtonState::DOWN);
            let (hit_area, hit_should_fire) =
                self.create_state(context, swf::ButtonState::HIT_TEST);

            let write = Gc::write(context.gc_context, self.0);

            unlock!(write, Avm2ButtonData, up_state).set(Some(up_state));
            unlock!(write, Avm2ButtonData, over_state).set(Some(over_state));
            unlock!(write, Avm2ButtonData, down_state).set(Some(down_state));
            unlock!(write, Avm2ButtonData, hit_area).set(Some(hit_area));
            write.skip_current_frame.set(true);
            write.needs_frame_construction.set(false);

            if up_should_fire {
                up_state.post_instantiation(context, None, Instantiator::Movie, false);

                if let Some(up_container) = up_state.as_container() {
                    for child in up_container.iter_render_list() {
                        dispatch_added_event((*self).into(), child, false, context);
                    }
                }
            }

            if over_should_fire {
                over_state.post_instantiation(context, None, Instantiator::Movie, false);

                if let Some(over_container) = over_state.as_container() {
                    for child in over_container.iter_render_list() {
                        dispatch_added_event((*self).into(), child, false, context);
                    }
                }
            }

            if down_should_fire {
                down_state.post_instantiation(context, None, Instantiator::Movie, false);

                if let Some(down_container) = down_state.as_container() {
                    for child in down_container.iter_render_list() {
                        dispatch_added_event((*self).into(), child, false, context);
                    }
                }
            }

            if hit_should_fire {
                hit_area.post_instantiation(context, None, Instantiator::Movie, false);

                if let Some(hit_container) = hit_area.as_container() {
                    for child in hit_container.iter_render_list() {
                        dispatch_added_event((*self).into(), child, false, context);
                    }
                }
            }

            if needs_avm2_construction {
                self.0.needs_avm2_initialization.set(true);

                self.frame_constructed(context);

                self.set_state(context, ButtonState::Up);

                up_state.run_frame_scripts(context);
                over_state.run_frame_scripts(context);
                down_state.run_frame_scripts(context);
                hit_area.run_frame_scripts(context);

                self.exit_frame(context);
            }
        } else if self.0.needs_avm2_initialization.get() {
            self.0.needs_avm2_initialization.set(true);
            if let Some(avm2_object) = self.0.object.get() {
                let mut constr_thing = || {
                    let mut activation = Avm2Activation::from_nothing(context.reborrow());
                    class.call_native_init(Some(avm2_object), &[], &mut activation)?;

                    Ok(())
                };
                let result: Result<(), Avm2Error> = constr_thing();

                if let Err(e) = result {
                    tracing::error!("Got {} when constructing AVM2 side of button", e);
                }
            }
        }
    }

    fn run_frame_scripts(self, context: &mut UpdateContext<'_, 'gc>) {
        if let Some(hit_area) = self.0.hit_area.get() {
            hit_area.run_frame_scripts(context);
        }

        if let Some(up_state) = self.0.up_state.get() {
            up_state.run_frame_scripts(context);
        }

        if let Some(down_state) = self.0.down_state.get() {
            down_state.run_frame_scripts(context);
        }

        if let Some(over_state) = self.0.over_state.get() {
            over_state.run_frame_scripts(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        let current_state = self.get_state_child(self.0.state.get().into());

        if let Some(state) = current_state {
            state.render(context);
        }
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        // No inherent bounds; contains child DisplayObjects.
        Default::default()
    }

    fn bounds_with_transform(&self, matrix: &Matrix) -> Rectangle<Twips> {
        // Get self bounds
        let mut bounds = *matrix * self.self_bounds();

        // Add the bounds of the child, dictated by current state
        if let Some(child) = self.get_state_child(self.0.state.get().into()) {
            let child_bounds = child.bounds_with_transform(matrix);
            bounds = bounds.union(&child_bounds);
        }

        bounds
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if !options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible() {
            if let Some(child) = self.get_state_child(self.0.state.get().into()) {
                //TODO: the if below should probably always be taken, why does the hit area
                // sometimes have a parent?
                let mut point = point;
                if child.parent().is_none() {
                    // hit_area is not actually a child, so transform point into local space before passing it down.
                    point = if let Some(point) = self.global_to_local(point) {
                        point
                    } else {
                        return false;
                    }
                }

                if child.hit_test_shape(context, point, options) {
                    return true;
                }
            }
        }

        false
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .object
            .get()
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'_, 'gc>, to: Avm2Object<'gc>) {
        let write = Gc::write(context.gc_context, self.0);
        unlock!(write, Avm2ButtonData, object).set(Some(to));
    }

    fn as_avm2_button(&self) -> Option<Self> {
        Some(*self)
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn allow_as_mask(&self) -> bool {
        let current_state = self.get_state_child(self.0.state.get().into());

        if let Some(current_state) = current_state.and_then(|cs| cs.as_container()) {
            current_state.is_empty()
        } else {
            false
        }
    }

    fn is_focusable(&self, _context: &mut UpdateContext<'_, 'gc>) -> bool {
        true
    }

    fn on_focus_changed(&self, _gc_context: MutationContext<'gc, '_>, focused: bool) {
        self.0.has_focus.set(focused);
    }
}

impl<'gc> TInteractiveObject<'gc> for Avm2Button<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        self.0.base.borrow()
    }

    fn raw_interactive_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> RefMut<InteractiveObjectBase<'gc>> {
        unlock!(Gc::write(mc, self.0), Avm2ButtonData, base).borrow_mut()
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'_, 'gc>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if !self.visible() {
            return ClipEventResult::NotHandled;
        }

        if !self.enabled() && !matches!(event, ClipEvent::KeyPress { .. }) {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn propagate_to_children(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if event.propagates() {
            let current_state = self.get_state_child(self.0.state.get().into());

            if let Some(current_state) = current_state.and_then(|s| s.as_interactive()) {
                if current_state.handle_clip_event(context, event) == ClipEventResult::Handled {
                    return ClipEventResult::Handled;
                }
            }
        }

        ClipEventResult::NotHandled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        // Translate the clip event to a button event, based on how the button state changes.
        let static_data = self.0.static_data.cell.borrow();
        let (new_state, sound) = match event {
            ClipEvent::DragOut { .. } => (ButtonState::Over, None),
            ClipEvent::DragOver { .. } => (ButtonState::Down, None),
            ClipEvent::Press => (ButtonState::Down, static_data.over_to_down_sound.as_ref()),
            ClipEvent::Release => (ButtonState::Over, static_data.down_to_over_sound.as_ref()),
            ClipEvent::ReleaseOutside => (ButtonState::Up, static_data.over_to_up_sound.as_ref()),
            ClipEvent::MouseUpInside => (ButtonState::Up, static_data.over_to_up_sound.as_ref()),
            ClipEvent::RollOut { .. } => (ButtonState::Up, static_data.over_to_up_sound.as_ref()),
            ClipEvent::RollOver { .. } => {
                (ButtonState::Over, static_data.up_to_over_sound.as_ref())
            }
            _ => return ClipEventResult::NotHandled,
        };

        self.0.play_sound(context, sound);
        let old_state = self.0.state.get();

        if old_state != new_state {
            self.set_state(context, new_state);
        }
        ClipEventResult::Handled
    }

    fn mouse_pick_avm2(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        mut point: Point<Twips>,
        require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() && self.mouse_enabled() {
            let state_child = self.get_state_child(self.0.state.get().into());

            if let Some(state_child) = state_child {
                let mouse_pick = state_child
                    .as_interactive()
                    .map(|c| c.mouse_pick_avm2(context, point, require_button_mode));
                match mouse_pick {
                    None | Some(Avm2MousePick::Miss) => {}
                    // Selecting a child of a button is equivalent to selecting the button itself
                    _ => return Avm2MousePick::Hit((*self).into()),
                };
            }

            if let Some(hit_area) = self.0.hit_area.get() {
                //TODO: the if below should probably always be taken, why does the hit area
                // sometimes have a parent?
                if hit_area.parent().is_none() {
                    // hit_area is not actually a child, so transform point into local space before passing it down.
                    point = if let Some(point) = self.global_to_local(point) {
                        point
                    } else {
                        return Avm2MousePick::Miss;
                    }
                }
                if hit_area.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                    return Avm2MousePick::Hit((*self).into());
                }
            }
        }
        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'_, 'gc>) -> MouseCursor {
        // TODO: Should we also need to check for the `enabled` property like AVM1 buttons?
        if self.use_hand_cursor() {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }
}

impl<'gc> Avm2ButtonData<'gc> {
    fn play_sound(&self, context: &mut UpdateContext<'_, 'gc>, sound: Option<&swf::ButtonSound>) {
        if let Some((id, sound_info)) = sound {
            if let Some(sound_handle) = context
                .library
                .library_for_movie_mut(self.movie())
                .get_sound(*id)
            {
                let _ = context.start_sound(sound_handle, sound_info, None, None);
            }
        }
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.static_data.swf.clone()
    }
}

#[derive(Collect, Debug)]
#[collect(require_static)]
struct ButtonStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    cell: RefCell<ButtonStaticMut>,
}

#[derive(Debug)]
struct ButtonStaticMut {
    records: Vec<swf::ButtonRecord>,

    /// The sounds to play on state changes for this button.
    up_to_over_sound: Option<swf::ButtonSound>,
    over_to_down_sound: Option<swf::ButtonSound>,
    down_to_over_sound: Option<swf::ButtonSound>,
    over_to_up_sound: Option<swf::ButtonSound>,
}
