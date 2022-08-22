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
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Avm2Button<'gc>(GcCell<'gc, Avm2ButtonData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Avm2ButtonData<'gc> {
    base: InteractiveObjectBase<'gc>,

    static_data: GcCell<'gc, ButtonStatic>,

    /// The current button state to render.
    state: ButtonState,

    /// The display object tree to render when the button is in the UP state.
    up_state: Option<DisplayObject<'gc>>,

    /// The display object tree to render when the button is in the OVER state.
    over_state: Option<DisplayObject<'gc>>,

    /// The display object tree to render when the button is in the DOWN state.
    down_state: Option<DisplayObject<'gc>>,

    /// The display object tree to use for mouse hit checks.
    hit_area: Option<DisplayObject<'gc>>,

    /// The current tracking mode of this button.
    tracking: ButtonTracking,

    /// The class of this button.
    ///
    /// If not specified in `SymbolClass`, this will be
    /// `flash.display.SimpleButton`.
    class: Avm2ClassObject<'gc>,

    /// The AVM2 representation of this button.
    object: Option<Avm2Object<'gc>>,

    /// If this button needs to have it's child states constructed, or not.
    ///
    /// All buttons start out unconstructed and have this flag set `true`.
    /// This flag is consumed during frame construction.
    needs_frame_construction: bool,

    /// If this button needs to have it's AVM2 side initialized, or not.
    ///
    /// All buttons start out not needing AVM2 initialization.
    needs_avm2_initialization: bool,

    has_focus: bool,
    enabled: bool,
    use_hand_cursor: bool,

    /// Skip the next `run_frame` call.
    ///
    /// This flag exists due to a really odd feature of buttons: they run their
    /// children for one frame before parents can run. Then they go back to the
    /// normal AVM2 execution order for future frames.
    skip_current_frame: bool,
}

