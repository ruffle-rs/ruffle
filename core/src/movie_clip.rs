use crate::audio::AudioStreamHandle;
use crate::character::Character;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::font::Font;
use crate::graphic::Graphic;
use crate::matrix::Matrix;
use crate::morph_shape::MorphShape;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::text::Text;
use std::collections::{BTreeMap, HashMap};
use swf::read::SwfRead;

type Depth = i16;
type FrameNumber = u16;

#[derive(Clone)]
pub struct MovieClip<'gc> {
    base: DisplayObjectBase,
    tag_stream_start: Option<u64>,
    tag_stream_pos: u64,
    is_playing: bool,
    action: Option<(usize, usize)>,
    goto_queue: Vec<FrameNumber>,
    current_frame: FrameNumber,
    total_frames: FrameNumber,

    audio_stream: Option<AudioStreamHandle>,
    stream_started: bool,

    children: BTreeMap<Depth, DisplayNode<'gc>>,
}

impl<'gc> MovieClip<'gc> {
    pub fn new() -> Self {
        Self {
            base: Default::default(),
            tag_stream_start: None,
            tag_stream_pos: 0,
            is_playing: true,
            action: None,
            goto_queue: Vec::new(),
            current_frame: 0,
            total_frames: 1,
            audio_stream: None,
            stream_started: false,
            children: BTreeMap::new(),
        }
    }

    pub fn new_with_data(tag_stream_start: u64, num_frames: u16) -> Self {
        Self {
            base: Default::default(),
            tag_stream_start: Some(tag_stream_start),
            tag_stream_pos: tag_stream_start,
            is_playing: true,
            action: None,
            goto_queue: Vec::new(),
            current_frame: 0,
            audio_stream: None,
            stream_started: false,
            total_frames: num_frames,
            children: BTreeMap::new(),
        }
    }

    pub fn playing(&self) -> bool {
        self.is_playing
    }

