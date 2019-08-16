use crate::backend::audio::AudioStreamHandle;
use crate::character::Character;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::font::Font;
use crate::graphic::Graphic;
use crate::matrix::Matrix;
use crate::morph_shape::MorphShape;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::tag_utils::{self, DecodeResult, SwfStream};
use crate::text::Text;
use std::collections::BTreeMap;
use swf::read::SwfRead;

type Depth = i16;
type FrameNumber = u16;

#[derive(Clone)]
pub struct MovieClip<'gc> {
    base: DisplayObjectBase,
    id: CharacterId,
    tag_stream_start: u64,
    tag_stream_pos: u64,
    tag_stream_len: usize,
    is_playing: bool,
    goto_queue: Vec<FrameNumber>,
    current_frame: FrameNumber,
    total_frames: FrameNumber,

    audio_stream_info: Option<swf::SoundStreamHead>,
    audio_stream: Option<AudioStreamHandle>,

    children: BTreeMap<Depth, DisplayNode<'gc>>,
}

impl<'gc> MovieClip<'gc> {
    pub fn new() -> Self {
        Self {
            base: Default::default(),
            id: 0,
            tag_stream_start: 0,
            tag_stream_pos: 0,
            tag_stream_len: 0,
            is_playing: true,
            goto_queue: Vec::new(),
            current_frame: 0,
            total_frames: 1,
            audio_stream: None,
            audio_stream_info: None,
            children: BTreeMap::new(),
        }
    }

    pub fn new_with_data(
        id: CharacterId,
        tag_stream_start: u64,
        tag_stream_len: usize,
        num_frames: u16,
    ) -> Self {
        Self {
            base: Default::default(),
            id,
            tag_stream_start,
            tag_stream_pos: 0,
            tag_stream_len,
            is_playing: true,
            goto_queue: Vec::new(),
            current_frame: 0,
            audio_stream: None,
            audio_stream_info: None,
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
        _frame_label: &str,
        _context: &mut UpdateContext<'_, '_, '_>,
    ) -> Option<FrameNumber> {
        // TODO(Herschel): We should cache the labels in the preload step.
        // let mut reader = self.reader(context);
        // use swf::Tag;
        // let mut frame_num = 1;
        // while let Ok(Some(tag)) = reader.read_tag() {
        //     match tag {
        //         Tag::FrameLabel { label, .. } => {
        //             if label == frame_label {
        //                 return Some(frame_num);
        //             }
        //         }
        //         Tag::ShowFrame => frame_num += 1,
        //         _ => (),
        //     }
        // }
        None
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
                self.tag_stream_pos = 0;
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

    fn reader<'a>(
        &self,
        context: &UpdateContext<'a, '_, '_>,
    ) -> swf::read::Reader<std::io::Cursor<&'a [u8]>> {
        let mut cursor = std::io::Cursor::new(
            &context.swf_data[self.tag_stream_start as usize
                ..self.tag_stream_start as usize + self.tag_stream_len],
        );
        cursor.set_position(self.tag_stream_pos);
        swf::read::Reader::new(cursor, context.swf_version)
    }
    fn run_frame_internal(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        only_display_actions: bool,
    ) {
        // Advance frame number.
        if self.current_frame < self.total_frames {
            self.current_frame += 1;
        } else {
            self.current_frame = 1;
            self.children.clear();
            self.tag_stream_pos = 0;
        }

        let _tag_pos = self.tag_stream_pos;
        let mut reader = self.reader(context);

        use swf::TagCode;
        if only_display_actions {
            let tag_callback = |reader: &mut _, tag_code, tag_len| match tag_code {
                TagCode::PlaceObject => self.place_object(context, reader, tag_len, 1),
                TagCode::PlaceObject2 => self.place_object(context, reader, tag_len, 2),
                TagCode::PlaceObject3 => self.place_object(context, reader, tag_len, 3),
                TagCode::PlaceObject4 => self.place_object(context, reader, tag_len, 4),
                TagCode::RemoveObject => self.remove_object(context, reader, 1),
                TagCode::RemoveObject2 => self.remove_object(context, reader, 2),
                _ => Ok(()),
            };
            let _ = tag_utils::decode_tags(&mut reader, tag_callback, TagCode::ShowFrame);
        } else {
            let tag_callback = |reader: &mut _, tag_code, tag_len| match tag_code {
                TagCode::DoAction => self.do_action(context, reader, tag_len),
                TagCode::PlaceObject => self.place_object(context, reader, tag_len, 1),
                TagCode::PlaceObject2 => self.place_object(context, reader, tag_len, 2),
                TagCode::PlaceObject3 => self.place_object(context, reader, tag_len, 3),
                TagCode::PlaceObject4 => self.place_object(context, reader, tag_len, 4),
                TagCode::RemoveObject => self.remove_object(context, reader, 1),
                TagCode::RemoveObject2 => self.remove_object(context, reader, 2),
                TagCode::SetBackgroundColor => self.set_background_color(context, reader),
                TagCode::StartSound => self.start_sound_1(context, reader),
                TagCode::SoundStreamBlock => self.sound_stream_block(context, reader),
                _ => Ok(()),
            };
            let _ = tag_utils::decode_tags(&mut reader, tag_callback, TagCode::ShowFrame);
        }

        self.tag_stream_pos = reader.get_ref().position();
    }
}