impl<'gc> Avm2Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        source_movie: &SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Self {
        let static_data = ButtonStatic {
            swf: source_movie.movie.clone(),
            id: button.id,
            records: button.records.clone(),
            up_to_over_sound: None,
            over_to_down_sound: None,
            down_to_over_sound: None,
            over_to_up_sound: None,
        };

        Avm2Button(GcCell::allocate(
            context.gc_context,
            Avm2ButtonData {
                base: Default::default(),
                static_data: GcCell::allocate(context.gc_context, static_data),
                state: self::ButtonState::Up,
                hit_area: None,
                up_state: None,
                over_state: None,
                down_state: None,
                class: context.avm2.classes().simplebutton,
                object: None,
                needs_frame_construction: true,
                needs_avm2_initialization: false,
                tracking: if button.is_track_as_menu {
                    ButtonTracking::Menu
                } else {
                    ButtonTracking::Push
                },
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
                skip_current_frame: false,
            },
        ))
    }

    pub fn empty_button(context: &mut UpdateContext<'_, 'gc, '_>) -> Self {
        let movie = Arc::new(SwfMovie::empty(context.swf.version()));
        let button_record = swf::Button {
            id: 0,
            is_track_as_menu: false,
            records: Vec::new(),
            actions: Vec::new(),
        };

        Self::from_swf_tag(&button_record, &movie.into(), context)
    }

    pub fn set_sounds(self, gc_context: MutationContext<'gc, '_>, sounds: swf::ButtonSounds) {
        let button = self.0.write(gc_context);
        let mut static_data = button.static_data.write(gc_context);
        static_data.up_to_over_sound = sounds.up_to_over_sound;
        static_data.over_to_down_sound = sounds.over_to_down_sound;
        static_data.down_to_over_sound = sounds.down_to_over_sound;
        static_data.over_to_up_sound = sounds.over_to_up_sound;
    }

    /// Handles the ancient DefineButtonCxform SWF tag.
    /// Set the color transform for all children of each state.
    pub fn set_colors(
        self,
        gc_context: MutationContext<'gc, '_>,
        color_transforms: &[swf::ColorTransform],
    ) {
        let button = self.0.write(gc_context);
        let mut static_data = button.static_data.write(gc_context);

        // This tag isn't documented well in SWF19. It is only used in very old SWF<=2 content.
        // It applies color transforms to every character in a button, in sequence(?).
        for (record, color_transform) in static_data.records.iter_mut().zip(color_transforms.iter())
        {
            record.color_transform = color_transform.clone();
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_state: swf::ButtonState,
    ) -> (DisplayObject<'gc>, bool) {
        let movie = self
            .movie()
            .expect("All SWF-defined buttons should have movies");
        let sprite_class = context.avm2.classes().sprite;

        let mut children = Vec::new();
        let static_data = self.0.read().static_data;

        for record in static_data.read().records.iter() {
            if record.states.contains(swf_state) {
                match context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .instantiate_by_id(record.id, context.gc_context)
                {
                    Ok(child) => {
                        child.set_matrix(context.gc_context, &record.matrix.into());
                        child.set_depth(context.gc_context, record.depth.into());

                        if swf_state != swf::ButtonState::HIT_TEST {
                            child.set_color_transform(
                                context.gc_context,
                                &record.color_transform.clone().into(),
                            );
                        }

                        children.push((child, record.depth));
                    }
                    Err(error) => {
                        log::error!(
                            "Button ID {}: could not instantiate child ID {}: {}",
                            static_data.read().id,
                            record.id,
                            error
                        );
                    }
                };
            }
        }

        if children.len() == 1 {
            let child = children.first().cloned().unwrap().0;

            child.set_parent(context.gc_context, Some(self.into()));
            child.post_instantiation(context, None, Instantiator::Movie, false);
            catchup_display_object_to_frame(context, child);

            (child, false)
        } else {
            let state_sprite = MovieClip::new(movie, context.gc_context);

            state_sprite.set_avm2_class(context.gc_context, Some(sprite_class));
            state_sprite.set_parent(context.gc_context, Some(self.into()));
            catchup_display_object_to_frame(context, state_sprite.into());

            for (child, depth) in children {
                // `parent` returns `null` for these grandchildren during construction time, even though
                // `stage` and `root` will be defined. Set the parent temporarily to the button itself so
                // that `parent` is `null` (`DisplayObject::avm2_parent` checks that the parent is a container),
                // and then properly set the parent to the state Sprite afterwards.
                state_sprite.replace_at_depth(context, child, depth.into());
                child.set_parent(context.gc_context, Some(self.into()));
                child.post_instantiation(context, None, Instantiator::Movie, false);
                catchup_display_object_to_frame(context, child);
                child.set_parent(context.gc_context, Some(state_sprite.into()));
            }

            (state_sprite.into(), true)
        }
    }

    /// Get the rendered state of the button.
    pub fn state(self) -> ButtonState {
        self.0.read().state
    }

    /// Change the rendered state of the button.
    pub fn set_state(self, context: &mut UpdateContext<'_, 'gc, '_>, state: ButtonState) {
        self.0.write(context.gc_context).state = state;
        let button = self.0.read();
        if let Some(state) = button.up_state {
            state.set_parent(context.gc_context, None);
        }
        if let Some(state) = button.over_state {
            state.set_parent(context.gc_context, None);
        }
        if let Some(state) = button.down_state {
            state.set_parent(context.gc_context, None);
        }
        if let Some(state) = button.hit_area {
            state.set_parent(context.gc_context, None);
        }
        if let Some(state) = self.get_state_child(state.into()) {
            state.set_parent(context.gc_context, Some(self.into()));
        }
    }

    /// Get the display object that represents a particular button state.
    pub fn get_state_child(self, state: swf::ButtonState) -> Option<DisplayObject<'gc>> {
        match state {
            swf::ButtonState::UP => self.0.read().up_state,
            swf::ButtonState::OVER => self.0.read().over_state,
            swf::ButtonState::DOWN => self.0.read().down_state,
            swf::ButtonState::HIT_TEST => self.0.read().hit_area,
            _ => None,
        }
    }

    /// Set the display object that represents a particular button state.
    pub fn set_state_child(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        state: swf::ButtonState,
        child: Option<DisplayObject<'gc>>,
    ) {
        let child_was_on_stage = child.map(|c| c.is_on_stage(context)).unwrap_or(false);
        let old_state_child = self.get_state_child(state);
        let is_cur_state = swf::ButtonState::from(self.0.read().state) == state;

        match state {
            swf::ButtonState::UP => self.0.write(context.gc_context).up_state = child,
            swf::ButtonState::OVER => self.0.write(context.gc_context).over_state = child,
            swf::ButtonState::DOWN => self.0.write(context.gc_context).down_state = child,
            swf::ButtonState::HIT_TEST => self.0.write(context.gc_context).hit_area = child,
            _ => (),
        }

        if let Some(child) = child {
            if let Some(mut parent) = child.parent().and_then(|parent| parent.as_container()) {
                parent.remove_child(context, child, Lists::all());
            }

            if is_cur_state {
                child.set_parent(context.gc_context, Some(self.into()));
            }
        }

        if let Some(old_state_child) = old_state_child {
            old_state_child.unload(context);
            old_state_child.set_parent(context.gc_context, None);
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

        if let Some(child) = child {
            child.run_frame_avm2(context);
        }

        if is_cur_state {
            if let Some(child) = child {
                child.run_frame_scripts(context);
                child.exit_frame(context);
            }
        }
    }

    pub fn enabled(self) -> bool {
        self.0.read().enabled
    }

    pub fn set_enabled(self, context: &mut UpdateContext<'_, 'gc, '_>, enabled: bool) {
        self.0.write(context.gc_context).enabled = enabled;
        if !enabled {
            self.set_state(context, ButtonState::Up);
        }
    }

    pub fn use_hand_cursor(self) -> bool {
        self.0.read().use_hand_cursor
    }

    pub fn set_use_hand_cursor(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        use_hand_cursor: bool,
    ) {
        self.0.write(context.gc_context).use_hand_cursor = use_hand_cursor;
    }

    pub fn button_tracking(self) -> ButtonTracking {
        self.0.read().tracking
    }

    pub fn set_button_tracking(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        tracking: ButtonTracking,
    ) {
        self.0.write(context.gc_context).tracking = tracking;
    }

    pub fn set_avm2_class(self, mc: MutationContext<'gc, '_>, class: Avm2ClassObject<'gc>) {
        self.0.write(mc).class = class;
    }
}