    pub fn next_frame(&mut self) {
        if self.current_frame < self.total_frames {
            self.goto_frame(self.current_frame + 1, true);
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn prev_frame(&mut self) {
        if self.current_frame > 1 {
            self.goto_frame(self.current_frame - 1, true);
        }
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn goto_frame(&mut self, frame: FrameNumber, stop: bool) {
        self.goto_queue.push(frame);

        if stop {
            self.stop();
        } else {
            self.play();
        }
    }

    pub fn x(&self) -> f32 {
        self.get_matrix().tx
    }

    pub fn y(&self) -> f32 {
        self.get_matrix().ty
    }

    pub fn x_scale(&self) -> f32 {
        self.get_matrix().a * 100.0
    }

    pub fn y_scale(&self) -> f32 {
        self.get_matrix().d * 100.0
    }

    pub fn current_frame(&self) -> FrameNumber {
        self.current_frame
    }

    pub fn total_frames(&self) -> FrameNumber {
        self.total_frames
    }

    pub fn frames_loaded(&self) -> FrameNumber {
        // TODO(Herschel): root needs to progressively stream in frames.
        self.total_frames
    }

    pub fn get_child_by_name(&self, name: &str) -> Option<&DisplayNode<'gc>> {
        self.children
            .values()
            .find(|child| child.read().name() == name)
    }

    pub fn frame_label_to_number(
        &self,
        frame_label: &str,
        context: &mut UpdateContext,
    ) -> Option<FrameNumber> {
        // TODO(Herschel): We should cache the labels in the preload step.
        let pos = context.tag_stream.get_ref().position();
        context
            .tag_stream
            .get_inner()
            .set_position(self.tag_stream_start.unwrap());
        use swf::Tag;
        let mut frame_num = 1;
        while let Ok(Some(tag)) = context.tag_stream.read_tag() {
            match tag {
                Tag::FrameLabel { label, .. } => {
                    if label == frame_label {
                        context.tag_stream.get_inner().set_position(pos);
                        return Some(frame_num);
                    }
                }
                Tag::ShowFrame => frame_num += 1,
                _ => (),
            }
        }
        context.tag_stream.get_inner().set_position(pos);
        None
    }

    pub fn action(&self) -> Option<(usize, usize)> {
        self.action
    }

    pub fn run_goto_queue(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut i = 0;
        while i < self.goto_queue.len() {
            let frame = self.goto_queue[i];
            if frame >= self.current_frame {
                // Advancing
                while self.current_frame + 1 < frame {
                    self.run_frame_internal(context, true);
                }
                self.run_frame_internal(context, false);
            } else {
                // Rewind
                // Reset everything to blank, start from frame 1,
                // and advance forward
                self.children.clear();
                self.tag_stream_pos = self.tag_stream_start.unwrap_or(0);
                self.current_frame = 0;
                while self.current_frame + 1 < frame {
                    self.run_frame_internal(context, true);
                }
                self.run_frame_internal(context, false);
            }

            i += 1;
        }

        self.goto_queue.clear();
    }

    pub fn place_object(
        &mut self,
        place_object: &swf::PlaceObject,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        use swf::PlaceObjectAction;
        let character = match place_object.action {
            PlaceObjectAction::Place(id) => {
                // TODO(Herschel): Behavior when character doesn't exist/isn't a DisplayObject?
                let character = if let Ok(character) = context
                    .library
                    .instantiate_display_object(id, context.gc_context)
                {
                    character
                } else {
                    return;
                };

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                self.children.insert(place_object.depth, character);
                self.children.get_mut(&place_object.depth).unwrap()
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = self.children.get_mut(&place_object.depth) {
                    child
                } else {
                    return;
                }
            }
            PlaceObjectAction::Replace(id) => {
                let character = if let Ok(character) = context
                    .library
                    .instantiate_display_object(id, context.gc_context)
                {
                    character
                } else {
                    return;
                };

                let prev_character = self.children.insert(place_object.depth, character);
                let character = self.children.get_mut(&place_object.depth).unwrap();
                if let Some(prev_character) = prev_character {
                    character
                        .write(context.gc_context)
                        .set_matrix(prev_character.read().get_matrix());
                    character
                        .write(context.gc_context)
                        .set_color_transform(prev_character.read().get_color_transform());
                }
                character
            }
        };

        if let Some(matrix) = &place_object.matrix {
            let m = matrix.clone();
            character
                .write(context.gc_context)
                .set_matrix(&Matrix::from(m));
        }

        if let Some(color_transform) = &place_object.color_transform {
            character
                .write(context.gc_context)
                .set_color_transform(&ColorTransform::from(color_transform.clone()));
        }

        if let Some(name) = &place_object.name {
            character.write(context.gc_context).set_name(name);
        }

        if let Some(ratio) = &place_object.ratio {
            if let Some(morph_shape) = character.write(context.gc_context).as_morph_shape_mut() {
                morph_shape.set_ratio(*ratio);
            }
        }

        if let Some(clip_depth) = &place_object.clip_depth {
            character
                .write(context.gc_context)
                .set_clip_depth(*clip_depth);
        }
    }

    fn run_frame_internal(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        only_display_actions: bool,
    ) {
        use swf::Tag;

        // Advance frame number.
        if self.current_frame < self.total_frames {
            self.current_frame += 1;
        } else {
            self.current_frame = 1;
            self.children.clear();
            if let Some(start) = self.tag_stream_start {
                self.tag_stream_pos = start;
            }
        }

        context
            .tag_stream
            .get_inner()
            .set_position(self.tag_stream_pos);

        let mut start_pos = self.tag_stream_pos;

        while let Ok(Some(tag)) = context.tag_stream.read_tag() {
            if only_display_actions {
                match tag {
                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => self.place_object(&*place_object, context),
                    Tag::RemoveObject { depth, .. } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth);
                    }

                    // All non-display-list tags get ignored.
                    _ => (),
                }
            } else {
                match tag {
                    // Definition Tags
                    Tag::SetBackgroundColor(color) => *context.background_color = color,

                    // Control Tags
                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => self.place_object(&*place_object, context),
                    Tag::RemoveObject { depth, .. } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth);
                    }

                    Tag::StartSound { id, .. } => {
                        if let Some(handle) = context.library.get_sound(id) {
                            context.audio.play_sound(handle);
                        }
                    }

                    Tag::SoundStreamHead(info) => self.sound_stream_head(&info, context, 0, 1),
                    Tag::SoundStreamHead2(info) => self.sound_stream_head(&info, context, 0, 2),
                    Tag::SoundStreamBlock(samples) => {
                        self.sound_stream_block(&samples[..], context, 0)
                    }

                    Tag::JpegTables(_) => (),
                    // Tag::DoAction(data) => self.do_action(context, &data[..]),
                    Tag::DoAction(data) => {
                        let pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(start_pos);
                        context.tag_stream.read_tag_code_and_length().unwrap();
                        let start_pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(pos);
                        self.action = Some((start_pos as usize, data.len()));
                    }
                    _ => (), // info!("Umimplemented tag: {:?}", tag),
                }
                start_pos = context.tag_stream.get_ref().position();
            }
        }

        self.tag_stream_pos = context.tag_stream.get_ref().position();
    }

    fn sound_stream_head(
        &mut self,
        _stream_info: &swf::SoundStreamInfo,
        _context: &mut UpdateContext,
        _length: usize,
        _version: u8,
    ) {
        if self.audio_stream.is_some() {
            //self.audio_stream = Some(context.audio.register_stream(stream_info));
            self.stream_started = false;
        }
    }

    fn sound_stream_block(&mut self, samples: &[u8], context: &mut UpdateContext, _length: usize) {
        if let Some(stream) = self.audio_stream {
            if !self.stream_started {
                self.stream_started = context.audio.start_stream(stream);
            }
            context.audio.queue_stream_samples(stream, samples);
        }
    }

    fn preload_sound_stream_head(
        &mut self,
        stream_info: &swf::SoundStreamInfo,
        context: &mut UpdateContext,
        _length: usize,
        _version: u8,
    ) {
        if self.audio_stream.is_none() {
            self.audio_stream = Some(context.audio.register_stream(stream_info));
        }
    }

    fn preload_sound_stream_block(
        &mut self,
        samples: &[u8],
        context: &mut UpdateContext,
        _length: usize,
    ) {
        if let Some(stream) = self.audio_stream {
            context.audio.preload_stream_samples(stream, samples)
        }
    }
}