impl<'gc> DisplayObject<'gc> for MovieClip<'gc> {
    impl_display_object!(base);

    fn preload(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Ignore error for now.
        use swf::TagCode;
        let mut reader = self.reader(context);
        let mut ids = fnv::FnvHashMap::default();
        let tag_callback = |reader: &mut _, tag_code, tag_len| match tag_code {
            TagCode::DefineBits => self.define_bits(context, reader, tag_len),
            TagCode::DefineBitsJpeg2 => self.define_bits_jpeg_2(context, reader, tag_len),
            TagCode::DefineBitsJpeg3 => self.define_bits_jpeg_3(context, reader, tag_len),
            TagCode::DefineBitsJpeg4 => self.define_bits_jpeg_4(context, reader, tag_len),
            TagCode::DefineBitsLossless => self.define_bits_lossless(context, reader, 1),
            TagCode::DefineBitsLossless2 => self.define_bits_lossless(context, reader, 2),
            TagCode::DefineButton => self.define_button_1(context, reader),
            TagCode::DefineButton2 => self.define_button_2(context, reader),
            TagCode::DefineFont => self.define_font_1(context, reader),
            TagCode::DefineFont2 => self.define_font_2(context, reader),
            TagCode::DefineFont3 => self.define_font_3(context, reader),
            TagCode::DefineFont4 => unimplemented!(),
            TagCode::DefineMorphShape => self.define_morph_shape(context, reader, 1),
            TagCode::DefineMorphShape2 => self.define_morph_shape(context, reader, 2),
            TagCode::DefineShape => self.define_shape(context, reader, 1),
            TagCode::DefineShape2 => self.define_shape(context, reader, 2),
            TagCode::DefineShape3 => self.define_shape(context, reader, 3),
            TagCode::DefineShape4 => self.define_shape(context, reader, 4),
            TagCode::DefineSound => self.define_sound(context, reader, tag_len),
            TagCode::DefineSprite => self.define_sprite(context, reader, tag_len),
            TagCode::DefineText => self.define_text(context, reader),
            TagCode::JpegTables => self.jpeg_tables(context, reader, tag_len),
            TagCode::PlaceObject => self.preload_place_object(context, reader, tag_len, &mut ids, 1),
            TagCode::PlaceObject2 => self.preload_place_object(context, reader, tag_len, &mut ids, 2),
            TagCode::PlaceObject3 => self.preload_place_object(context, reader, tag_len, &mut ids, 3),
            TagCode::PlaceObject4 => self.preload_place_object(context, reader, tag_len, &mut ids, 4),
            TagCode::RemoveObject => self.preload_remove_object(context, reader, &mut ids, 1),
            TagCode::RemoveObject2 => self.preload_remove_object(context, reader, &mut ids, 2),
            TagCode::SoundStreamHead => self.preload_sound_stream_head(context, reader, 1),
            TagCode::SoundStreamHead2 => self.preload_sound_stream_head(context, reader, 2),
            TagCode::SoundStreamBlock => self.preload_sound_stream_block(context, reader, tag_len),
            _ => Ok(()),
        };
        let _ = tag_utils::decode_tags(&mut reader, tag_callback, TagCode::End);
        if self.audio_stream_info.is_some() {
            context.audio.preload_sound_stream_end(self.id);
        }
    }

    fn run_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.is_playing {
            self.run_frame_internal(context, false);
        }

        // TODO(Herschel): Verify order of execution for parent/children.
        // Parent first? Children first? Sorted by depth?
        for child in self.children.values_mut() {
            child.write(context.gc_context).run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.run_goto_queue(context);

        for child in self.children.values() {
            child.write(context.gc_context).run_post_frame(context);
        }
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

// Preloading of definition tags
impl<'gc, 'a> MovieClip<'gc> {
    #[inline]
    fn define_bits_lossless(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        version: u8,
    ) -> DecodeResult {
        let define_bits_lossless = reader.read_define_bits_lossless(version)?;
        let handle = context.renderer.register_bitmap_png(&define_bits_lossless);
        context
            .library
            .register_character(define_bits_lossless.id, Character::Bitmap(handle));
        Ok(())
    }

    #[inline]
    fn define_morph_shape(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        version: u8,
    ) -> DecodeResult {
        let swf_shape = reader.read_define_morph_shape(version)?;
        let morph_shape = MorphShape::from_swf_tag(&swf_shape, context.renderer);
        context
            .library
            .register_character(swf_shape.id, Character::MorphShape(Box::new(morph_shape)));
        Ok(())
    }

    #[inline]
    fn define_shape(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        version: u8,
    ) -> DecodeResult {
        let swf_shape = reader.read_define_shape(version)?;
        let graphic = Graphic::from_swf_tag(&swf_shape, context.renderer);
        context
            .library
            .register_character(swf_shape.id, Character::Graphic(Box::new(graphic)));
        Ok(())
    }

    #[inline]
    fn preload_place_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
        ids: &mut fnv::FnvHashMap<Depth, CharacterId>,
        version: u8,
    ) -> DecodeResult {
        use swf::PlaceObjectAction;
        let place_object = if version == 1 {
            reader.read_place_object(tag_len)
        } else {
            reader.read_place_object_2_or_3(version)
        }?;
        match place_object.action {
            PlaceObjectAction::Place(id) => {
                if let Some(Character::MorphShape(morph_shape)) =
                    context.library.get_character_mut(id)
                {
                    ids.insert(place_object.depth, id);
                    if let Some(ratio) = place_object.ratio {
                        morph_shape.register_ratio(context.renderer, ratio);
                    }
                }
            }
            PlaceObjectAction::Modify => {
                if let Some(&id) = ids.get(&place_object.depth) {
                    if let Some(Character::MorphShape(morph_shape)) =
                        context.library.get_character_mut(id)
                    {
                        ids.insert(place_object.depth, id);
                        if let Some(ratio) = place_object.ratio {
                            morph_shape.register_ratio(context.renderer, ratio);
                        }
                    }
                }
            }
            PlaceObjectAction::Replace(id) => {
                if let Some(Character::MorphShape(morph_shape)) =
                    context.library.get_character_mut(id)
                {
                    ids.insert(place_object.depth, id);
                    if let Some(ratio) = place_object.ratio {
                        morph_shape.register_ratio(context.renderer, ratio);
                    }
                } else {
                    ids.remove(&place_object.depth);
                }
            }
        };

        Ok(())
    }

    #[inline]
    fn preload_sound_stream_block(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        if self.audio_stream_info.is_some() {
            let pos = reader.get_ref().position() as usize;
            let data = reader.get_ref().get_ref();
            let data = &data[pos..pos + tag_len];
            context.audio.preload_sound_stream_block(self.id, data);
        }

        Ok(())
    }

    #[inline]
    fn preload_sound_stream_head(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        _version: u8,
    ) -> DecodeResult {
        let audio_stream_info = reader.read_sound_stream_head()?;
        context
            .audio
            .preload_sound_stream_head(self.id, &audio_stream_info);
        self.audio_stream_info = Some(audio_stream_info);
        Ok(())
    }

    #[inline]
    fn define_bits(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        use std::io::Read;
        let id = reader.read_u16()?;
        let data_len = tag_len - 2;
        let mut jpeg_data = Vec::with_capacity(data_len);
        reader
            .get_mut()
            .take(data_len as u64)
            .read_to_end(&mut jpeg_data)?;
        let handle = context.renderer.register_bitmap_jpeg(
            id,
            &jpeg_data,
            context.library.jpeg_tables().unwrap(),
        );
        context
            .library
            .register_character(id, Character::Bitmap(handle));
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        use std::io::Read;
        let id = reader.read_u16()?;
        let data_len = tag_len - 2;
        let mut jpeg_data = Vec::with_capacity(data_len);
        reader
            .get_mut()
            .take(data_len as u64)
            .read_to_end(&mut jpeg_data)?;
        let handle = context.renderer.register_bitmap_jpeg_2(id, &jpeg_data);
        context
            .library
            .register_character(id, Character::Bitmap(handle));
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_3(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        use std::io::Read;
        let id = reader.read_u16()?;
        let jpeg_len = reader.read_u32()? as usize;
        let alpha_len = tag_len - 6 - jpeg_len;
        let mut jpeg_data = Vec::with_capacity(jpeg_len);
        let mut alpha_data = Vec::with_capacity(alpha_len);
        reader
            .get_mut()
            .take(jpeg_len as u64)
            .read_to_end(&mut jpeg_data)?;
        reader
            .get_mut()
            .take(alpha_len as u64)
            .read_to_end(&mut alpha_data)?;
        let handle = context
            .renderer
            .register_bitmap_jpeg_3(id, &jpeg_data, &alpha_data);
        context
            .library
            .register_character(id, Character::Bitmap(handle));
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_4(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        use std::io::Read;
        let id = reader.read_u16()?;
        let jpeg_len = reader.read_u32()? as usize;
        let _deblocking = reader.read_u16()?;
        let alpha_len = tag_len - 6 - jpeg_len;
        let mut jpeg_data = Vec::with_capacity(jpeg_len);
        let mut alpha_data = Vec::with_capacity(alpha_len);
        reader
            .get_mut()
            .take(jpeg_len as u64)
            .read_to_end(&mut jpeg_data)?;
        reader
            .get_mut()
            .take(alpha_len as u64)
            .read_to_end(&mut alpha_data)?;
        let handle = context
            .renderer
            .register_bitmap_jpeg_3(id, &jpeg_data, &alpha_data);
        context
            .library
            .register_character(id, Character::Bitmap(handle));
        Ok(())
    }

    #[inline]
    fn define_button_1(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let swf_button = reader.read_define_button_1()?;
        let button =
            crate::button::Button::from_swf_tag(&swf_button, &context.library, context.gc_context);
        context
            .library
            .register_character(swf_button.id, Character::Button(Box::new(button)));
        Ok(())
    }

    #[inline]
    fn define_button_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let swf_button = reader.read_define_button_2()?;
        let button =
            crate::button::Button::from_swf_tag(&swf_button, &context.library, context.gc_context);
        context
            .library
            .register_character(swf_button.id, Character::Button(Box::new(button)));
        Ok(())
    }

    #[inline]
    fn define_font_1(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let font = reader.read_define_font_1()?;
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
        let font_object = Font::from_swf_tag(context.renderer, &font).unwrap();
        context
            .library
            .register_character(font.id, Character::Font(Box::new(font_object)));
        Ok(())
    }

    #[inline]
    fn define_font_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let font = reader.read_define_font_2(2)?;
        let font_object = Font::from_swf_tag(context.renderer, &font).unwrap();
        context
            .library
            .register_character(font.id, Character::Font(Box::new(font_object)));
        Ok(())
    }

    #[inline]
    fn define_font_3(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let font = reader.read_define_font_2(3)?;
        let font_object = Font::from_swf_tag(context.renderer, &font).unwrap();
        context
            .library
            .register_character(font.id, Character::Font(Box::new(font_object)));

        Ok(())
    }

    #[inline]
    fn define_sound(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        // TODO(Herschel): Can we use a slice of the sound data instead of copying the data?
        use std::io::Read;
        let mut reader =
            swf::read::Reader::new(reader.get_mut().take(tag_len as u64), context.swf_version);
        let sound = reader.read_define_sound()?;
        let handle = context.audio.register_sound(&sound).unwrap();
        context
            .library
            .register_character(sound.id, Character::Sound(handle));
        Ok(())
    }

    fn define_sprite(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        let id = reader.read_character_id()?;
        let num_frames = reader.read_u16()?;
        let mut movie_clip =
            MovieClip::new_with_data(id, reader.get_ref().position(), tag_len - 4, num_frames);

        movie_clip.preload(context);

        context
            .library
            .register_character(id, Character::MovieClip(Box::new(movie_clip)));

        Ok(())
    }

    #[inline]
    fn define_text(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let text = reader.read_define_text()?;
        let text_object = Text::from_swf_tag(&text);
        context
            .library
            .register_character(text.id, Character::Text(Box::new(text_object)));
        Ok(())
    }

    #[inline]
    fn jpeg_tables(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        use std::io::Read;
        // TODO(Herschel): Can we use a slice instead of copying?
        let mut jpeg_data = Vec::with_capacity(tag_len);
        reader
            .get_mut()
            .take(tag_len as u64)
            .read_to_end(&mut jpeg_data)?;
        context.library.set_jpeg_tables(jpeg_data);
        Ok(())
    }

    #[inline]
    fn preload_remove_object(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        ids: &mut fnv::FnvHashMap<Depth, CharacterId>,
        version: u8,
    ) -> DecodeResult {
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;
        ids.remove(&remove_object.depth);
        Ok(())
    }
}