impl<'gc> TDisplayObject<'gc> for Avm2Button<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.read().id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().static_data.read().swf.clone())
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if run_frame {
            self.run_frame_avm2(context);
        }

        self.set_state(context, ButtonState::Up);
    }

    fn enter_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let hit_area = self.0.read().hit_area;
        if let Some(hit_area) = hit_area {
            hit_area.enter_frame(context);
        }

        let up_state = self.0.read().up_state;
        if let Some(up_state) = up_state {
            up_state.enter_frame(context);
        }

        let down_state = self.0.read().down_state;
        if let Some(down_state) = down_state {
            down_state.enter_frame(context);
        }

        let over_state = self.0.read().over_state;
        if let Some(over_state) = over_state {
            over_state.enter_frame(context);
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let hit_area = self.0.read().hit_area;
        if let Some(hit_area) = hit_area {
            hit_area.construct_frame(context);
        }

        let up_state = self.0.read().up_state;
        if let Some(up_state) = up_state {
            up_state.construct_frame(context);
        }

        let down_state = self.0.read().down_state;
        if let Some(down_state) = down_state {
            down_state.construct_frame(context);
        }

        let over_state = self.0.read().over_state;
        if let Some(over_state) = over_state {
            over_state.construct_frame(context);
        }

        let needs_avm2_construction = self.0.read().object.is_none();
        let class = self.0.read().class;
        if needs_avm2_construction {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            match Avm2StageObject::for_display_object(&mut activation, (*self).into(), class) {
                Ok(object) => self.0.write(context.gc_context).object = Some(object.into()),
                Err(e) => log::error!("Got {} when constructing AVM2 side of button", e),
            };
        }

        let needs_frame_construction = self.0.read().needs_frame_construction;
        if needs_frame_construction {
            let (up_state, up_should_fire) = self.create_state(context, swf::ButtonState::UP);
            let (over_state, over_should_fire) = self.create_state(context, swf::ButtonState::OVER);
            let (down_state, down_should_fire) = self.create_state(context, swf::ButtonState::DOWN);
            let (hit_area, hit_should_fire) =
                self.create_state(context, swf::ButtonState::HIT_TEST);

            let mut write = self.0.write(context.gc_context);
            write.up_state = Some(up_state);
            write.over_state = Some(over_state);
            write.down_state = Some(down_state);
            write.hit_area = Some(hit_area);
            write.skip_current_frame = true;
            write.needs_frame_construction = false;

            drop(write);

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
                self.0.write(context.gc_context).needs_avm2_initialization = true;

                self.frame_constructed(context);

                self.set_state(context, ButtonState::Over);

                //NOTE: Yes, we do have to run these in a different order from the
                //regular run_frame method.
                up_state.run_frame_avm2(context);
                over_state.run_frame_avm2(context);
                down_state.run_frame_avm2(context);
                hit_area.run_frame_avm2(context);

                up_state.run_frame_scripts(context);
                over_state.run_frame_scripts(context);
                down_state.run_frame_scripts(context);
                hit_area.run_frame_scripts(context);

                self.exit_frame(context);
            }
        } else if self.0.read().needs_avm2_initialization {
            self.0.write(context.gc_context).needs_avm2_initialization = false;
            let avm2_object = self.0.read().object;
            if let Some(avm2_object) = avm2_object {
                let mut constr_thing = || {
                    let mut activation = Avm2Activation::from_nothing(context.reborrow());
                    class.call_native_init(Some(avm2_object), &[], &mut activation)?;

                    Ok(())
                };
                let result: Result<(), Avm2Error> = constr_thing();

                if let Err(e) = result {
                    log::error!("Got {} when constructing AVM2 side of button", e);
                }
            }
        }
    }

    fn run_frame_avm2(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.0.read().skip_current_frame {
            self.0.write(context.gc_context).skip_current_frame = false;
            return;
        }

        let hit_area = self.0.read().hit_area;
        if let Some(hit_area) = hit_area {
            hit_area.run_frame_avm2(context);
        }

        let up_state = self.0.read().up_state;
        if let Some(up_state) = up_state {
            up_state.run_frame_avm2(context);
        }

        let down_state = self.0.read().down_state;
        if let Some(down_state) = down_state {
            down_state.run_frame_avm2(context);
        }

        let over_state = self.0.read().over_state;
        if let Some(over_state) = over_state {
            over_state.run_frame_avm2(context);
        }
    }

    fn run_frame_scripts(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let hit_area = self.0.read().hit_area;
        if let Some(hit_area) = hit_area {
            hit_area.run_frame_scripts(context);
        }

        let up_state = self.0.read().up_state;
        if let Some(up_state) = up_state {
            up_state.run_frame_scripts(context);
        }

        let down_state = self.0.read().down_state;
        if let Some(down_state) = down_state {
            down_state.run_frame_scripts(context);
        }

        let over_state = self.0.read().over_state;
        if let Some(over_state) = over_state {
            over_state.run_frame_scripts(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        let state = self.0.read().state;
        let current_state = self.get_state_child(state.into());

        if let Some(state) = current_state {
            state.render(context);
        }
    }

    fn self_bounds(&self) -> BoundingBox {
        // No inherent bounds; contains child DisplayObjects.
        BoundingBox::default()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
        options: HitTestOptions,
    ) -> bool {
        if !options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible() {
            let state = self.0.read().state;
            if let Some(child) = self.get_state_child(state.into()) {
                // hit_area is not actually a child, so transform point into local space before passing it down.
                let point = self.global_to_local(point);
                if child.hit_test_shape(context, point, options) {
                    return true;
                }
            }
        }

        false
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Undefined)
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).object = Some(to);
    }

    fn as_avm2_button(&self) -> Option<Self> {
        Some(*self)
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn allow_as_mask(&self) -> bool {
        let state = self.0.read().state;
        let current_state = self.get_state_child(state.into());

        if let Some(current_state) = current_state.and_then(|cs| cs.as_container()) {
            current_state.is_empty()
        } else {
            false
        }
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_focus_changed(&self, gc_context: MutationContext<'gc, '_>, focused: bool) {
        self.0.write(gc_context).has_focus = focused;
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let had_focus = self.0.read().has_focus;
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }
        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }
        self.set_removed(context.gc_context, true);
    }
}