impl<'gc> DisplayObject<'gc> for MovieClip<'gc> {
    impl_display_object!(base);

    fn preload(&mut self, context: &mut UpdateContext) {
        context
            .tag_stream
            .get_inner()
            .set_position(self.tag_stream_start.unwrap());

        let mut ids = HashMap::new();
        use swf::Tag;
        let mut start_pos = context.tag_stream.get_ref().position();
        while let Ok(Some(tag)) = context.tag_stream.read_tag() {
            match tag {
                // Definition Tags
                Tag::DefineButton2(swf_button) => {
                    if !context.library.contains_character(swf_button.id) {
                        let button = crate::button::Button::from_swf_tag(
                            &swf_button,
                            &context.library,
                            context.gc_context,
                        );
                        context
                            .library
                            .register_character(swf_button.id, Character::Button(Box::new(button)));
                    }
                }
                Tag::DefineBits { id, jpeg_data } => {
                    if !context.library.contains_character(id) {
                        let handle = context.renderer.register_bitmap_jpeg(
                            id,
                            &jpeg_data,
                            context.library.jpeg_tables().unwrap(),
                        );
                        context
                            .library
                            .register_character(id, Character::Bitmap(handle));
                    }
                }
                Tag::DefineBitsJpeg2 { id, jpeg_data } => {
                    if !context.library.contains_character(id) {
                        let handle = context.renderer.register_bitmap_jpeg_2(id, &jpeg_data);
                        context
                            .library
                            .register_character(id, Character::Bitmap(handle));
                    }
                }
                Tag::DefineBitsLossless(bitmap) => {
                    if !context.library.contains_character(bitmap.id) {
                        let handle = context.renderer.register_bitmap_png(&bitmap);
                        context
                            .library
                            .register_character(bitmap.id, Character::Bitmap(handle));
                    }
                }
                Tag::DefineFont(font) => {
                    if !context.library.contains_character(font.id) {
                        let glyphs = font
                            .glyphs
                            .into_iter()
                            .map(|g| swf::Glyph {
                                shape_records: g,
                                code: 0,
                                advance: None,
                                bounds: None,
                            })
                            .collect::<Vec<_>>();

                        let font = swf::Font {
                            id: font.id,
                            version: 0,
                            name: "".to_string(),
                            glyphs,
                            language: swf::Language::Unknown,
                            layout: None,
                            is_small_text: false,
                            is_shift_jis: false,
                            is_ansi: false,
                            is_bold: false,
                            is_italic: false,
                        };
                        let font_object = Font::from_swf_tag(context, &font).unwrap();
                        context
                            .library
                            .register_character(font.id, Character::Font(Box::new(font_object)));
                    }
                }
                Tag::DefineFont2(font) => {
                    if !context.library.contains_character(font.id) {
                        let font_object = Font::from_swf_tag(context, &font).unwrap();
                        context
                            .library
                            .register_character(font.id, Character::Font(Box::new(font_object)));
                    }
                }
                Tag::DefineFontInfo(_) => {
                    // TODO(Herschel)
                }
                Tag::DefineMorphShape(swf_shape) => {
                    if !context.library.contains_character(swf_shape.id) {
                        let morph_shape = MorphShape::from_swf_tag(&swf_shape, context);
                        context.library.register_character(
                            swf_shape.id,
                            Character::MorphShape(Box::new(morph_shape)),
                        );
                    }
                }
                Tag::DefineShape(swf_shape) => {
                    if !context.library.contains_character(swf_shape.id) {
                        let graphic = Graphic::from_swf_tag(&swf_shape, context);
                        context.library.register_character(
                            swf_shape.id,
                            Character::Graphic(Box::new(graphic)),
                        );
                    }
                }
                Tag::DefineSound(sound) => {
                    if !context.library.contains_character(sound.id) {
                        let handle = context.audio.register_sound(&sound).unwrap();
                        context
                            .library
                            .register_character(sound.id, Character::Sound(handle));
                    }
                }
                Tag::DefineSprite(swf_sprite) => {
                    let pos = context.tag_stream.get_ref().position();
                    context.tag_stream.get_inner().set_position(start_pos);
                    context.tag_stream.read_tag_code_and_length().unwrap();
                    context.tag_stream.read_u32().unwrap();
                    let mc_start_pos = context.tag_stream.get_ref().position();
                    context.tag_stream.get_inner().set_position(pos);
                    if !context.library.contains_character(swf_sprite.id) {
                        let mut movie_clip =
                            MovieClip::new_with_data(mc_start_pos, swf_sprite.num_frames);

                        movie_clip.preload(context);

                        context.library.register_character(
                            swf_sprite.id,
                            Character::MovieClip(Box::new(movie_clip)),
                        );
                    }
                }
                Tag::DefineText(text) => {
                    if !context.library.contains_character(text.id) {
                        let text_object = Text::from_swf_tag(&text);
                        context
                            .library
                            .register_character(text.id, Character::Text(Box::new(text_object)));
                    }
                }

                Tag::PlaceObject(place_object) => {
                    use swf::PlaceObjectAction;
                    match place_object.action {
                        PlaceObjectAction::Place(id) | PlaceObjectAction::Replace(id) => {
                            ids.insert(place_object.depth, id);
                        }
                        _ => (),
                    }
                    if let Some(ratio) = place_object.ratio {
                        if let Some(&id) = ids.get(&place_object.depth) {
                            if let Some(Character::MorphShape(morph_shape)) =
                                context.library.get_character_mut(id)
                            {
                                morph_shape.register_ratio(context.renderer, ratio);
                            }
                        }
                    }
                }

                Tag::SoundStreamHead(info) => self.preload_sound_stream_head(&info, context, 0, 1),
                Tag::SoundStreamHead2(info) => self.preload_sound_stream_head(&info, context, 0, 2),
                Tag::SoundStreamBlock(samples) => {
                    self.preload_sound_stream_block(&samples[..], context, 0)
                }

                Tag::JpegTables(data) => context.library.set_jpeg_tables(data),
                _ => (),
            }
            start_pos = context.tag_stream.get_inner().position();
        }

        if let Some(stream) = self.audio_stream {
            context.audio.preload_stream_finalize(stream);
        }
    }

    fn run_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.action = None;
        if self.tag_stream_start.is_some() {
            context
                .position_stack
                .push(context.tag_stream.get_ref().position());
        }

        if self.is_playing {
            self.run_frame_internal(context, false);
        }

        // TODO(Herschel): Verify order of execution for parent/children.
        // Parent first? Children first? Sorted by depth?
        for child in self.children.values_mut() {
            child.write(context.gc_context).run_frame(context);
        }

        if self.tag_stream_start.is_some() {
            context
                .tag_stream
                .get_inner()
                .set_position(context.position_stack.pop().unwrap());
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.run_goto_queue(context);

        //for child in self.children.values() {
        //child.borrow_mut().run_post_frame(context);
        //}
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(self.transform());

        for child in self.children.values() {
            child.read().render(context);
        }

        context.transform_stack.pop();
    }

    fn handle_click(&mut self, _pos: (f32, f32)) {
        // for child in self.children.values_mut() {
        //     child.handle_click(pos);
        // }
    }
    fn as_movie_clip(&self) -> Option<&crate::movie_clip::MovieClip<'gc>> {
        Some(self)
    }

    fn as_movie_clip_mut(&mut self) -> Option<&mut crate::movie_clip::MovieClip<'gc>> {
        Some(self)
    }
}

impl Default for MovieClip<'_> {
    fn default() -> Self {
        MovieClip::new()
    }
}

unsafe impl<'gc> gc_arena::Collect for MovieClip<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for child in self.children.values() {
            child.trace(cc);
        }
    }
}
