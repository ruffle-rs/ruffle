use crate::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::collections::HashMap;
use std::io::Cursor;
use swf::avm1::read::Reader;

pub struct ActionContext<'a, 'gc> {
    pub global_time: u64,
    pub root: DisplayNode<'gc>,
    pub start_clip: DisplayNode<'gc>,
    pub active_clip: DisplayNode<'gc>,
    pub audio: &'a mut crate::audio::Audio,
}

pub struct Avm1 {
    swf_version: u8,
    stack: Vec<Value>,
    rng: SmallRng,
    constant_pool: Vec<String>,
    locals: HashMap<String, Value>,
}

type Error = Box<std::error::Error>;

impl Avm1 {
    pub fn new(swf_version: u8) -> Self {
        Self {
            swf_version,
            stack: vec![],
            rng: SmallRng::from_seed([0u8; 16]), // TODO(Herschel): Get a proper seed on all platforms.
            constant_pool: vec![],
            locals: HashMap::new(),
        }
    }

    pub fn do_action(&mut self, context: &mut ActionContext, code: &[u8]) -> Result<(), Error> {
        let mut reader = Reader::new(Cursor::new(code), self.swf_version);

        while let Some(action) = reader.read_action()? {
            use swf::avm1::types::Action;
            match action {
                Action::Add => self.action_add(context)?,
                Action::Add2 => self.action_add_2(context)?,
                Action::And => self.action_and(context)?,
                Action::AsciiToChar => self.action_ascii_to_char(context)?,
                Action::BitAnd => self.action_bit_and(context)?,
                Action::BitLShift => self.action_bit_lshift(context)?,
                Action::BitOr => self.action_bit_or(context)?,
                Action::BitRShift => self.action_bit_rshift(context)?,
                Action::BitURShift => self.action_bit_urshift(context)?,
                Action::BitXor => self.action_bit_xor(context)?,
                Action::Call => self.action_call(context)?,
                Action::CallFunction => self.action_call_function(context)?,
                Action::CallMethod => self.action_call_method(context)?,
                Action::CharToAscii => self.action_char_to_ascii(context)?,
                Action::ConstantPool(constant_pool) => {
                    self.action_constant_pool(context, &constant_pool[..])?
                }
                Action::Decrement => self.action_decrement(context)?,
                Action::DefineFunction {
                    name,
                    params,
                    actions,
                } => self.action_define_function(context, &name, &params[..], &actions[..])?,
                Action::DefineLocal => self.action_define_local(context)?,
                Action::DefineLocal2 => self.action_define_local_2(context)?,
                Action::Delete => self.action_delete(context)?,
                Action::Delete2 => self.action_delete_2(context)?,
                Action::Divide => self.action_divide(context)?,
                Action::EndDrag => self.action_end_drag(context)?,
                Action::Enumerate => self.action_enumerate(context)?,
                Action::Equals => self.action_equals(context)?,
                Action::Equals2 => self.action_equals_2(context)?,
                Action::GetMember => self.action_get_member(context)?,
                Action::GetProperty => self.action_get_property(context)?,
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
                Action::Increment => self.action_increment(context)?,
                Action::InitArray => self.action_init_array(context)?,
                Action::InitObject => self.action_init_object(context)?,
                Action::Jump { offset } => self.action_jump(context, offset, &mut reader)?,
                Action::Less => self.action_less(context)?,
                Action::Less2 => self.action_less_2(context)?,
                Action::MBAsciiToChar => self.action_mb_ascii_to_char(context)?,
                Action::MBCharToAscii => self.action_mb_char_to_ascii(context)?,
                Action::MBStringLength => self.action_mb_string_length(context)?,
                Action::MBStringExtract => self.action_mb_string_extract(context)?,
                Action::Modulo => self.action_modulo(context)?,
                Action::Multiply => self.action_multiply(context)?,
                Action::NextFrame => self.action_next_frame(context)?,
                Action::NewMethod => self.action_new_method(context)?,
                Action::NewObject => self.action_new_object(context)?,
                Action::Not => self.action_not(context)?,
                Action::Or => self.action_or(context)?,
                Action::Play => self.play(context)?,
                Action::Pop => self.action_pop(context)?,
                Action::PreviousFrame => self.prev_frame(context)?,
                Action::Push(values) => self.action_push(context, &values[..])?,
                Action::PushDuplicate => self.action_push_duplicate(context)?,
                Action::RandomNumber => self.action_random_number(context)?,
                Action::RemoveSprite => self.action_remove_sprite(context)?,
                Action::Return => self.action_return(context)?,
                Action::SetMember => self.action_set_member(context)?,
                Action::SetTarget(target) => self.action_set_target(context, &target)?,
                Action::StackSwap => self.action_stack_swap(context)?,
                Action::StartDrag => self.action_start_drag(context)?,
                Action::Stop => self.action_stop(context)?,
                Action::StopSounds => self.action_stop_sounds(context)?,
                Action::StoreRegister(register) => self.action_store_register(context, register)?,
                Action::StringAdd => self.action_string_add(context)?,
                Action::StringEquals => self.action_string_equals(context)?,
                Action::StringExtract => self.action_string_extract(context)?,
                Action::StringLength => self.action_string_length(context)?,
                Action::StringLess => self.action_string_less(context)?,
                Action::Subtract => self.action_subtract(context)?,
                Action::TargetPath => self.action_target_path(context)?,
                Action::ToggleQuality => self.toggle_quality(context)?,
                Action::ToInteger => self.action_to_integer(context)?,
                Action::ToNumber => self.action_to_number(context)?,
                Action::ToString => self.action_to_string(context)?,
                Action::Trace => self.action_trace(context)?,
                Action::TypeOf => self.action_type_of(context)?,
                Action::WaitForFrame {
                    frame,
                    num_actions_to_skip,
                } => {
                    self.action_wait_for_frame(context, frame, num_actions_to_skip, &mut reader)?
                }
                Action::WaitForFrame2 {
                    num_actions_to_skip,
                } => self.action_wait_for_frame_2(context, num_actions_to_skip, &mut reader)?,
                Action::With { .. } => self.action_with(context)?,
                _ => self.unknown_op(context)?,
            }
        }

        Ok(())
    }