impl<'gc> TInteractiveObject<'gc> for Avm2Button<'gc> {
    fn ibase(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn ibase_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(self, event: ClipEvent) -> ClipEventResult {
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if event.propagates() {
            let state = self.0.read().state;
            let current_state = self.get_state_child(state.into());

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
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        let write = self.0.write(context.gc_context);

        // Translate the clip event to a button event, based on how the button state changes.
        let static_data = write.static_data;
        let static_data = static_data.read();
        let (new_state, sound) = match event {
            ClipEvent::DragOut { .. } => (ButtonState::Over, None),
            ClipEvent::DragOver { .. } => (ButtonState::Down, None),
            ClipEvent::Press => (ButtonState::Down, static_data.over_to_down_sound.as_ref()),
            ClipEvent::Release => (ButtonState::Over, static_data.down_to_over_sound.as_ref()),
            ClipEvent::ReleaseOutside => (ButtonState::Up, static_data.over_to_up_sound.as_ref()),
            ClipEvent::RollOut { .. } => (ButtonState::Up, static_data.over_to_up_sound.as_ref()),
            ClipEvent::RollOver { .. } => {
                (ButtonState::Over, static_data.up_to_over_sound.as_ref())
            }
            _ => return ClipEventResult::NotHandled,
        };

        write.play_sound(context, sound);
        let old_state = write.state;
        drop(write);

        if old_state != new_state {
            self.set_state(context, new_state);
        }

        self.event_dispatch_to_avm2(context, event)
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() && self.mouse_enabled() {
            let state = self.0.read().state;
            let state_child = self.get_state_child(state.into());

            if let Some(state_child) = state_child {
                let mouse_pick = state_child
                    .as_interactive()
                    .and_then(|c| c.mouse_pick(context, point, require_button_mode));
                if mouse_pick.is_some() {
                    return mouse_pick;
                }
            }

            let hit_area = self.0.read().hit_area;
            if let Some(hit_area) = hit_area {
                // hit_area is not actually a child, so transform point into local space before passing it down.
                let point = self.global_to_local(point);
                if hit_area.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                    return Some((*self).into());
                }
            }
        }
        None
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'_, 'gc, '_>) -> MouseCursor {
        // TODO: Should we also need to check for the `enabled` property like AVM1 buttons?
        if self.use_hand_cursor() {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }
}

impl<'gc> Avm2ButtonData<'gc> {
    fn play_sound(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        sound: Option<&swf::ButtonSound>,
    ) {
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
        self.static_data.read().swf.clone()
    }
}

/// Static data shared between all instances of a button.
#[allow(dead_code)]
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct ButtonStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    records: Vec<swf::ButtonRecord>,

    /// The sounds to play on state changes for this button.
    up_to_over_sound: Option<swf::ButtonSound>,
    over_to_down_sound: Option<swf::ButtonSound>,
    down_to_over_sound: Option<swf::ButtonSound>,
    over_to_up_sound: Option<swf::ButtonSound>,
}
