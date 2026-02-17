use crate::avm1::Object as Avm1Object;
use crate::avm2::StageObject as Avm2StageObject;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{BoundsMode, DisplayObjectBase, MovieClip};
use crate::font::{FontLike, TextRenderSettings};
use crate::prelude::*;
use crate::tag_utils::SwfMovieGc;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::Lock;
use gc_arena::barrier::unlock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::transform::Transform;
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;
use std::cell::RefCell;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Text<'gc>(Gc<'gc, TextData<'gc>>);

impl fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextData<'gc> {
    base: DisplayObjectBase<'gc>,
    shared: Lock<Gc<'gc, TextShared<'gc>>>,
    render_settings: RefCell<TextRenderSettings>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
}

impl<'gc> Text<'gc> {
    pub fn from_swf_tag(
        context: &mut UpdateContext<'gc>,
        swf: SwfMovieGc<'gc>,
        tag: &swf::Text,
    ) -> Self {
        Text(Gc::new(
            context.gc(),
            TextData {
                base: Default::default(),
                shared: Lock::new(Gc::new(
                    context.gc(),
                    TextShared {
                        swf,
                        id: tag.id,
                        bounds: tag.bounds,
                        text_transform: tag.matrix.into(),
                        text_blocks: tag.records.clone(),
                    },
                )),
                render_settings: RefCell::new(Default::default()),
                avm2_object: Lock::new(None),
            },
        ))
    }

    fn set_shared(&self, context: &mut UpdateContext<'gc>, to: Gc<'gc, TextShared<'gc>>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), TextData, shared).set(to);
    }

    pub fn set_render_settings(self, settings: TextRenderSettings) {
        *self.0.render_settings.borrow_mut() = settings;
        self.invalidate_cached_bitmap();
    }

    pub fn text(self, context: &mut UpdateContext<'gc>) -> Option<WString> {
        let mut ret = WString::new();

        let mut font_id = None;

        for block in &self.0.shared.get().text_blocks {
            if block.font_id.is_some() {
                font_id = block.font_id;
            }

            if let Some(font) = font_id.and_then(|fid| {
                context
                    .library
                    .library_for_movie_gc(self.movie(), context.gc())
                    .unwrap()
                    .borrow()
                    .get_font(fid)
            }) {
                for glyph in &block.glyphs {
                    if let Some(g) = font.get_glyph(glyph.index as usize) {
                        ret.push_char(g.character());
                    }
                }
            }
        }

        if ret.is_empty() { None } else { Some(ret) }
    }
}

impl<'gc> TDisplayObject<'gc> for Text<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.shared.get().id
    }

    fn movie(self) -> SwfMovieGc<'gc> {
        self.0.shared.get().swf
    }

    fn replace_with(self, context: &mut UpdateContext<'gc>, id: CharacterId) {
        if let Some(new_text) = context
            .library
            .library_for_movie_gc(self.movie(), context.gc())
            .unwrap()
            .borrow()
            .get_text(id)
        {
            self.set_shared(context, new_text.0.shared.get());
        } else {
            tracing::warn!("PlaceObject: expected text at character ID {}", id);
        }
        self.invalidate_cached_bitmap();
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        let shared = self.0.shared.get();
        context.transform_stack.push(&Transform {
            matrix: shared.text_transform,
            ..Default::default()
        });

        let mut color = swf::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let mut font_id = 0;
        let mut height = Twips::ZERO;
        let mut transform: Transform = Default::default();
        for block in &shared.text_blocks {
            if let Some(x) = block.x_offset {
                transform.matrix.tx = x;
            }
            if let Some(y) = block.y_offset {
                transform.matrix.ty = y;
            }
            color = block.color.unwrap_or(color);
            font_id = block.font_id.unwrap_or(font_id);
            height = block.height.unwrap_or(height);
            if let Some(font) = context
                .library
                .library_for_movie_gc(self.movie(), context.gc_context)
                .unwrap()
                .borrow()
                .get_font(font_id)
            {
                let scale = (height.get() as f32) / font.scale();
                transform.matrix.a = scale;
                transform.matrix.d = scale;
                transform.color_transform.set_mult_color(color);
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        if glyph.renderable(context) {
                            context.transform_stack.push(&transform);
                            glyph.render(context);
                            context.transform_stack.pop();
                        }

                        transform.matrix.tx += Twips::new(c.advance);
                    }
                }
            }
        }
        context.transform_stack.pop();
    }

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        self.0.shared.get().bounds
    }

    fn hit_test_shape(
        self,
        context: &mut UpdateContext<'gc>,
        mut point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.world_bounds(BoundsMode::Engine).contains(point)
        {
            // Texts using the "Advanced text rendering" always hit test using their bounding box.
            if self.0.render_settings.borrow().is_advanced() {
                return true;
            }

            let shared = self.0.shared.get();

            // Transform the point into the text's local space.
            let Some(local_matrix) = self.global_to_local_matrix() else {
                return false;
            };
            let Some(text_matrix) = shared.text_transform.inverse() else {
                return false;
            };
            point = text_matrix * local_matrix * point;

            let mut font_id = 0;
            let mut height = Twips::ZERO;
            let mut glyph_matrix = Matrix::default();
            for block in &shared.text_blocks {
                if let Some(x) = block.x_offset {
                    glyph_matrix.tx = x;
                }
                if let Some(y) = block.y_offset {
                    glyph_matrix.ty = y;
                }
                font_id = block.font_id.unwrap_or(font_id);
                height = block.height.unwrap_or(height);

                if let Some(font) = context
                    .library
                    .library_for_movie_gc(self.movie(), context.gc())
                    .unwrap()
                    .borrow()
                    .get_font(font_id)
                {
                    let scale = (height.get() as f32) / font.scale();
                    glyph_matrix.a = scale;
                    glyph_matrix.d = scale;
                    for c in &block.glyphs {
                        if let Some(glyph) = font.get_glyph(c.index as usize) {
                            // Transform the point into glyph space and test.
                            let Some(matrix) = glyph_matrix.inverse() else {
                                return false;
                            };
                            let point = matrix * point;
                            if glyph.hit_test(point, &local_matrix) {
                                return true;
                            }

                            glyph_matrix.tx += Twips::new(c.advance);
                        }
                    }
                }
            }
        }

        false
    }

    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && self.object2().is_none() {
            let statictext = context.avm2.classes().statictext;

            let object = Avm2StageObject::for_display_object(context.gc(), self.into(), statictext);
            // We don't need to call the initializer method, as AVM2 can't link
            // a custom class to a StaticText, and the initializer method for
            // StaticText itself is a no-op
            self.set_object2(context, object);

            self.on_construction_complete(context);
        }
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        if self.movie().is_action_script_3() {
            self.set_default_instance_name(context);
        }
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), TextData, avm2_object).set(Some(to));
    }
}

