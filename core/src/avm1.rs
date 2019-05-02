use crate::movie_clip::MovieClip;
use rand::{rngs::SmallRng, FromEntropy, Rng};
use std::io::Cursor;
use swf::avm1::read::Reader;

pub struct ActionContext<'a> {
    pub global_time: u64,
    pub active_clip: &'a mut MovieClip,
    pub audio: &'a mut crate::audio::Audio,
}

impl<'a> ActionContext<'a> {}

pub struct Avm1 {
    swf_version: u8,
    stack: Vec<Value>,
    rng: SmallRng,
}

type Error = Box<std::error::Error>;

impl Avm1 {
    pub fn new(swf_version: u8) -> Self {
        Self {
            swf_version,
            stack: vec![],
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn do_action(&mut self, context: &mut ActionContext, code: &[u8]) -> Result<(), Error> {
        let mut reader = Reader::new(Cursor::new(code), self.swf_version);

        while let Some(action) = reader.read_action()? {
            use swf::avm1::types::Action;
            match action {
                Action::Add => self.action_add(context)?,
                Action::And => self.action_and(context)?,
                Action::AsciiToChar => self.action_ascii_to_char(context)?,
                Action::Call => self.action_call(context)?,
                Action::CharToAscii => self.action_char_to_ascii(context)?,
                Action::Divide => self.action_divide(context)?,
                Action::EndDrag => self.action_end_drag(context)?,
                Action::Equals => self.action_equals(context)?,
                Action::GetTime => self.action_get_time(context)?,
                Action::GetUrl { url, target } => self.action_get_url(context, &url, &target)?,
                Action::GetUrl2 {
                    send_vars_method,
                    is_target_sprite,
                    is_load_vars,
                } => self.action_get_url_2(
                    context,
                    send_vars_method,
                    is_target_sprite,
                    is_load_vars,
                )?,
                Action::GotoFrame(frame) => self.action_goto_frame(context, frame)?,
                Action::GotoFrame2 {
                    set_playing,
                    scene_offset,
                } => self.action_goto_frame_2(context, set_playing, scene_offset)?,
                Action::GotoLabel(label) => self.action_goto_label(context, &label)?,
                Action::If { offset } => self.action_if(context, offset, &mut reader)?,
                Action::Jump { offset } => self.action_jump(context, offset, &mut reader)?,
                Action::Less => self.action_less(context)?,
                Action::MBAsciiToChar => self.action_mb_ascii_to_char(context)?,
                Action::MBCharToAscii => self.action_mb_char_to_ascii(context)?,
                Action::MBStringLength => self.action_mb_string_length(context)?,
                Action::MBStringExtract => self.action_mb_string_extract(context)?,
                Action::Multiply => self.action_multiply(context)?,
                Action::NextFrame => self.next_frame(context)?,
                Action::Not => self.action_not(context)?,
                Action::Play => self.play(context)?,
                Action::Pop => self.action_pop(context)?,
                Action::PreviousFrame => self.prev_frame(context)?,
                Action::Push(values) => self.action_push(context, &values[..])?,
                Action::RandomNumber => self.action_random_number(context)?,
                Action::RemoveSprite => self.action_remove_sprite(context)?,
                Action::SetTarget(target) => self.set_target(context, &target)?,
                Action::StartDrag => self.action_start_drag(context)?,
                Action::Stop => self.stop(context)?,
                Action::StopSounds => self.stop_sounds(context)?,
                Action::StringAdd => self.action_string_add(context)?,
                Action::StringEquals => self.action_string_equals(context)?,
                Action::StringExtract => self.action_string_extract(context)?,
                Action::StringLength => self.action_string_length(context)?,
                Action::StringLess => self.action_string_less(context)?,
                Action::Subtract => self.action_subtract(context)?,
                Action::ToggleQuality => self.toggle_quality(context)?,
                Action::ToInteger => self.action_to_integer(context)?,
                Action::Trace => self.action_trace(context)?,
                Action::WaitForFrame {
                    frame,
                    num_actions_to_skip,
                } => {
                    self.action_wait_for_frame(context, frame, num_actions_to_skip, &mut reader)?
                }
                Action::WaitForFrame2 {
                    num_actions_to_skip,
                } => self.action_wait_for_frame_2(context, num_actions_to_skip, &mut reader)?,
                _ => self.unknown_op(context)?,
            }
        }

        Ok(())
    }

    fn push(&mut self, value: impl Into<Value>) {
        self.stack.push(value.into());
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().ok_or("Stack underflow".into())
    }

    fn unknown_op(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        Err("Unknown op".into())
    }

    fn action_add(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(b.into_number_v1() + a.into_number_v1()));
        Ok(())
    }

    fn action_and(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical and
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 && a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_ascii_to_char(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.pop()?.as_f64()? as u8) as char;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_char_to_ascii(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.to_string();
        let result = s.bytes().nth(0).unwrap_or(0);
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_call(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        unimplemented!();
        // TODO(Herschel)
        Ok(())
    }

    fn action_divide(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 divide
        let a = self.pop()?;
        let b = self.pop()?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be unture for SWF v4, I get 1.#INF.

        self.push(Value::Number(b.into_number_v1() / a.into_number_v1()));
        Ok(())
    }

    fn action_end_drag(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel)
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() == a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_get_time(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        self.stack.push(Value::Number(context.global_time as f64));
        Ok(())
    }

    fn action_get_url(
        &mut self,
        context: &mut ActionContext,
        _url: &str,
        _target: &str,
    ) -> Result<(), Error> {
        // TODO(Herschel): Noop for now. Need a UI/ActionScript/network backend
        // to handle network requests appropriately for the platform.
        Ok(())
    }

    fn action_get_url_2(
        &mut self,
        context: &mut ActionContext,
        method: swf::avm1::types::SendVarsMethod,
        is_target_sprite: bool,
        is_load_vars: bool,
    ) -> Result<(), Error> {
        // TODO(Herschel): Noop for now. Need a UI/ActionScript/network backend
        // to handle network requests appropriately for the platform.
        let url = self.pop()?.to_string();

        Ok(())
    }

    fn action_goto_frame(&mut self, context: &mut ActionContext, frame: u16) -> Result<(), Error> {
        if context.active_clip.playing() {
            context.active_clip.goto_frame(frame + 1, false);
        } else {
            context.active_clip.goto_frame(frame + 1, true);
        }
        Ok(())
    }

    fn action_goto_frame_2(
        &mut self,
        context: &mut ActionContext,
        set_playing: bool,
        scene_offset: u16,
    ) -> Result<(), Error> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        // TODO(Herschel): Slash notation
        match self.pop()? {
            Value::Number(frame) => context
                .active_clip
                .goto_frame(scene_offset + (frame as u16) + 1, !set_playing),
            Value::String(frame_label) => {
                unimplemented!();
            }
            _ => return Err("Expected frame number or label".into()),
        }
        Ok(())
    }

    fn action_goto_label(&mut self, context: &mut ActionContext, label: &str) -> Result<(), Error> {
        // TODO(Herschel)
        unimplemented!("Action::GotoLabel");
        Ok(())
    }

    fn action_if(
        &mut self,
        context: &mut ActionContext,
        jump_offset: i16,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        let val = self.pop()?;
        if val.as_bool() {
            use swf::read::SwfRead;
            let pos = reader.get_inner().position();
            let new_pos = ((pos as i64) + (jump_offset as i64)) as u64;
            reader.get_inner().set_position(new_pos);
        }
        Ok(())
    }

    fn action_jump(
        &mut self,
        context: &mut ActionContext,
        jump_offset: i16,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Handle out-of-bounds.
        use swf::read::SwfRead;
        let pos = reader.get_inner().position();
        let new_pos = ((pos as i64) + (jump_offset as i64)) as u64;
        reader.get_inner().set_position(new_pos);
        Ok(())
    }

    fn action_less(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 less than
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() < a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_mb_ascii_to_char(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let val = char::try_from(self.pop()?.as_f64()? as u32)?;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_mb_char_to_ascii(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.to_string();
        let result = s.chars().nth(0).unwrap_or('\0') as u32;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_mb_string_extract(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.to_string();
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.push(Value::String(result));
        Ok(())
    }

    fn action_mb_string_length(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.to_string().len();
        self.push(Value::Number(val as f64));
        Ok(())
    }

    fn action_multiply(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 multiply
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(a.into_number_v1() * b.into_number_v1()));
        Ok(())
    }

    fn action_not(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical not
        let val = self.pop()?;
        let result = val.into_number_v1() == 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn next_frame(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.active_clip.next_frame();
        Ok(())
    }

    fn action_or(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical or
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 || a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn play(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.active_clip.play();
        Ok(())
    }

    fn prev_frame(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.active_clip.prev_frame();
        Ok(())
    }

    fn action_pop(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        self.pop();
        Ok(())
    }

    fn action_push(
        &mut self,
        context: &mut ActionContext,
        values: &[swf::avm1::types::Value],
    ) -> Result<(), Error> {
        for value in values {
            use swf::avm1::types::Value as SwfValue;
            let value = match value {
                SwfValue::Undefined => Value::Undefined,
                SwfValue::Null => Value::Null,
                SwfValue::Bool(v) => Value::Bool(*v),
                SwfValue::Int(v) => Value::Number(f64::from(*v)),
                SwfValue::Float(v) => Value::Number(f64::from(*v)),
                SwfValue::Double(v) => Value::Number(*v),
                SwfValue::Str(v) => Value::String(v.clone()),
                SwfValue::Register(u8) => unimplemented!(),
                SwfValue::ConstantPool(u16) => unimplemented!(),
            };
            self.push(value);
        }
        Ok(())
    }

    fn action_random_number(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let max = self.pop()?.as_f64()? as u32;
        let val = self.rng.gen_range(0, max);
        self.push(Value::Number(val.into()));
        Ok(())
    }

    fn action_remove_sprite(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let _target = self.pop()?.to_string();

        Ok(())
    }

    fn set_target(&mut self, _context: &mut ActionContext, target: &str) -> Result<(), Error> {
        log::info!("SetTarget: {}", target);
        // TODO(Herschel)
        Ok(())
    }

    fn action_start_drag(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel):
        Ok(())
    }

    fn stop(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.active_clip.stop();
        Ok(())
    }

    fn stop_sounds(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.audio.stop_all_sounds();
        Ok(())
    }

    fn action_string_add(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.pop()?.to_string();
        let mut b = self.pop()?.to_string();
        b.push_str(&a);
        self.push(Value::String(b));
        Ok(())
    }

    fn action_string_equals(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.to_string() == a.to_string();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_string_extract(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.to_string();
        // This is specifically a non-UTF8 aware substring.
        // SWFv4 only used ANSI strings.
        let result = s
            .bytes()
            .skip(start)
            .take(len)
            .map(|c| c as char)
            .collect::<String>();
        self.push(Value::String(result));
        Ok(())
    }

    fn action_string_length(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.to_string().bytes().len() as f64;
        self.push(Value::Number(val));
        Ok(())
    }

    fn action_string_less(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        // This is specifically a non-UTF8 aware comparison.
        let result = b.to_string().bytes().lt(a.to_string().bytes());
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_subtract(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(a.into_number_v1() + b.into_number_v1()));
        Ok(())
    }

    fn toggle_quality(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(())
    }

    fn action_to_integer(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::Number(val.into_number_v1().trunc()));
        Ok(())
    }

    fn action_trace(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        log::info!("{}", val.to_string());
        Ok(())
    }

    fn action_wait_for_frame(
        &mut self,
        _context: &mut ActionContext,
        frame: u16,
        num_actions_to_skip: u8,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            for _ in 0..num_actions_to_skip {
                reader.read_action()?;
            }
        }
        Ok(())
    }

    fn action_wait_for_frame_2(
        &mut self,
        _context: &mut ActionContext,
        num_actions_to_skip: u8,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.pop()?.as_f64()? as u16;
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            for _ in 0..num_actions_to_skip {
                reader.read_action()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
    fn into_number_v1(self) -> f64 {
        match self {
            Value::Bool(true) => 1.0,
            Value::Number(v) => v,
            Value::String(v) => v.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn from_bool_v1(value: bool, swf_version: u8) -> Value {
        // SWF version 4 did not have true bools and will push bools as 0 or 1.
        // e.g. SWF19 p. 72:
        // "If the numbers are equal, true is pushed to the stack for SWF 5 and later. For SWF 4, 1 is pushed to the stack."
        if swf_version >= 5 {
            Value::Bool(value)
        } else {
            Value::Number(if value { 1.0 } else { 0.0 })
        }
    }

    fn to_string(self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Number(v) => v.to_string(), // TODO(Herschel): Rounding for int?
            Value::String(v) => v,
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Number(v) => *v != 0.0,
            // TODO(Herschel): Value::String(v) => ??
            _ => false,
        }
    }

    fn as_f64(&self) -> Result<f64, Error> {
        match self {
            Value::Number(v) => Ok(*v),
            _ => Err("Expected Number".into()),
        }
    }
}