    pub fn resolve_slash_path<'gc>(
        _start: DisplayNode<'gc>,
        _root: DisplayNode<'gc>,
        _path: &str,
    ) -> Option<DisplayNode<'gc>> {
        // let mut cur_clip = if path.bytes().nth(0).unwrap_or(0) == b'/' {
        //     path = &path[1..];
        //     root.clone()
        // } else {
        //     start.clone()
        // };
        // if !path.is_empty() {
        //     for name in path.split('/') {
        //         let next_clip = if let Some(clip) = cur_clip.borrow().as_movie_clip() {
        //             if let Some(child) = clip.get_child_by_name(name) {
        //                 child.clone()
        //             } else {
        //                 return None;
        //             }
        //         } else {
        //             return None;
        //         };
        //         cur_clip = next_clip;
        //     }
        // }
        // Some(cur_clip)
        None
    }

    fn push(&mut self, value: impl Into<Value>) {
        self.stack.push(value.into());
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack.pop().ok_or_else(|| "Stack underflow".into())
    }

    fn unknown_op(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        Err("Unknown op".into())
    }

    fn action_add(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(b.into_number_v1() + a.into_number_v1()));
        Ok(())
    }

    fn action_add_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // ECMA-262 s. 11.6.1
        let a = self.pop()?;
        let b = self.pop()?;
        // TODO(Herschel):
        if let Value::String(a) = a {
            let mut s = b.into_string();
            s.push_str(&a);
            self.push(Value::String(s));
        } else if let Value::String(mut b) = b {
            b.push_str(&a.into_string());
            self.push(Value::String(b));
        } else {
            self.push(Value::Number(b.into_number() + a.into_number()));
        }
        Ok(())
    }

    fn action_and(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical and
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 && a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_ascii_to_char(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.pop()?.as_f64()? as u8) as char;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_char_to_ascii(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.bytes().nth(0).unwrap_or(0);
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_and(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a & b;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_lshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b << a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_or(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a | b;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_rshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b >> a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_urshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_u32()?;
        let result = b >> a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_xor(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = b ^ a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_call(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _val = self.pop()?;
        unimplemented!();
        // TODO(Herschel)
    }

    fn action_call_function(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _fn_name = self.pop()?.as_string()?;
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            self.pop()?;
        }

        self.stack.push(Value::Undefined);
        // TODO(Herschel)
        unimplemented!();
    }

    fn action_call_method(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _method_name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            self.pop()?;
        }

        self.stack.push(Value::Undefined);
        // TODO(Herschel)
        unimplemented!();
    }

    fn action_constant_pool(
        &mut self,
        _context: &mut ActionContext,
        constant_pool: &[String],
    ) -> Result<(), Error> {
        self.constant_pool = constant_pool.to_vec();
        Ok(())
    }

    fn action_decrement(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.into_number();
        self.push(Value::Number(a - 1.0));
        Ok(())
    }

    fn action_define_function(
        &mut self,
        _context: &mut ActionContext,
        _name: &str,
        _params: &[String],
        _actions: &[swf::avm1::types::Action],
    ) -> Result<(), Error> {
        // TODO(Herschel)
        unimplemented!();
    }

    fn action_define_local(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let value = self.pop()?;
        let name = self.pop()?;
        self.locals.insert(name.as_string()?.clone(), value);
        Ok(())
    }

    fn action_define_local_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let name = self.pop()?;
        self.locals
            .insert(name.as_string()?.clone(), Value::Undefined);
        Ok(())
    }

    fn action_delete(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        unimplemented!();
        // TODO(Herschel)
    }

    fn action_delete_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        unimplemented!();
        // TODO(Herschel)
    }

    fn action_divide(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 divide
        let a = self.pop()?;
        let b = self.pop()?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be unture for SWF v4, I get 1.#INF.

        self.push(Value::Number(b.into_number_v1() / a.into_number_v1()));
        Ok(())
    }

    fn action_end_drag(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel)
        unimplemented!()
    }

    fn action_enumerate(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        self.push(Value::Null); // Sentinel that indicates end of enumeration
                                // TODO(Herschel): Push each property name onto the stack
        unimplemented!()
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() == a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // Version >=5 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = match (b, a) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Null, Value::Undefined) => true,
            (Value::Undefined, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(_a), Value::Object(_b)) => false, // TODO(Herschel)
            (Value::String(a), Value::Number(b)) => a.parse().unwrap_or(std::f64::NAN) == b,
            (Value::Number(a), Value::String(b)) => a == b.parse().unwrap_or(std::f64::NAN),
            _ => false,
        };
        self.push(Value::Bool(result));
        Ok(())
    }

    fn action_get_member(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        // TODO(Herschel)
        unimplemented!();
    }

    fn action_get_property(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // let prop_index = self.pop()?.as_u32()? as usize;
        // let clip_path = self.pop()?.as_string()?;
        // let ret = if let Some(clip) =
        //     Avm1::resolve_slash_path(&context.active_clip, &context.root, &clip_path)
        // {
        //     if let Some(clip) = clip.borrow().as_movie_clip() {
        //         match prop_index {
        //             0 => Value::Number(f64::from(clip.x())),
        //             1 => Value::Number(f64::from(clip.y())),
        //             2 => Value::Number(f64::from(clip.x_scale())),
        //             3 => Value::Number(f64::from(clip.y_scale())),
        //             4 => Value::Number(f64::from(clip.current_frame())),
        //             5 => Value::Number(f64::from(clip.total_frames())),
        //             12 => Value::Number(f64::from(clip.frames_loaded())),
        //             _ => unimplemented!(),
        //         }
        //     } else {
        //         Value::Undefined
        //     }
        // } else {
        //     Value::Undefined
        // };
        // self.push(ret);
        Ok(())
    }

    fn action_get_time(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        self.stack.push(Value::Number(context.global_time as f64));
        Ok(())
    }

    fn action_get_url(
        &mut self,
        _context: &mut ActionContext,
        _url: &str,
        _target: &str,
    ) -> Result<(), Error> {
        // TODO(Herschel): Noop for now. Need a UI/ActionScript/network backend
        // to handle network requests appropriately for the platform.
        Ok(())
    }

    fn action_get_url_2(
        &mut self,
        _context: &mut ActionContext,
        _method: swf::avm1::types::SendVarsMethod,
        _is_target_sprite: bool,
        _is_load_vars: bool,
    ) -> Result<(), Error> {
        // TODO(Herschel): Noop for now. Need a UI/ActionScript/network backend
        // to handle network requests appropriately for the platform.
        let _url = self.pop()?.into_string();

        Ok(())
    }

    fn action_goto_frame(
        &mut self,
        _context: &mut ActionContext,
        _frame: u16,
    ) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // if clip.playing() {
        //     clip.goto_frame(frame + 1, false);
        // } else {
        //     clip.goto_frame(frame + 1, true);
        // }
        Ok(())
    }

    fn action_goto_frame_2(
        &mut self,
        _context: &mut ActionContext,
        _set_playing: bool,
        _scene_offset: u16,
    ) -> Result<(), Error> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        // TODO(Herschel): Slash notation
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // match self.pop()? {
        //     Value::Number(frame) => {
        //         clip.goto_frame(scene_offset + (frame as u16) + 1, !set_playing)
        //     }
        //     Value::String(frame_label) => {
        //         //if let Some(frame) = clip.frame_label_to_number(&frame_label, context) {
        //         //  clip.goto_frame(scene_offset + frame, !set_playing)
        //         //}
        //     }
        //     _ => return Err("Expected frame number or label".into()),
        // }
        Ok(())
    }

    fn action_goto_label(
        &mut self,
        _context: &mut ActionContext,
        _label: &str,
    ) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        //if let Some(frame) = clip.frame_label_to_number(label, context) {
        //clip.goto_frame(frame !set_playing)
        //}
        Ok(())
    }

    fn action_if(
        &mut self,
        _context: &mut ActionContext,
        jump_offset: i16,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        let val = self.pop()?;
        if val.as_bool() {
            use swf::read::SwfRead;
            let pos = reader.get_inner().position();
            let new_pos = ((pos as i64) + i64::from(jump_offset)) as u64;
            reader.get_inner().set_position(new_pos);
        }
        Ok(())
    }

    fn action_increment(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.into_number();
        self.push(Value::Number(a + 1.0));
        Ok(())
    }

    fn action_init_array(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let num_elements = self.pop()?.as_i64()?;
        for _ in 0..num_elements {
            let _value = self.pop()?;
        }

        // TODO(Herschel)
        unimplemented!("Action::InitArray");
    }

    fn action_init_object(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let num_props = self.pop()?.as_i64()?;
        for _ in 0..num_props {
            let _value = self.pop()?;
            let _name = self.pop()?;
        }

        // TODO(Herschel)
        unimplemented!("Action::InitArray");
    }

    fn action_jump(
        &mut self,
        _context: &mut ActionContext,
        jump_offset: i16,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Handle out-of-bounds.
        use swf::read::SwfRead;
        let pos = reader.get_inner().position();
        let new_pos = ((pos as i64) + i64::from(jump_offset)) as u64;
        reader.get_inner().set_position(new_pos);
        Ok(())
    }

    fn action_less(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 less than
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() < a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_less_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // ECMA-262 s. 11.8.5
        let a = self.pop()?;
        let b = self.pop()?;

        let result = match (a, b) {
            (Value::String(a), Value::String(b)) => b.to_string().bytes().lt(a.to_string().bytes()),
            (a, b) => b.into_number() < a.into_number(),
        };

        self.push(Value::Bool(result));
        Ok(())
    }

    fn action_mb_ascii_to_char(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let val = char::try_from(self.pop()?.as_f64()? as u32)?;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_mb_char_to_ascii(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.chars().nth(0).unwrap_or('\0') as u32;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_mb_string_extract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.into_string();
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.push(Value::String(result));
        Ok(())
    }

    fn action_mb_string_length(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().len();
        self.push(Value::Number(val as f64));
        Ok(())
    }

    fn action_multiply(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 multiply
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(a.into_number_v1() * b.into_number_v1()));
        Ok(())
    }

    fn action_modulo(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO: Wrong operands?
        let a = self.pop()?.as_f64()?;
        let b = self.pop()?.as_f64()?;
        self.push(Value::Number(a % b));
        Ok(())
    }

    fn action_not(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical not
        let val = self.pop()?;
        let result = val.into_number_v1() == 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_next_frame(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // clip.next_frame();
        Ok(())
    }

    fn action_new_method(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        let _num_args = self.pop()?.as_i64()?;
        self.push(Value::Undefined);
        // TODO(Herschel)
        unimplemented!("Action::NewMethod");
    }

    fn action_new_object(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _object = self.pop()?.as_string()?;
        let num_args = self.pop()?.as_i64()?;
        for _ in 0..num_args {
            let _arg = self.pop()?;
        }
        self.push(Value::Undefined);
        // TODO(Herschel)
        unimplemented!("Action::NewObject");
    }

    fn action_or(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical or
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 || a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn play(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // clip.play();
        Ok(())
    }

    fn prev_frame(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // clip.prev_frame();
        Ok(())
    }

    fn action_pop(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        self.pop()?;
        Ok(())
    }

    fn action_push(
        &mut self,
        _context: &mut ActionContext,
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
                SwfValue::Register(_v) => unimplemented!(),
                SwfValue::ConstantPool(_v) => unimplemented!(),
            };
            self.push(value);
        }
        Ok(())
    }

    fn action_push_duplicate(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.stack.last().ok_or("Stack underflow")?.clone();
        self.push(val);
        Ok(())
    }

    fn action_random_number(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let max = self.pop()?.as_f64()? as u32;
        let val = self.rng.gen_range(0, max);
        self.push(Value::Number(val.into()));
        Ok(())
    }

    fn action_remove_sprite(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _target = self.pop()?.into_string();
        // TODO(Herschel)
        unimplemented!("Action::RemoveSprite");
    }

    fn action_return(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _result = self.pop()?;
        // TODO(Herschel)
        unimplemented!("Action::Return");
    }

    fn action_set_member(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _value = self.pop()?;
        let _name = self.pop()?;
        let _object = self.pop()?;
        // TODO(Herschel)
        unimplemented!("Action::SetMember");
    }

    fn action_set_target(
        &mut self,
        _context: &mut ActionContext,
        target: &str,
    ) -> Result<(), Error> {
        log::info!("SetTarget: {}", target);
        // if target.is_empty() {
        //     context.active_clip = context.start_clip.clone();
        // } else {
        //     if let Some(clip) = Avm1::resolve_slash_path(&context.start_clip, &context.root, target)
        //     {
        //         context.active_clip = clip;
        //     }
        // }
        Ok(())
    }

    fn action_stack_swap(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn action_start_drag(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel):
        Ok(())
    }

    fn action_stop(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // let mut display_object = context.active_clip.borrow_mut();
        // let mut clip = display_object.as_movie_clip_mut().unwrap();
        // clip.stop();
        Ok(())
    }

    fn action_stop_sounds(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.audio.stop_all_sounds();
        Ok(())
    }

    fn action_store_register(
        &mut self,
        _context: &mut ActionContext,
        _register: u8,
    ) -> Result<(), Error> {
        // Does NOT pop the value from the stack.
        let _val = self.stack.last().ok_or("Stack underflow")?;
        unimplemented!("Action::StoreRegister");
    }

    fn action_string_add(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.pop()?.into_string();
        let mut b = self.pop()?.into_string();
        b.push_str(&a);
        self.push(Value::String(b));
        Ok(())
    }

    fn action_string_equals(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_string() == a.into_string();
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_string_extract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.into_string();
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

    fn action_string_length(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().bytes().len() as f64;
        self.push(Value::Number(val));
        Ok(())
    }

    fn action_string_less(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        // This is specifically a non-UTF8 aware comparison.
        let result = b.into_string().bytes().lt(a.into_string().bytes());
        self.push(Value::from_bool_v1(result, self.swf_version));
        Ok(())
    }

    fn action_subtract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(a.into_number_v1() + b.into_number_v1()));
        Ok(())
    }

    fn action_target_path(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel)
        let _clip = self.pop()?.as_object()?;
        self.push(Value::Undefined);
        unimplemented!("Action::TargetPath");
    }

    fn toggle_quality(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(())
    }

    fn action_to_integer(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::Number(val.into_number_v1().trunc()));
        Ok(())
    }

    fn action_to_number(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::Number(val.into_number()));
        Ok(())
    }

    fn action_to_string(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::String(val.into_string()));
        Ok(())
    }

    fn action_trace(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        log::info!("{}", val.into_string());
        Ok(())
    }

    fn action_type_of(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _type_str = match self.pop()? {
            Value::Undefined => "undefined",
            Value::Null => "null",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::String(_) => "string",
            Value::Object(_) => "object",
        };
        // TODO(Herschel): function, movieclip
        Ok(())
    }

    fn action_wait_for_frame(
        &mut self,
        _context: &mut ActionContext,
        _frame: u16,
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

    fn action_with(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _object = self.pop()?.as_object()?;
        unimplemented!("Action::With");
    }
}

