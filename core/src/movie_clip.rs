use crate::audio::AudioStreamHandle;
use crate::character::Character;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObject, DisplayObjectBase, DisplayObjectImpl};
use crate::font::Font;
use crate::graphic::Graphic;
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use crate::text::Text;
use bacon_rajan_cc::{Cc, Trace, Tracer};
use log::info;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use swf::read::SwfRead;

type Depth = i16;
type FrameNumber = u16;

#[derive(Clone)]
pub struct MovieClip {
    base: DisplayObjectBase,
    tag_stream_start: Option<u64>,
    tag_stream_pos: u64,
    is_playing: bool,
    action: Option<(usize, usize)>,
    goto_queue: VecDeque<FrameNumber>,
    current_frame: FrameNumber,
    total_frames: FrameNumber,
    audio_stream: Option<AudioStreamHandle>,
    children: BTreeMap<Depth, Cc<RefCell<DisplayObject>>>,
}

impl MovieClip {
    pub fn new() -> MovieClip {
        MovieClip {
            base: Default::default(),
            tag_stream_start: None,
            tag_stream_pos: 0,
            is_playing: true,
            action: None,
            goto_queue: VecDeque::new(),
            current_frame: 0,
            total_frames: 1,
            audio_stream: None,
            children: BTreeMap::new(),
        }
    }

    pub fn new_with_data(tag_stream_start: u64, num_frames: u16) -> MovieClip {
        MovieClip {
            base: Default::default(),
            tag_stream_start: Some(tag_stream_start),
            tag_stream_pos: tag_stream_start,
            is_playing: true,
            action: None,
            goto_queue: VecDeque::new(),
            current_frame: 0,
            audio_stream: None,
            total_frames: num_frames,
            children: BTreeMap::new(),
        }
    }

    pub fn playing(&self) -> bool {
        self.is_playing
    }

    pub fn next_frame(&mut self) {
        if self.current_frame + 1 <= self.total_frames {
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
        self.goto_queue.push_back(frame);

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

    pub fn get_child_by_name(&self, name: &str) -> Option<&Cc<RefCell<DisplayObject>>> {
        self.children
            .values()
            .find(|child| child.borrow().name() == name)
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

    pub fn run_goto_queue(&mut self, context: &mut UpdateContext) {
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

    pub fn place_object(&mut self, place_object: &swf::PlaceObject, context: &mut UpdateContext) {
        use swf::PlaceObjectAction;
        let character = match place_object.action {
            PlaceObjectAction::Place(id) => {
                // TODO(Herschel): Behavior when character doesn't exist/isn't a DisplayObject?
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                self.children.insert(place_object.depth, character.clone());
                character
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = self.children.get(&place_object.depth) {
                    child.clone()
                } else {
                    return;
                }
            }
            PlaceObjectAction::Replace(id) => {
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                if let Some(prev_character) =
                    self.children.insert(place_object.depth, character.clone())
                {
                    character
                        .borrow_mut()
                        .set_matrix(prev_character.borrow().get_matrix());
                    character
                        .borrow_mut()
                        .set_color_transform(prev_character.borrow().get_color_transform());
                }
                character
            }
        };

        let mut character = character.borrow_mut();
        if let Some(matrix) = &place_object.matrix {
            let m = matrix.clone();
            character.set_matrix(&Matrix::from(m));
        }

        if let Some(color_transform) = &place_object.color_transform {
            character.set_color_transform(&ColorTransform::from(color_transform.clone()));
        }

        if let Some(name) = &place_object.name {
            character.set_name(name);
        }
    }

    fn run_frame_internal(&mut self, context: &mut UpdateContext, only_display_actions: bool) {
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

                    Tag::StartSound { id, sound_info } => {
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
        stream_info: &swf::SoundStreamInfo,
        context: &mut UpdateContext,
        _length: usize,
        _version: u8,
    ) {
        if self.audio_stream.is_none() {
            self.audio_stream = Some(context.audio.register_stream(stream_info));
        }
    }

    fn sound_stream_block(&mut self, samples: &[u8], context: &mut UpdateContext, _length: usize) {
        if let Some(stream) = self.audio_stream {
            context.audio.queue_stream_samples(stream, samples)
        }
    }
}

impl DisplayObjectImpl for MovieClip {
    impl_display_object!(base);

    fn preload(&self, context: &mut UpdateContext) {
        context
            .tag_stream
            .get_inner()
            .set_position(self.tag_stream_start.unwrap());

        use swf::Tag;
        let mut start_pos = context.tag_stream.get_ref().position();
        while let Ok(Some(tag)) = context.tag_stream.read_tag() {
            match tag {
                // Definition Tags
                Tag::DefineButton2(swf_button) => {
                    if !context.library.contains_character(swf_button.id) {
                        let button =
                            crate::button::Button::from_swf_tag(&swf_button, &context.library);
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
                Tag::DefineFont2(font) => {
                    if !context.library.contains_character(font.id) {
                        let font_object = Font::from_swf_tag(context, &font).unwrap();
                        context
                            .library
                            .register_character(font.id, Character::Font(Box::new(font_object)));
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
                        let movie_clip =
                            MovieClip::new_with_data(mc_start_pos, swf_sprite.num_frames);
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
                Tag::JpegTables(data) => context.library.set_jpeg_tables(data),
                _ => (),
            }
            start_pos = context.tag_stream.get_inner().position();
        }
    }

    fn run_frame(&mut self, context: &mut UpdateContext) {
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
        for child in self.children.values() {
            child.borrow_mut().run_frame(context);
        }

        if self.tag_stream_start.is_some() {
            context
                .tag_stream
                .get_inner()
                .set_position(context.position_stack.pop().unwrap());
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        self.run_goto_queue(context);

        //for child in self.children.values() {
        //child.borrow_mut().run_post_frame(context);
        //}
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        for child in self.children.values() {
            child.borrow_mut().render(context);
        }

        context.transform_stack.pop();
    }

    fn handle_click(&mut self, pos: (f32, f32)) {
        for child in self.children.values_mut() {
            child.borrow_mut().handle_click(pos);
        }
    }

    fn visit_children(&self, queue: &mut VecDeque<Cc<RefCell<DisplayObject>>>) {
        for child in self.children.values() {
            queue.push_back(child.clone());
        }
    }

    fn as_movie_clip(&self) -> Option<&crate::movie_clip::MovieClip> {
        Some(self)
    }

    fn as_movie_clip_mut(&mut self) -> Option<&mut crate::movie_clip::MovieClip> {
        Some(self)
    }
}

impl Trace for MovieClip {
    fn trace(&mut self, tracer: &mut Tracer) {
        for child in self.children.values_mut() {
            child.trace(tracer);
        }
    }
}