// Control tags
impl<'gc, 'a> MovieClip<'gc> {
    #[inline]
    fn do_action(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
    ) -> DecodeResult {
        // Queue the actions.
        let slice = crate::tag_utils::SwfSlice {
            data: std::sync::Arc::clone(context.swf_data),
            start: reader.get_ref().position() as usize,
            end: reader.get_ref().position() as usize + tag_len,
        };
        context.actions.push(slice);
        Ok(())
    }

    fn place_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        tag_len: usize,
        version: u8,
    ) -> DecodeResult {
        let place_object = if version == 1 {
            reader.read_place_object(tag_len)
        } else {
            reader.read_place_object_2_or_3(version)
        }?;
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
                    return Ok(());
                };

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                self.children.insert(place_object.depth, character);
                self.children.get_mut(&place_object.depth).unwrap()
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = self.children.get_mut(&place_object.depth) {
                    child
                } else {
                    return Ok(());
                }
            }
            PlaceObjectAction::Replace(id) => {
                let character = if let Ok(character) = context
                    .library
                    .instantiate_display_object(id, context.gc_context)
                {
                    character
                } else {
                    return Ok(());
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

        Ok(())
    }

    #[inline]
    fn remove_object(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
        version: u8,
    ) -> DecodeResult {
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;
        self.children.remove(&remove_object.depth);
        Ok(())
    }

    #[inline]
    fn set_background_color(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        *context.background_color = reader.read_rgb()?;
        Ok(())
    }

    #[inline]
    fn sound_stream_block(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        if let (Some(stream_info), None) = (&self.audio_stream_info, &self.audio_stream) {
            let slice = crate::tag_utils::SwfSlice {
                data: std::sync::Arc::clone(context.swf_data),
                start: self.tag_stream_start as usize,
                end: self.tag_stream_start as usize + self.tag_stream_len,
            };
            let audio_stream = context.audio.start_stream(self.id, slice, stream_info);
            self.audio_stream = Some(audio_stream);
        }

        Ok(())
    }

    #[inline]
    fn start_sound_1(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<&'a [u8]>,
    ) -> DecodeResult {
        let start_sound = reader.read_start_sound_1()?;
        if let Some(handle) = context.library.get_sound(start_sound.id) {
            context.audio.play_sound(handle);
        }
        Ok(())
    }
}
