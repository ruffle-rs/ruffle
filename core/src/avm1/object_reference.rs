use super::{object::stage_object::StageObjectData, Activation, Object, StageObject, TObject};
use crate::{
    display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer},
    string::AvmString,
};
use gc_arena::{Collect, GcWeakCell};

/// Represents a reference to a movie clip in AVM1
/// This consists of a string path which will be resolved to a target value when used
/// This also handles caching to maintain performance
#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct MovieClipReference<'gc> {
    /// The path to the target clip
    pub path: AvmString<'gc>,

    /// A weak reference to the target stage object that `path` points to
    /// This is used for fast-path resvoling when possible, as well as for re-generating `path` (in the case the the target object is renamed)
    //TODO: on first fast-path fail, clear this, and keep it up to date
    pub cached_stage_object: GcWeakCell<'gc, StageObjectData<'gc>>,
}

impl<'gc> MovieClipReference<'gc> {
    pub fn try_from_stage_object(
        activation: &mut Activation<'_, 'gc>,
        stage_object: StageObject<'gc>,
    ) -> Option<Self> {
        // We can't use as_display_object + as_movie_clip here as we explicitly don't want to convert `SuperObjects`
        let display_object = stage_object.as_display_object().unwrap();
        if let DisplayObject::MovieClip(mc) = display_object {
            let path = AvmString::new(activation.context.gc_context, mc.path());
            Some(Self {
                path,
                cached_stage_object: stage_object.as_weak(),
            })
        } else if activation.swf_version() <= 5 {
            let display_object = Self::process_swf5_references(activation, display_object);
            let stage_object = display_object
                .object()
                .coerce_to_object(activation)
                .as_stage_object()?;
            let path = AvmString::new(activation.context.gc_context, display_object.path());

            Some(Self {
                path,
                cached_stage_object: stage_object.as_weak(),
            })
        } else {
            None
        }
    }

    /// Handle the logic of swfv5 DisplayObjects
    fn process_swf5_references(
        activation: &mut Activation<'_, 'gc>,
        mut display_object: DisplayObject<'gc>,
    ) -> DisplayObject<'gc> {
        // In swfv5 paths resolve to the first MovieClip parent if the target isn't a movieclip
        if activation.swf_version() <= 5 {
            while display_object.as_movie_clip().is_none() {
                if let Some(p) = display_object.avm1_parent() {
                    display_object = p;
                }
            }
        }
        display_object
    }

    /// Resolve this reference to an object
    /// First tuple param indificates if this path came from the cache or not
    pub fn resolve_reference(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<(bool, Object<'gc>, DisplayObject<'gc>)> {
        // Check if we can re-use the cached `DisplayObject`, if we can then take this fast path
        if let Some(mc) = self
            .cached_stage_object
            .upgrade(activation.context.gc_context)
        {
            // We have to fallback to manual path-walking if the object is removed
            if !mc.read().display_object.removed()
            //TODO: add a test for this:
            /* || mc.read().display_object.pending_removal()*/
            {
                let display_object = mc.read().display_object;
                let display_object = Self::process_swf5_references(activation, display_object);

                //TODO: we need to update our cached path here, if it's different, otherwise once this WeakRef dies, we might not be able to resolve anymore

                return Some((
                    true,
                    display_object.object().coerce_to_object(activation),
                    display_object,
                ));
            }
        }

        // Either the GcWeak ref is gone, or the clip can't be used (not on stage etc)
        // Here we manually parse the path, in order to find the target display object
        // This is different to how paths resolve in activation in two ways:
        // 1: We only handle slash-paths to display objects, other path type and paths to variables are *not* valid here
        // 2: We only interact with the DisplayObject tree, not scopes, if you shadow a display object in a path this needs to still resolve to the correct object, e.g:
        // var _level0 = 123;
        // trace(this.child);
        // Should correctly find the child. As `this` is Value::MovieClip("_level0.child"), we don't want to try and find `123.child`!

        // Break up the path (TODO: cache this?)
        let mut parts = self.path.as_wstr().split(b'.');

        // Get the stage root
        let mut start = activation
            .root_object()
            .coerce_to_object(activation)
            .as_display_object();

        // Handle the level id, to support multi-file movies (TODO: cache this)
        if let Some(root) = parts.next() {
            if root.to_utf8_lossy().starts_with("_level") {
                if let Ok(level_id) = root[6..].parse::<i32>() {
                    start = Some(activation.resolve_level(level_id));
                }
            }
        }

        // Keep traversing to find the target DisplayObject
        for part in parts {
            if let Some(s) = start {
                if let Some(con) = s.as_container() {
                    start = con.child_by_name(part, activation.is_case_sensitive());
                }
            }
        }

        if let Some(start) = start {
            let display_object = Self::process_swf5_references(activation, start);

            Some((
                false,
                display_object.object().coerce_to_object(activation),
                display_object,
            ))
        } else {
            None
        }
    }

    /// Convert this reference to an `Object`
    pub fn coerce_to_object(&self, activation: &mut Activation<'_, 'gc>) -> Option<Object<'gc>> {
        let (_, object, _) = self.resolve_reference(activation)?;
        Some(object)
    }

    /// Convert this reference to a `String`
    pub fn coerce_to_string(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self.resolve_reference(activation) {
            // Couldn't find the reference
            None => "".into(),
            // Found the reference, cached, we can't re-use `self.path` sadly, it would be quicker if we could
            // But if the clip has been re-named, since being created then `mc.path() != path`
            //TODO: if we update internal path, we won't need to distinguish these two
            Some((true, _, display_object)) => AvmString::new(
                activation.context.gc_context,
                display_object.path().as_wstr(),
            ),
            // Found the reference, un-cached, so our cached path must be correct
            Some((false, _, _)) => self.path,
        }
    }

    /// Get the path used for this reference
    pub fn path(&self) -> &AvmString<'gc> {
        &self.path
    }
}


/// A reference to an `Object`-like, either an `Object` or a `MovieClip`
pub enum ObjectReference<'gc> {
    Object(Object<'gc>),
    MovieClip(MovieClipReference<'gc>)
}