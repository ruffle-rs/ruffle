use crate::avm2::opcode::OpCode;
use crate::avm2::types::*;
use crate::string::SwfStr;
use crate::write::SwfWriteExt;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Result, Write};

pub struct Writer<W: Write> {
    output: W,
}

impl<W: Write> SwfWriteExt for Writer<W> {
    #[inline]
    fn write_u8(&mut self, n: u8) -> io::Result<()> {
        self.output.write_u8(n)
    }

    #[inline]
    fn write_u16(&mut self, n: u16) -> io::Result<()> {
        self.output.write_u16::<LittleEndian>(n)
    }

    #[inline]
    fn write_u32(&mut self, n: u32) -> io::Result<()> {
        self.output.write_u32::<LittleEndian>(n)
    }

    #[inline]
    fn write_u64(&mut self, n: u64) -> io::Result<()> {
        self.output.write_u64::<LittleEndian>(n)
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> io::Result<()> {
        self.output.write_i8(n)
    }

    #[inline]
    fn write_i16(&mut self, n: i16) -> io::Result<()> {
        self.output.write_i16::<LittleEndian>(n)
    }

    #[inline]
    fn write_i32(&mut self, n: i32) -> io::Result<()> {
        self.output.write_i32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f32(&mut self, n: f32) -> io::Result<()> {
        self.output.write_f32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f64(&mut self, n: f64) -> io::Result<()> {
        self.output.write_f64::<LittleEndian>(n)
    }

    #[inline]
    fn write_string(&mut self, s: &'_ SwfStr) -> io::Result<()> {
        self.output.write_all(s.as_bytes())?;
        self.write_u8(0)
    }
}

impl<W: Write> Writer<W> {
    pub fn new(output: W) -> Self {
        Self { output }
    }

    pub fn write(&mut self, abc_file: AbcFile) -> Result<()> {
        self.write_u16(abc_file.minor_version)?;
        self.write_u16(abc_file.major_version)?;
        self.write_constant_pool(&abc_file.constant_pool)?;

        self.write_u30(abc_file.methods.len() as u32)?;
        for method in &abc_file.methods {
            self.write_method(method)?;
        }

        self.write_u30(abc_file.metadata.len() as u32)?;
        for metadata in &abc_file.metadata {
            self.write_metadata(metadata)?;
        }

        self.write_u30(abc_file.instances.len() as u32)?;
        for instance in &abc_file.instances {
            self.write_instance(instance)?;
        }

        for class in &abc_file.classes {
            self.write_class(class)?;
        }

        self.write_u30(abc_file.scripts.len() as u32)?;
        for script in &abc_file.scripts {
            self.write_script(script)?;
        }

        self.write_u30(abc_file.method_bodies.len() as u32)?;
        for method_body in &abc_file.method_bodies {
            self.write_method_body(method_body)?;
        }

        Ok(())
    }

    fn write_u30(&mut self, n: u32) -> Result<()> {
        // TODO: Verify n fits in 30 bits.
        self.write_u32(n)
    }

    #[allow(clippy::unusual_byte_groupings)]
    fn write_u32(&mut self, mut n: u32) -> Result<()> {
        loop {
            let byte = (n as u8) & 0x7f;
            n >>= 7;
            if n != 0 {
                self.write_u8(0b1_0000000 | byte)?;
            } else {
                self.write_u8(byte)?;
                break;
            }
        }

        Ok(())
    }

    fn write_i24(&mut self, n: i32) -> Result<()> {
        let bytes = n.to_le_bytes();
        debug_assert!(bytes[3] == 0 || bytes[3] == 0xFF);
        self.output.write_all(&bytes[..3])
    }

    fn write_i32(&mut self, n: i32) -> Result<()> {
        self.write_u32(n as u32)
    }

    fn write_index<T>(&mut self, i: &Index<T>) -> Result<()> {
        self.write_u30(i.0)
    }

    fn write_string(&mut self, s: &str) -> Result<()> {
        self.write_u30(s.len() as u32)?;
        self.output.write_all(s.as_bytes())?;
        Ok(())
    }

    fn write_constant_pool(&mut self, constant_pool: &ConstantPool) -> Result<()> {
        if !constant_pool.ints.is_empty() {
            self.write_u30(constant_pool.ints.len() as u32 + 1)?;
            for n in &constant_pool.ints {
                self.write_i32(*n)?;
            }
        } else {
            self.write_u32(0)?;
        }

        if !constant_pool.uints.is_empty() {
            self.write_u30(constant_pool.uints.len() as u32 + 1)?;
            for n in &constant_pool.uints {
                self.write_u32(*n)?;
            }
        } else {
            self.write_u30(0)?;
        }

        if !constant_pool.doubles.is_empty() {
            self.write_u30(constant_pool.doubles.len() as u32 + 1)?;
            for n in &constant_pool.doubles {
                self.write_f64(*n)?;
            }
        } else {
            self.write_u32(0)?;
        }

        if !constant_pool.strings.is_empty() {
            self.write_u30(constant_pool.strings.len() as u32 + 1)?;
            for s in &constant_pool.strings {
                self.write_string(s)?;
            }
        } else {
            self.write_u32(0)?;
        }

        if !constant_pool.namespaces.is_empty() {
            self.write_u30(constant_pool.namespaces.len() as u32 + 1)?;
            for namespace in &constant_pool.namespaces {
                self.write_namespace(namespace)?;
            }
        } else {
            self.write_u32(0)?;
        }

        if !constant_pool.namespace_sets.is_empty() {
            self.write_u30(constant_pool.namespace_sets.len() as u32 + 1)?;
            for namespace_set in &constant_pool.namespace_sets {
                self.write_namespace_set(namespace_set)?;
            }
        } else {
            self.write_u32(0)?;
        }

        if !constant_pool.multinames.is_empty() {
            self.write_u30(constant_pool.multinames.len() as u32 + 1)?;
            for multiname in &constant_pool.multinames {
                self.write_multiname(multiname)?;
            }
        } else {
            self.write_u32(0)?;
        }

        Ok(())
    }

    fn write_namespace(&mut self, namespace: &Namespace) -> Result<()> {
        match *namespace {
            Namespace::Namespace(ref name) => {
                self.write_u8(0x08)?;
                self.write_index(name)?;
            }
            Namespace::Package(ref name) => {
                self.write_u8(0x16)?;
                self.write_index(name)?;
            }
            Namespace::PackageInternal(ref name) => {
                self.write_u8(0x17)?;
                self.write_index(name)?;
            }
            Namespace::Protected(ref name) => {
                self.write_u8(0x18)?;
                self.write_index(name)?;
            }
            Namespace::Explicit(ref name) => {
                self.write_u8(0x19)?;
                self.write_index(name)?;
            }
            Namespace::StaticProtected(ref name) => {
                self.write_u8(0x1a)?;
                self.write_index(name)?;
            }
            Namespace::Private(ref name) => {
                self.write_u8(0x05)?;
                self.write_index(name)?;
            }
        }
        Ok(())
    }

    fn write_namespace_set(&mut self, namespace_set: &[Index<Namespace>]) -> Result<()> {
        self.write_u30(namespace_set.len() as u32)?;
        for i in namespace_set {
            self.write_index(i)?;
        }
        Ok(())
    }

    fn write_multiname(&mut self, multiname: &Multiname) -> Result<()> {
        match *multiname {
            Multiname::QName {
                ref namespace,
                ref name,
            } => {
                self.write_u8(0x07)?;
                self.write_index(namespace)?;
                self.write_index(name)?;
            }
            Multiname::QNameA {
                ref namespace,
                ref name,
            } => {
                self.write_u8(0x0d)?;
                self.write_index(namespace)?;
                self.write_index(name)?;
            }
            Multiname::RTQName { ref name } => {
                self.write_u8(0x0f)?;
                self.write_index(name)?;
            }
            Multiname::RTQNameA { ref name } => {
                self.write_u8(0x10)?;
                self.write_index(name)?;
            }
            Multiname::RTQNameL => {
                self.write_u8(0x11)?;
            }
            Multiname::RTQNameLA => {
                self.write_u8(0x12)?;
            }
            Multiname::Multiname {
                ref namespace_set,
                ref name,
            } => {
                self.write_u8(0x09)?;
                self.write_index(name)?;
                self.write_index(namespace_set)?;
            }
            Multiname::MultinameA {
                ref namespace_set,
                ref name,
            } => {
                self.write_u8(0x0e)?;
                self.write_index(name)?;
                self.write_index(namespace_set)?;
            }
            Multiname::MultinameL { ref namespace_set } => {
                self.write_u8(0x1b)?;
                self.write_index(namespace_set)?;
            }
            Multiname::MultinameLA { ref namespace_set } => {
                self.write_u8(0x1c)?;
                self.write_index(namespace_set)?;
            }
            Multiname::TypeName {
                ref base_type,
                ref parameters,
            } => {
                self.write_u8(0x1d)?;
                self.write_index(base_type)?;
                self.write_u30(parameters.len() as u32)?;

                for param in parameters {
                    self.write_index(param)?;
                }
            }
        }
        Ok(())
    }

    fn write_method(&mut self, method: &Method) -> Result<()> {
        self.write_u8(method.params.len() as u8)?;
        self.write_index(&method.return_type)?;
        let mut num_optional_params = 0;
        let mut has_param_names = false;
        for param in &method.params {
            self.write_index(&param.kind)?;
            if param.default_value.is_some() {
                num_optional_params += 1;
            }
            if param.name.is_some() {
                has_param_names = true;
            }
        }
        self.write_index(&method.name)?;
        self.write_u8(method.flags.bits())?;

        if num_optional_params > 0 {
            self.write_u30(num_optional_params)?;
            let num_required = method.params.len() - num_optional_params as usize;
            for param in method.params.iter().skip(num_required) {
                if let Some(ref value) = param.default_value {
                    self.write_constant_value(value)?;
                }
            }
        }

        if has_param_names {
            for param in &method.params {
                if let Some(ref name) = param.name {
                    self.write_index(name)?;
                }
            }
        }

        Ok(())
    }

    fn write_constant_value(&mut self, value: &DefaultValue) -> Result<()> {
        let (index, kind) = match *value {
            DefaultValue::Undefined => (0, 0x00),
            DefaultValue::String(ref i) => (i.as_u30(), 0x01),
            DefaultValue::Int(ref i) => (i.as_u30(), 0x03),
            DefaultValue::Uint(ref i) => (i.as_u30(), 0x04),
            DefaultValue::Private(ref i) => (i.as_u30(), 0x05),
            DefaultValue::Double(ref i) => (i.as_u30(), 0x06),
            DefaultValue::Namespace(ref i) => (i.as_u30(), 0x08),
            DefaultValue::False => (0x0a, 0x0a),
            DefaultValue::True => (0x0b, 0x0b),
            DefaultValue::Null => (0x0c, 0x0c),
            DefaultValue::Package(ref i) => (i.as_u30(), 0x16),
            DefaultValue::PackageInternal(ref i) => (i.as_u30(), 0x17),
            DefaultValue::Protected(ref i) => (i.as_u30(), 0x18),
            DefaultValue::Explicit(ref i) => (i.as_u30(), 0x19),
            DefaultValue::StaticProtected(ref i) => (i.as_u30(), 0x1a),
        };
        self.write_u30(index)?;
        self.write_u8(kind)?;
        Ok(())
    }

    fn write_optional_value(&mut self, value: &Option<DefaultValue>) -> Result<()> {
        match *value {
            None => self.write_u30(0)?,
            Some(ref value) => {
                let (index, kind) = match *value {
                    // Just write out a non-zero 'index' field - it's unused,
                    // so it doesn't matter what it is.
                    DefaultValue::Undefined => (0x01, 0x00),
                    DefaultValue::String(ref i) => (i.as_u30(), 0x01),
                    DefaultValue::Int(ref i) => (i.as_u30(), 0x03),
                    DefaultValue::Uint(ref i) => (i.as_u30(), 0x04),
                    DefaultValue::Private(ref i) => (i.as_u30(), 0x05),
                    DefaultValue::Double(ref i) => (i.as_u30(), 0x06),
                    DefaultValue::Namespace(ref i) => (i.as_u30(), 0x08),
                    DefaultValue::False => (0x0a, 0x0a),
                    DefaultValue::True => (0x0b, 0x0b),
                    DefaultValue::Null => (0x0c, 0x0c),
                    DefaultValue::Package(ref i) => (i.as_u30(), 0x16),
                    DefaultValue::PackageInternal(ref i) => (i.as_u30(), 0x17),
                    DefaultValue::Protected(ref i) => (i.as_u30(), 0x18),
                    DefaultValue::Explicit(ref i) => (i.as_u30(), 0x19),
                    DefaultValue::StaticProtected(ref i) => (i.as_u30(), 0x1a),
                };
                self.write_u30(index)?;
                self.write_u8(kind)?;
            }
        }
        Ok(())
    }

    fn write_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        self.write_index(&metadata.name)?;
        self.write_u30(metadata.items.len() as u32)?;
        for item in &metadata.items {
            self.write_index(&item.key)?;
            self.write_index(&item.value)?;
        }
        Ok(())
    }

    fn write_instance(&mut self, instance: &Instance) -> Result<()> {
        self.write_index(&instance.name)?;
        self.write_index(&instance.super_name)?;
        self.write_u8(
            if instance.protected_namespace.is_some() {
                0x08
            } else {
                0
            } | if instance.is_interface { 0x04 } else { 0 }
                | if instance.is_final { 0x02 } else { 0 }
                | if instance.is_sealed { 0x01 } else { 0 },
        )?;

        if let Some(ref namespace) = instance.protected_namespace {
            self.write_index(namespace)?;
        }

        self.write_u30(instance.interfaces.len() as u32)?;
        for interface in &instance.interfaces {
            self.write_index(interface)?;
        }

        self.write_index(&instance.init_method)?;

        self.write_u30(instance.traits.len() as u32)?;
        for t in &instance.traits {
            self.write_trait(t)?;
        }

        Ok(())
    }

    fn write_class(&mut self, class: &Class) -> Result<()> {
        self.write_index(&class.init_method)?;
        self.write_u30(class.traits.len() as u32)?;
        for t in &class.traits {
            self.write_trait(t)?;
        }
        Ok(())
    }

    fn write_script(&mut self, script: &Script) -> Result<()> {
        self.write_index(&script.init_method)?;
        self.write_u30(script.traits.len() as u32)?;
        for t in &script.traits {
            self.write_trait(t)?;
        }
        Ok(())
    }

    #[allow(clippy::unusual_byte_groupings)]
    fn write_trait(&mut self, t: &Trait) -> Result<()> {
        self.write_index(&t.name)?;
        let flags = if !t.metadata.is_empty() {
            0b0100_0000
        } else {
            0
        } | if t.is_override { 0b0010_0000 } else { 0 }
            | if t.is_final { 0b0001_0000 } else { 0 };

        match t.kind {
            TraitKind::Slot {
                slot_id,
                ref type_name,
                ref value,
            } => {
                self.write_u8(flags)?;
                self.write_u30(slot_id)?;
                self.write_index(type_name)?;
                self.write_optional_value(value)?;
            }
            TraitKind::Method {
                disp_id,
                ref method,
            } => {
                self.write_u8(flags | 1)?;
                self.write_u30(disp_id)?;
                self.write_index(method)?;
            }
            TraitKind::Getter {
                disp_id,
                ref method,
            } => {
                self.write_u8(flags | 2)?;
                self.write_u30(disp_id)?;
                self.write_index(method)?;
            }
            TraitKind::Setter {
                disp_id,
                ref method,
            } => {
                self.write_u8(flags | 3)?;
                self.write_u30(disp_id)?;
                self.write_index(method)?;
            }
            TraitKind::Class { slot_id, ref class } => {
                self.write_u8(flags | 4)?;
                self.write_u30(slot_id)?;
                self.write_index(class)?;
            }
            TraitKind::Function {
                slot_id,
                ref function,
            } => {
                self.write_u8(flags | 5)?;
                self.write_u30(slot_id)?;
                self.write_index(function)?;
            }
            TraitKind::Const {
                slot_id,
                ref type_name,
                ref value,
            } => {
                self.write_u8(flags | 6)?;
                self.write_u30(slot_id)?;
                self.write_index(type_name)?;
                self.write_optional_value(value)?;
            }
        }

        if !t.metadata.is_empty() {
            self.write_u30(t.metadata.len() as u32)?;
            for metadata in &t.metadata {
                self.write_index(metadata)?;
            }
        }

        Ok(())
    }

    fn write_method_body(&mut self, method_body: &MethodBody) -> Result<()> {
        self.write_index(&method_body.method)?;
        self.write_u30(method_body.max_stack)?;
        self.write_u30(method_body.num_locals)?;
        self.write_u30(method_body.init_scope_depth)?;
        self.write_u30(method_body.max_scope_depth)?;

        self.write_u30(method_body.code.len() as u32)?;
        self.output.write_all(&method_body.code)?;

        self.write_u30(method_body.exceptions.len() as u32)?;
        for exception in &method_body.exceptions {
            self.write_exception(exception)?;
        }

        self.write_u30(method_body.traits.len() as u32)?;
        for t in &method_body.traits {
            self.write_trait(t)?;
        }

        Ok(())
    }

    fn write_exception(&mut self, exception: &Exception) -> Result<()> {
        self.write_u30(exception.from_offset)?;
        self.write_u30(exception.to_offset)?;
        self.write_u30(exception.target_offset)?;
        self.write_index(&exception.type_name)?;
        self.write_index(&exception.variable_name)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn write_op(&mut self, op: &Op) -> Result<()> {
        match *op {
            Op::Add => self.write_opcode(OpCode::Add)?,
            Op::AddI => self.write_opcode(OpCode::AddI)?,
            Op::ApplyType { num_types } => {
                self.write_opcode(OpCode::ApplyType)?;
                self.write_u30(num_types)?;
            }
            Op::AsType { ref type_name } => {
                self.write_opcode(OpCode::AsType)?;
                self.write_index(type_name)?;
            }
            Op::AsTypeLate => self.write_opcode(OpCode::AsTypeLate)?,
            Op::BitAnd => self.write_opcode(OpCode::BitAnd)?,
            Op::BitNot => self.write_opcode(OpCode::BitNot)?,
            Op::BitOr => self.write_opcode(OpCode::BitOr)?,
            Op::BitXor => self.write_opcode(OpCode::BitXor)?,
            Op::Bkpt => self.write_opcode(OpCode::Bkpt)?,
            Op::BkptLine { line_num } => {
                self.write_opcode(OpCode::BkptLine)?;
                self.write_u30(line_num)?;
            }
            Op::Call { num_args } => {
                self.write_opcode(OpCode::Call)?;
                self.write_u30(num_args)?;
            }
            Op::CallMethod {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallMethod)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallProperty {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallProperty)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallPropLex {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallPropLex)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallPropVoid {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallPropVoid)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallStatic {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallStatic)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallSuper {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallSuper)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CallSuperVoid {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::CallSuperVoid)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::CheckFilter => self.write_opcode(OpCode::CheckFilter)?,
            Op::Coerce { ref index } => {
                self.write_opcode(OpCode::Coerce)?;
                self.write_index(index)?;
            }
            Op::CoerceA => self.write_opcode(OpCode::CoerceA)?,
            Op::CoerceB => self.write_opcode(OpCode::CoerceB)?,
            Op::CoerceD => self.write_opcode(OpCode::CoerceD)?,
            Op::CoerceI => self.write_opcode(OpCode::CoerceI)?,
            Op::CoerceO => self.write_opcode(OpCode::CoerceO)?,
            Op::CoerceS => self.write_opcode(OpCode::CoerceS)?,
            Op::CoerceU => self.write_opcode(OpCode::CoerceU)?,
            Op::Construct { num_args } => {
                self.write_opcode(OpCode::Construct)?;
                self.write_u30(num_args)?;
            }
            Op::ConstructProp {
                ref index,
                num_args,
            } => {
                self.write_opcode(OpCode::ConstructProp)?;
                self.write_index(index)?;
                self.write_u30(num_args)?;
            }
            Op::ConstructSuper { num_args } => {
                self.write_opcode(OpCode::ConstructSuper)?;
                self.write_u30(num_args)?;
            }
            Op::ConvertB => self.write_opcode(OpCode::ConvertB)?,
            Op::ConvertD => self.write_opcode(OpCode::ConvertD)?,
            Op::ConvertI => self.write_opcode(OpCode::ConvertI)?,
            Op::ConvertO => self.write_opcode(OpCode::ConvertO)?,
            Op::ConvertS => self.write_opcode(OpCode::ConvertS)?,
            Op::ConvertU => self.write_opcode(OpCode::ConvertU)?,
            Op::Debug {
                is_local_register,
                ref register_name,
                register,
            } => {
                self.write_opcode(OpCode::Debug)?;
                self.write_u8(is_local_register as u8)?;
                self.write_index(register_name)?;
                self.write_u8(register)?;
                self.write_u30(0)?; // Unused
            }
            Op::DebugFile { ref file_name } => {
                self.write_opcode(OpCode::DebugFile)?;
                self.write_index(file_name)?;
            }
            Op::DebugLine { line_num } => {
                self.write_opcode(OpCode::DebugLine)?;
                self.write_u30(line_num)?;
            }
            Op::DecLocal { index } => {
                self.write_opcode(OpCode::DecLocal)?;
                self.write_u30(index)?;
            }
            Op::DecLocalI { index } => {
                self.write_opcode(OpCode::DecLocalI)?;
                self.write_u30(index)?;
            }
            Op::Decrement => self.write_opcode(OpCode::Decrement)?,
            Op::DecrementI => self.write_opcode(OpCode::DecrementI)?,
            Op::DeleteProperty { ref index } => {
                self.write_opcode(OpCode::DeleteProperty)?;
                self.write_index(index)?;
            }
            Op::Divide => self.write_opcode(OpCode::Divide)?,
            Op::Dup => self.write_opcode(OpCode::Dup)?,
            Op::Dxns { ref index } => {
                self.write_opcode(OpCode::Dxns)?;
                self.write_index(index)?;
            }
            Op::DxnsLate => self.write_opcode(OpCode::DxnsLate)?,
            Op::Equals => self.write_opcode(OpCode::Equals)?,
            Op::EscXAttr => self.write_opcode(OpCode::EscXAttr)?,
            Op::EscXElem => self.write_opcode(OpCode::EscXElem)?,
            Op::FindDef { ref index } => {
                self.write_opcode(OpCode::FindDef)?;
                self.write_index(index)?;
            }
            Op::FindProperty { ref index } => {
                self.write_opcode(OpCode::FindProperty)?;
                self.write_index(index)?;
            }
            Op::FindPropStrict { ref index } => {
                self.write_opcode(OpCode::FindPropStrict)?;
                self.write_index(index)?;
            }
            Op::GetDescendants { ref index } => {
                self.write_opcode(OpCode::GetDescendants)?;
                self.write_index(index)?;
            }
            Op::GetGlobalScope => self.write_opcode(OpCode::GetGlobalScope)?,
            Op::GetGlobalSlot { index } => {
                self.write_opcode(OpCode::GetGlobalSlot)?;
                self.write_u30(index)?;
            }
            Op::GetLex { ref index } => {
                self.write_opcode(OpCode::GetLex)?;
                self.write_index(index)?;
            }
            Op::GetLocal { index } => match index {
                0 => self.write_opcode(OpCode::GetLocal0)?,
                1 => self.write_opcode(OpCode::GetLocal1)?,
                2 => self.write_opcode(OpCode::GetLocal2)?,
                3 => self.write_opcode(OpCode::GetLocal3)?,
                _ => {
                    self.write_opcode(OpCode::GetLocal)?;
                    self.write_u30(index)?;
                }
            },
            Op::GetOuterScope { index } => {
                self.write_opcode(OpCode::GetOuterScope)?;
                self.write_u30(index)?;
            }
            Op::GetProperty { ref index } => {
                self.write_opcode(OpCode::GetProperty)?;
                self.write_index(index)?;
            }
            Op::GetScopeObject { index } => {
                self.write_opcode(OpCode::GetScopeObject)?;
                self.write_u8(index)?;
            }
            Op::GetSlot { index } => {
                self.write_opcode(OpCode::GetSlot)?;
                self.write_u30(index)?;
            }
            Op::GetSuper { ref index } => {
                self.write_opcode(OpCode::GetSuper)?;
                self.write_index(index)?;
            }
            Op::GreaterEquals => self.write_opcode(OpCode::GreaterEquals)?,
            Op::GreaterThan => self.write_opcode(OpCode::GreaterThan)?,
            Op::HasNext => self.write_opcode(OpCode::HasNext)?,
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                self.write_opcode(OpCode::HasNext2)?;
                self.write_u30(object_register)?;
                self.write_u30(index_register)?;
            }
            Op::IfEq { offset } => {
                self.write_opcode(OpCode::IfEq)?;
                self.write_i24(offset)?;
            }
            Op::IfFalse { offset } => {
                self.write_opcode(OpCode::IfFalse)?;
                self.write_i24(offset)?;
            }
            Op::IfGe { offset } => {
                self.write_opcode(OpCode::IfGe)?;
                self.write_i24(offset)?;
            }
            Op::IfGt { offset } => {
                self.write_opcode(OpCode::IfGt)?;
                self.write_i24(offset)?;
            }
            Op::IfLe { offset } => {
                self.write_opcode(OpCode::IfLe)?;
                self.write_i24(offset)?;
            }
            Op::IfLt { offset } => {
                self.write_opcode(OpCode::IfLt)?;
                self.write_i24(offset)?;
            }
            Op::IfNge { offset } => {
                self.write_opcode(OpCode::IfNge)?;
                self.write_i24(offset)?;
            }
            Op::IfNgt { offset } => {
                self.write_opcode(OpCode::IfNgt)?;
                self.write_i24(offset)?;
            }
            Op::IfNle { offset } => {
                self.write_opcode(OpCode::IfNle)?;
                self.write_i24(offset)?;
            }
            Op::IfNlt { offset } => {
                self.write_opcode(OpCode::IfNlt)?;
                self.write_i24(offset)?;
            }
            Op::IfNe { offset } => {
                self.write_opcode(OpCode::IfNe)?;
                self.write_i24(offset)?;
            }
            Op::IfStrictEq { offset } => {
                self.write_opcode(OpCode::IfStrictEq)?;
                self.write_i24(offset)?;
            }
            Op::IfStrictNe { offset } => {
                self.write_opcode(OpCode::IfStrictNe)?;
                self.write_i24(offset)?;
            }
            Op::IfTrue { offset } => {
                self.write_opcode(OpCode::IfTrue)?;
                self.write_i24(offset)?;
            }
            Op::In => self.write_opcode(OpCode::In)?,
            Op::IncLocal { index } => {
                self.write_opcode(OpCode::IncLocal)?;
                self.write_u30(index)?;
            }
            Op::IncLocalI { index } => {
                self.write_opcode(OpCode::IncLocalI)?;
                self.write_u30(index)?;
            }
            Op::Increment => self.write_opcode(OpCode::Increment)?,
            Op::IncrementI => self.write_opcode(OpCode::IncrementI)?,
            Op::InitProperty { ref index } => {
                self.write_opcode(OpCode::InitProperty)?;
                self.write_index(index)?;
            }
            Op::InstanceOf => self.write_opcode(OpCode::InstanceOf)?,
            Op::IsType { ref index } => {
                self.write_opcode(OpCode::IsType)?;
                self.write_index(index)?;
            }
            Op::IsTypeLate => self.write_opcode(OpCode::IsTypeLate)?,
            Op::Jump { offset } => {
                self.write_opcode(OpCode::Jump)?;
                self.write_i24(offset)?;
            }
            Op::Kill { index } => {
                self.write_opcode(OpCode::Kill)?;
                self.write_u30(index)?;
            }
            Op::Label => self.write_opcode(OpCode::Label)?,
            Op::LessEquals => self.write_opcode(OpCode::LessEquals)?,
            Op::LessThan => self.write_opcode(OpCode::LessThan)?,
            Op::Lf32 => self.write_opcode(OpCode::Lf32)?,
            Op::Lf64 => self.write_opcode(OpCode::Lf64)?,
            Op::Li16 => self.write_opcode(OpCode::Li16)?,
            Op::Li32 => self.write_opcode(OpCode::Li32)?,
            Op::Li8 => self.write_opcode(OpCode::Li8)?,
            Op::LookupSwitch {
                default_offset,
                ref case_offsets,
            } => {
                self.write_opcode(OpCode::LookupSwitch)?;
                self.write_i24(default_offset)?;
                self.write_u30(case_offsets.len() as u32 - 1)?;
                for offset in case_offsets {
                    self.write_i24(*offset)?;
                }
            }
            Op::LShift => self.write_opcode(OpCode::LShift)?,
            Op::Modulo => self.write_opcode(OpCode::Modulo)?,
            Op::Multiply => self.write_opcode(OpCode::Multiply)?,
            Op::MultiplyI => self.write_opcode(OpCode::MultiplyI)?,
            Op::Negate => self.write_opcode(OpCode::Negate)?,
            Op::NegateI => self.write_opcode(OpCode::NegateI)?,
            Op::NewActivation => self.write_opcode(OpCode::NewActivation)?,
            Op::NewArray { num_args } => {
                self.write_opcode(OpCode::NewArray)?;
                self.write_u30(num_args)?;
            }
            Op::NewCatch { ref index } => {
                self.write_opcode(OpCode::NewCatch)?;
                self.write_index(index)?;
            }
            Op::NewClass { ref index } => {
                self.write_opcode(OpCode::NewClass)?;
                self.write_index(index)?;
            }
            Op::NewFunction { ref index } => {
                self.write_opcode(OpCode::NewFunction)?;
                self.write_index(index)?;
            }
            Op::NewObject { num_args } => {
                self.write_opcode(OpCode::NewObject)?;
                self.write_u30(num_args)?;
            }
            Op::NextName => self.write_opcode(OpCode::NextName)?,
            Op::NextValue => self.write_opcode(OpCode::NextValue)?,
            Op::Nop => self.write_opcode(OpCode::Nop)?,
            Op::Not => self.write_opcode(OpCode::Not)?,
            Op::Pop => self.write_opcode(OpCode::Pop)?,
            Op::PopScope => self.write_opcode(OpCode::PopScope)?,
            Op::PushByte { value } => {
                self.write_opcode(OpCode::PushByte)?;
                self.write_u8(value)?;
            }
            Op::PushConstant { value } => {
                self.write_opcode(OpCode::PushConstant)?;
                self.write_u30(value)?;
            }
            Op::PushDouble { ref value } => {
                self.write_opcode(OpCode::PushDouble)?;
                self.write_index(value)?;
            }
            Op::PushFalse => self.write_opcode(OpCode::PushFalse)?,
            Op::PushInt { ref value } => {
                self.write_opcode(OpCode::PushInt)?;
                self.write_index(value)?;
            }
            Op::PushNamespace { ref value } => {
                self.write_opcode(OpCode::PushNamespace)?;
                self.write_index(value)?;
            }
            Op::PushNaN => self.write_opcode(OpCode::PushNaN)?,
            Op::PushNull => self.write_opcode(OpCode::PushNull)?,
            Op::PushScope => self.write_opcode(OpCode::PushScope)?,
            Op::PushShort { value } => {
                self.write_opcode(OpCode::PushShort)?;
                self.write_u30(value as u32)?;
            }
            Op::PushString { ref value } => {
                self.write_opcode(OpCode::PushString)?;
                self.write_index(value)?;
            }
            Op::PushTrue => self.write_opcode(OpCode::PushTrue)?,
            Op::PushUint { ref value } => {
                self.write_opcode(OpCode::PushUint)?;
                self.write_index(value)?;
            }
            Op::PushUndefined => self.write_opcode(OpCode::PushUndefined)?,
            Op::PushWith => self.write_opcode(OpCode::PushWith)?,
            Op::ReturnValue => self.write_opcode(OpCode::ReturnValue)?,
            Op::ReturnVoid => self.write_opcode(OpCode::ReturnVoid)?,
            Op::RShift => self.write_opcode(OpCode::RShift)?,
            Op::SetLocal { index } => match index {
                0 => self.write_opcode(OpCode::SetLocal0)?,
                1 => self.write_opcode(OpCode::SetLocal1)?,
                2 => self.write_opcode(OpCode::SetLocal2)?,
                3 => self.write_opcode(OpCode::SetLocal3)?,
                _ => {
                    self.write_opcode(OpCode::SetLocal)?;
                    self.write_u30(index)?;
                }
            },
            Op::SetGlobalSlot { index } => {
                self.write_opcode(OpCode::SetGlobalSlot)?;
                self.write_u30(index)?;
            }
            Op::SetProperty { ref index } => {
                self.write_opcode(OpCode::SetProperty)?;
                self.write_index(index)?;
            }
            Op::SetSlot { index } => {
                self.write_opcode(OpCode::SetSlot)?;
                self.write_u30(index)?;
            }
            Op::SetSuper { ref index } => {
                self.write_opcode(OpCode::SetSuper)?;
                self.write_index(index)?;
            }
            Op::Sf32 => self.write_opcode(OpCode::Sf32)?,
            Op::Sf64 => self.write_opcode(OpCode::Sf64)?,
            Op::Si16 => self.write_opcode(OpCode::Si16)?,
            Op::Si32 => self.write_opcode(OpCode::Si32)?,
            Op::Si8 => self.write_opcode(OpCode::Si8)?,
            Op::StrictEquals => self.write_opcode(OpCode::StrictEquals)?,
            Op::Subtract => self.write_opcode(OpCode::Subtract)?,
            Op::SubtractI => self.write_opcode(OpCode::SubtractI)?,
            Op::Swap => self.write_opcode(OpCode::Swap)?,
            Op::Sxi1 => self.write_opcode(OpCode::Sxi1)?,
            Op::Sxi16 => self.write_opcode(OpCode::Sxi16)?,
            Op::Sxi8 => self.write_opcode(OpCode::Sxi8)?,
            Op::Throw => self.write_opcode(OpCode::Throw)?,
            Op::Timestamp => self.write_opcode(OpCode::Timestamp)?,
            Op::TypeOf => self.write_opcode(OpCode::TypeOf)?,
            Op::URShift => self.write_opcode(OpCode::URShift)?,
        };

        Ok(())
    }

    fn write_opcode(&mut self, opcode: OpCode) -> Result<()> {
        self.write_u8(opcode as u8)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test_data;

    #[test]
    fn write_abc() {
        for (_, abc_file, bytes) in test_data::avm2_tests() {
            let mut out = vec![];
            {
                let mut writer = Writer::new(&mut out);
                writer.write(abc_file).unwrap();
            }
            assert_eq!(
                out, bytes,
                "Incorrectly written ABC.\nWritten:\n{:?}\n\nExpected:\n{:?}",
                out, bytes
            );
        }
    }

    #[test]
    fn write_i24() {
        let write = |n: i32| {
            let mut out = vec![];
            {
                let mut writer = Writer::new(&mut out);
                writer.write_i24(n).unwrap();
            }
            out
        };

        assert_eq!(write(0), &[0, 0, 0]);
        assert_eq!(write(2), &[2, 0, 0]);
        assert_eq!(write(77777), &[0b1101_0001, 0b0010_1111, 0b0000_0001]);
        assert_eq!(write(-77777), &[0b0010_1111, 0b1101_0000, 0b1111_1110]);
    }
}