/// Data shared between all instances of a text object.
#[derive(Debug, Collect)]
#[collect(no_drop)]
struct TextShared<'gc> {
    swf: SwfMovieGc<'gc>,
    #[collect(require_static)]
    id: CharacterId,
    #[collect(require_static)]
    bounds: Rectangle<Twips>,
    #[collect(require_static)]
    text_transform: Matrix,
    #[collect(require_static)]
    text_blocks: Vec<swf::TextRecord>,
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextSnapshot<'gc>(Gc<'gc, TextSnapshotData<'gc>>);

impl fmt::Debug for TextSnapshot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TextSnapshot")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct TextSnapshotData<'gc> {
    chunks: Vec<TextSnapshotChunk<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
struct TextSnapshotChunk<'gc> {
    object: Text<'gc>,
    #[collect(require_static)]
    text: WString,
    global_index: usize,
}

impl<'gc> TextSnapshotChunk<'gc> {
    fn sub_string(&self, global_index_start: usize, global_index_end: usize) -> &WStr {
        let start = global_index_start.saturating_sub(self.global_index);
        let end = global_index_end
            .saturating_sub(self.global_index)
            .min(self.text.len());
        &self.text[start..end]
    }
}

impl<'gc> TextSnapshot<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>, target: MovieClip<'gc>) -> Self {
        let mut chunks = Vec::new();
        let mut index = 0;
        for child in target.iter_render_list() {
            if let Some(object) = child.as_text()
                && let Some(text) = object.text(context)
            {
                let len = text.len();
                chunks.push(TextSnapshotChunk {
                    object,
                    text,
                    global_index: index,
                });
                index += len;
            }
        }

        Self(Gc::new(context.gc(), TextSnapshotData { chunks }))
    }

    pub fn count(self) -> usize {
        self.0.chunks.iter().map(|c| c.text.len()).sum()
    }

    pub fn get_text(self, from: i32, to: i32, include_newlines: bool) -> WString {
        let count = self.count();
        if count == 0 {
            return WString::new();
        }

        let start = usize::try_from(from).unwrap_or_default().min(count - 1);
        let end = usize::try_from(to)
            .unwrap_or_default()
            .min(count)
            .max(start + 1);

        let mut chunks = self
            .0
            .chunks
            .iter()
            .filter(|c| c.global_index < end)
            .filter(|c| c.global_index + c.text.len() > start)
            .map(|c| c.sub_string(start, end));

        let mut ret = WString::new();

        if let Some(chunk) = chunks.next() {
            ret.push_str(chunk);
        }

        for chunk in chunks {
            if include_newlines {
                ret.push_char('\n');
            }
            ret.push_str(chunk);
        }

        ret
    }

    pub fn find_text(self, from: i32, text: &WStr, case_sensitive: bool) -> i32 {
        if text.is_empty() {
            return -1;
        }

        let Ok(from) = usize::try_from(from) else {
            return -1;
        };
        let count = self.count();

        let chunks = self
            .0
            .chunks
            .iter()
            .filter(|c| c.global_index + c.text.len() > from)
            .map(|c| c.sub_string(from, count));

        let mut full_text = WString::new();
        for chunk in chunks {
            full_text.push_str(chunk);
        }

        let text = if !case_sensitive {
            full_text.make_ascii_lowercase();
            Cow::Owned(text.to_ascii_lowercase())
        } else {
            Cow::Borrowed(text)
        };

        let Some(index) = full_text.find(text.as_ref()) else {
            return -1;
        };

        i32::try_from(from + index).unwrap_or(-1)
    }
}