type ObjectPtr = std::marker::PhantomData<()>;

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(ObjectPtr),
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

    fn into_number(self) -> f64 {
        // ECMA-262 2nd edtion s. 9.3 ToNumber
        use std::f64::NAN;
        match self {
            Value::Undefined => NAN,
            Value::Null => NAN,
            Value::Bool(false) => 0.0,
            Value::Bool(true) => 1.0,
            Value::Number(v) => v,
            Value::String(v) => v.parse().unwrap_or(NAN), // TODO(Herschel): Handle Infinity/etc.?
            Value::Object(_object) => unimplemented!(),   // TODO(Herschel)
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

    fn into_string(self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Number(v) => v.to_string(), // TODO(Herschel): Rounding for int?
            Value::String(v) => v,
            Value::Object(_) => "[Object object]".to_string(), // TODO(Herschel):
        }
    }

    fn as_bool(&self) -> bool {
        match *self {
            Value::Bool(v) => v,
            Value::Number(v) => v != 0.0,
            // TODO(Herschel): Value::String(v) => ??
            _ => false,
        }
    }

    fn as_i32(&self) -> Result<i32, Error> {
        self.as_f64().map(|n| n as i32)
    }

    fn as_u32(&self) -> Result<u32, Error> {
        self.as_f64().map(|n| n as u32)
    }

    fn as_i64(&self) -> Result<i64, Error> {
        self.as_f64().map(|n| n as i64)
    }

    fn as_f64(&self) -> Result<f64, Error> {
        match *self {
            Value::Number(v) => Ok(v),
            _ => Err("Expected Number".into()),
        }
    }

    fn as_string(&self) -> Result<&String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err("Expected Number".into()),
        }
    }

    fn as_object(&self) -> Result<&ObjectPtr, Error> {
        if let Value::Object(object) = self {
            Ok(object)
        } else {
            Err("Expected Object".into())
        }
    }
}
