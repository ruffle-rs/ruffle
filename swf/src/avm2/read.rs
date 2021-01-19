use crate::avm2::types::*;
use crate::error::{Error, Result};
use crate::read::SwfReadExt;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};

pub struct Reader<R: Read> {
    input: R,
}

impl<R> Reader<R>
where
    R: Read + Seek,
{
    #[inline]
    pub fn seek(&mut self, relative_offset: i64) -> std::io::Result<u64> {
        self.input.seek(SeekFrom::Current(relative_offset as i64))
    }
}

impl<R: Read> Reader<R> {
    pub fn new(input: R) -> Reader<R> {
        Reader { input }
    }

    pub fn read(&mut self) -> Result<AbcFile> {
        let minor_version = self.read_u16()?;
        let major_version = self.read_u16()?;
        let constant_pool = self.read_constant_pool()?;

        let len = self.read_u30()?;
        let mut methods = vec![];
        for _ in 0..len {
            methods.push(self.read_method()?);
        }

        let len = self.read_u30()? as usize;
        let mut metadata = Vec::with_capacity(len);
        for _ in 0..len {
            metadata.push(self.read_metadata()?);
        }

        let len = self.read_u30()? as usize;
        let mut instances = Vec::with_capacity(len);
        for _ in 0..len {
            instances.push(self.read_instance()?);
        }

        let mut classes = Vec::with_capacity(len);
        for _ in 0..len {
            classes.push(self.read_class()?);
        }

        let len = self.read_u30()? as usize;
        let mut scripts = Vec::with_capacity(len);
        for _ in 0..len {
            scripts.push(self.read_script()?);
        }

        let len = self.read_u30()? as usize;
        let mut method_bodies = Vec::with_capacity(len);
        for _ in 0..len {
            method_bodies.push(self.read_method_body()?);
        }

        Ok(AbcFile {
            major_version,
            minor_version,

            constant_pool,
            methods,
            metadata,
            instances,
            classes,
            scripts,
            method_bodies,
        })
    }

    fn read_u30(&mut self) -> Result<u32> {
        let mut n = 0;
        let mut i = 0;
        loop {
            let byte: u32 = self.read_u8()?.into();
            n |= (byte & 0b0111_1111) << i;
            i += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(n)
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.read_u30()
    }

    fn read_i24(&mut self) -> Result<i32> {
        Ok(i32::from(self.read_u8()? as i8)
            | (i32::from(self.read_u8()? as i8) << 8)
            | (i32::from(self.read_u8()? as i8) << 16))
    }
    fn read_i32(&mut self) -> Result<i32> {
        let mut n: i32 = 0;
        let mut i = 0;
        loop {
            let byte: i32 = self.read_u8()?.into();
            n |= (byte & 0b0111_1111) << i;
            i += 7;

            if byte & 0b1000_0000 == 0 {
                if i < 32 {
                    n <<= 32 - i;
                    n >>= 32 - i;
                }

                break;
            }
        }
        Ok(n)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_u30()? as usize;
        let mut s = String::with_capacity(len);
        self.input
            .by_ref()
            .take(len as u64)
            .read_to_string(&mut s)?;
        Ok(s)
    }

    fn read_index<T>(&mut self) -> Result<Index<T>> {
        use std::marker::PhantomData;
        Ok(Index(self.read_u30()?, PhantomData))
    }

    fn read_namespace(&mut self) -> Result<Namespace> {
        let kind = self.read_u8()?;
        let name: Index<String> = self.read_index()?;
        // TODO: AVM2 specs say that "non-system" namespaces
        // should have an empty name?
        Ok(match kind {
            0x05 => Namespace::Private(name),
            0x08 => Namespace::Namespace(name),
            0x16 => Namespace::Package(name),
            0x17 => Namespace::PackageInternal(name),
            0x18 => Namespace::Protected(name),
            0x19 => Namespace::Explicit(name),
            0x1a => Namespace::StaticProtected(name),
            _ => return Err(Error::invalid_data("Invalid namespace kind")),
        })
    }

    fn read_namespace_set(&mut self) -> Result<NamespaceSet> {
        let len = self.read_u30()? as usize;
        let mut namespace_set = vec![];
        for _ in 0..len {
            namespace_set.push(self.read_index()?);
        }
        Ok(namespace_set)
    }

    fn read_multiname(&mut self) -> Result<Multiname> {
        let kind = self.read_u8()?;
        Ok(match kind {
            0x07 => Multiname::QName {
                namespace: self.read_index()?,
                name: self.read_index()?,
            },
            0x0d => Multiname::QNameA {
                namespace: self.read_index()?,
                name: self.read_index()?,
            },
            0x0f => Multiname::RTQName {
                name: self.read_index()?,
            },
            0x10 => Multiname::RTQNameA {
                name: self.read_index()?,
            },
            0x11 => Multiname::RTQNameL,
            0x12 => Multiname::RTQNameLA,
            0x09 => Multiname::Multiname {
                name: self.read_index()?,
                namespace_set: self.read_index()?,
            },
            0x0e => Multiname::MultinameA {
                name: self.read_index()?,
                namespace_set: self.read_index()?,
            },
            0x1b => Multiname::MultinameL {
                namespace_set: self.read_index()?,
            },
            0x1c => Multiname::MultinameLA {
                namespace_set: self.read_index()?,
            },
            _ => return Err(Error::invalid_data("Invalid multiname kind")),
        })
    }

    fn read_constant_pool(&mut self) -> Result<ConstantPool> {
        let len = self.read_u30()?;
        let mut ints = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                ints.push(self.read_i32()?);
            }
        }

        let len = self.read_u30()?;
        let mut uints = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                uints.push(self.read_u32()?);
            }
        }

        let len = self.read_u30()?;
        let mut doubles = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                doubles.push(self.read_f64()?);
            }
        }

        let len = self.read_u30()?;
        let mut strings = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                strings.push(self.read_string()?);
            }
        }

        let len = self.read_u30()?;
        let mut namespaces = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                namespaces.push(self.read_namespace()?);
            }
        }

        let len = self.read_u30()?;
        let mut namespace_sets = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                namespace_sets.push(self.read_namespace_set()?);
            }
        }

        let len = self.read_u30()?;
        let mut multinames = Vec::with_capacity(len as usize);
        if len > 0 {
            for _ in 0..len - 1 {
                multinames.push(self.read_multiname()?);
            }
        }

        Ok(ConstantPool {
            ints,
            uints,
            doubles,
            strings,
            namespaces,
            namespace_sets,
            multinames,
        })
    }

    fn read_method(&mut self) -> Result<Method> {
        let num_params = self.read_u8()? as usize;
        let return_type = self.read_index()?;
        let mut params = vec![];
        for _ in 0..num_params {
            params.push(MethodParam {
                kind: self.read_index()?,
                name: None,
                default_value: None,
            })
        }
        let name = self.read_index()?;
        let flags = self.read_u8()?;

        if flags & 0x08 != 0 {
            let num_optional_params = self.read_u30()? as usize;
            #[allow(clippy::needless_range_loop)]
            for i in 0..num_optional_params {
                params[i].default_value = Some(self.read_constant_value()?);
            }
        }

        if flags & 0x80 != 0 {
            #[allow(clippy::needless_range_loop)]
            for i in 0..num_params {
                params[i].name = Some(self.read_index()?);
            }
        }

        Ok(Method {
            name,
            params,
            return_type,
            needs_arguments_object: flags & 0x01 != 0,
            needs_activation: flags & 0x02 != 0,
            needs_rest: flags & 0x04 != 0,
            needs_dxns: flags & 0x40 != 0,
        })
    }

    fn read_constant_value(&mut self) -> Result<DefaultValue> {
        let index = self.read_u30()?;
        Ok(match self.read_u8()? {
            0x00 => DefaultValue::Undefined,
            0x01 => DefaultValue::String(Index::new(index)),
            0x03 => DefaultValue::Int(Index::new(index)),
            0x04 => DefaultValue::Uint(Index::new(index)),
            0x05 => DefaultValue::Private(Index::new(index)),
            0x06 => DefaultValue::Double(Index::new(index)),
            0x08 => DefaultValue::Namespace(Index::new(index)),
            0x0a => DefaultValue::False,
            0x0b => DefaultValue::True,
            0x0c => DefaultValue::Null,
            0x16 => DefaultValue::Package(Index::new(index)),
            0x17 => DefaultValue::PackageInternal(Index::new(index)),
            0x18 => DefaultValue::Protected(Index::new(index)),
            0x19 => DefaultValue::Explicit(Index::new(index)),
            0x1a => DefaultValue::StaticProtected(Index::new(index)),
            _ => return Err(Error::invalid_data("Invalid default value")),
        })
    }

    fn read_optional_value(&mut self) -> Result<Option<DefaultValue>> {
        let index = self.read_u30()?;
        if index == 0 {
            Ok(None)
        } else {
            Ok(Some(match self.read_u8()? {
                0x00 => DefaultValue::Undefined,
                0x01 => DefaultValue::String(Index::new(index)),
                0x03 => DefaultValue::Int(Index::new(index)),
                0x04 => DefaultValue::Uint(Index::new(index)),
                0x05 => DefaultValue::Private(Index::new(index)),
                0x06 => DefaultValue::Double(Index::new(index)),
                0x08 => DefaultValue::Namespace(Index::new(index)),
                0x0a => DefaultValue::False,
                0x0b => DefaultValue::True,
                0x0c => DefaultValue::Null,
                0x16 => DefaultValue::Package(Index::new(index)),
                0x17 => DefaultValue::PackageInternal(Index::new(index)),
                0x18 => DefaultValue::Protected(Index::new(index)),
                0x19 => DefaultValue::Explicit(Index::new(index)),
                0x1a => DefaultValue::StaticProtected(Index::new(index)),
                _ => return Err(Error::invalid_data("Invalid default value")),
            }))
        }
    }

    fn read_metadata(&mut self) -> Result<Metadata> {
        let name = self.read_index()?;
        let mut items = vec![];
        let num_items = self.read_u30()?;
        for _ in 0..num_items {
            items.push(MetadataItem {
                key: self.read_index()?,
                value: self.read_index()?,
            })
        }
        Ok(Metadata { name, items })
    }

    fn read_instance(&mut self) -> Result<Instance> {
        let name = self.read_index()?;
        let super_name = self.read_index()?;
        let flags = self.read_u8()?;

        let protected_namespace = if flags & 0x08 != 0 {
            Some(self.read_index()?)
        } else {
            None
        };

        let num_interfaces = self.read_u30()? as usize;
        let mut interfaces = Vec::with_capacity(num_interfaces);
        for _ in 0..num_interfaces {
            interfaces.push(self.read_index()?);
        }

        let init_method = self.read_index()?;

        let num_traits = self.read_u30()? as usize;
        let mut traits = Vec::with_capacity(num_traits);
        for _ in 0..num_traits {
            traits.push(self.read_trait()?);
        }

        Ok(Instance {
            name,
            super_name,
            protected_namespace,
            interfaces,
            traits,
            init_method,
            is_sealed: flags & 0x01 != 0,
            is_final: flags & 0x02 != 0,
            is_interface: flags & 0x04 != 0,
        })
    }

    fn read_class(&mut self) -> Result<Class> {
        let init_method = self.read_index()?;
        let num_traits = self.read_u30()? as usize;
        let mut traits = Vec::with_capacity(num_traits);
        for _ in 0..num_traits {
            traits.push(self.read_trait()?);
        }
        Ok(Class {
            init_method,
            traits,
        })
    }

    fn read_script(&mut self) -> Result<Script> {
        let init_method = self.read_index()?;
        let num_traits = self.read_u30()? as usize;
        let mut traits = Vec::with_capacity(num_traits);
        for _ in 0..num_traits {
            traits.push(self.read_trait()?);
        }
        Ok(Script {
            init_method,
            traits,
        })
    }

    fn read_trait(&mut self) -> Result<Trait> {
        let name = self.read_index()?;
        let flags = self.read_u8()?;
        let kind = match flags & 0b1111 {
            0 => TraitKind::Slot {
                slot_id: self.read_u30()?,
                type_name: self.read_index()?,
                value: self.read_optional_value()?,
            },
            1 => TraitKind::Method {
                disp_id: self.read_u30()?,
                method: self.read_index()?,
            },
            2 => TraitKind::Getter {
                disp_id: self.read_u30()?,
                method: self.read_index()?,
            },
            3 => TraitKind::Setter {
                disp_id: self.read_u30()?,
                method: self.read_index()?,
            },
            4 => TraitKind::Class {
                slot_id: self.read_u30()?,
                class: self.read_index()?,
            },
            5 => TraitKind::Function {
                slot_id: self.read_u30()?,
                function: self.read_index()?,
            },
            6 => TraitKind::Const {
                slot_id: self.read_u30()?,
                type_name: self.read_index()?,
                value: self.read_optional_value()?,
            },
            _ => return Err(Error::invalid_data("Invalid trait kind")),
        };

        let mut metadata = vec![];
        if flags & 0b0100_0000 != 0 {
            let num_metadata = self.read_u30()? as usize;
            metadata.reserve(num_metadata);
            for _ in 0..num_metadata {
                metadata.push(self.read_index()?);
            }
        }

        Ok(Trait {
            name,
            kind,
            metadata,
            is_final: flags & 0b0001_0000 != 0,
            is_override: flags & 0b0010_0000 != 0,
        })
    }

    fn read_method_body(&mut self) -> Result<MethodBody> {
        let method = self.read_index()?;
        let max_stack = self.read_u30()?;
        let num_locals = self.read_u30()?;
        let init_scope_depth = self.read_u30()?;
        let max_scope_depth = self.read_u30()?;

        // Read the code data.
        let code_len = self.read_u30()?;
        let mut code = Vec::with_capacity(code_len as usize);
        self.input
            .by_ref()
            .take(code_len.into())
            .read_to_end(&mut code)?;

        let num_exceptions = self.read_u30()? as usize;
        let mut exceptions = Vec::with_capacity(num_exceptions);
        for _ in 0..num_exceptions {
            exceptions.push(self.read_exception()?);
        }

        let num_traits = self.read_u30()? as usize;
        let mut traits = Vec::with_capacity(num_traits);
        for _ in 0..num_traits {
            traits.push(self.read_trait()?);
        }

        Ok(MethodBody {
            method,
            max_stack,
            num_locals,
            init_scope_depth,
            max_scope_depth,
            code,
            exceptions,
            traits,
        })
    }

    pub fn read_op(&mut self) -> Result<Option<Op>> {
        use crate::avm2::opcode::OpCode;
        use num_traits::FromPrimitive;

        let byte = self.read_u8()?;
        let opcode = match OpCode::from_u8(byte) {
            Some(o) => o,
            None => {
                return Err(Error::invalid_data(format!(
                    "Unknown ABC opcode {:#x}",
                    byte
                )))
            }
        };

        let op = match opcode {
            OpCode::Add => Op::Add,
            OpCode::AddI => Op::AddI,
            OpCode::AsType => Op::AsType {
                type_name: self.read_index()?,
            },
            OpCode::AsTypeLate => Op::AsTypeLate,
            OpCode::BitAnd => Op::BitAnd,
            OpCode::BitNot => Op::BitNot,
            OpCode::BitOr => Op::BitOr,
            OpCode::BitXor => Op::BitXor,
            OpCode::Call => Op::Call {
                num_args: self.read_u30()?,
            },
            OpCode::CallMethod => Op::CallMethod {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallProperty => Op::CallProperty {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallPropLex => Op::CallPropLex {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallPropVoid => Op::CallPropVoid {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallStatic => Op::CallStatic {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallSuper => Op::CallSuper {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CallSuperVoid => Op::CallSuperVoid {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::CheckFilter => Op::CheckFilter,
            OpCode::Coerce => Op::Coerce {
                index: self.read_index()?,
            },
            OpCode::CoerceA => Op::CoerceA,
            OpCode::CoerceS => Op::CoerceS,
            OpCode::Construct => Op::Construct {
                num_args: self.read_u30()?,
            },
            OpCode::ConstructProp => Op::ConstructProp {
                index: self.read_index()?,
                num_args: self.read_u30()?,
            },
            OpCode::ConstructSuper => Op::ConstructSuper {
                num_args: self.read_u30()?,
            },
            OpCode::ConvertB => Op::ConvertB,
            OpCode::ConvertD => Op::ConvertD,
            OpCode::ConvertI => Op::ConvertI,
            OpCode::ConvertO => Op::ConvertO,
            OpCode::ConvertS => Op::ConvertS,
            OpCode::ConvertU => Op::ConvertU,
            OpCode::Debug => {
                let op = Op::Debug {
                    is_local_register: self.read_u8()? != 0,
                    register_name: self.read_index()?,
                    register: self.read_u8()?,
                };
                self.read_u30()?; // Unused
                op
            }
            OpCode::DebugFile => Op::DebugFile {
                file_name: self.read_index()?,
            },
            OpCode::DebugLine => Op::DebugLine {
                line_num: self.read_u30()?,
            },
            OpCode::DecLocal => Op::DecLocal {
                index: self.read_u30()?,
            },
            OpCode::DecLocalI => Op::DecLocalI {
                index: self.read_u30()?,
            },
            OpCode::Decrement => Op::Decrement,
            OpCode::DecrementI => Op::DecrementI,
            OpCode::DeleteProperty => Op::DeleteProperty {
                index: self.read_index()?,
            },
            OpCode::Divide => Op::Divide,
            OpCode::Dup => Op::Dup,
            OpCode::Dxns => Op::Dxns {
                index: self.read_index()?,
            },
            OpCode::DxnsLate => Op::DxnsLate,
            OpCode::Equals => Op::Equals,
            OpCode::EscXAttr => Op::EscXAttr,
            OpCode::EscXElem => Op::EscXElem,
            OpCode::FindProperty => Op::FindProperty {
                index: self.read_index()?,
            },
            OpCode::FindPropStrict => Op::FindPropStrict {
                index: self.read_index()?,
            },
            OpCode::GetDescendants => Op::GetDescendants {
                index: self.read_index()?,
            },
            OpCode::GetGlobalScope => Op::GetGlobalScope,
            OpCode::GetGlobalSlot => Op::GetGlobalSlot {
                index: self.read_u30()?,
            },
            OpCode::GetLex => Op::GetLex {
                index: self.read_index()?,
            },
            OpCode::GetLocal => Op::GetLocal {
                index: self.read_u30()?,
            },
            OpCode::GetLocal0 => Op::GetLocal { index: 0 },
            OpCode::GetLocal1 => Op::GetLocal { index: 1 },
            OpCode::GetLocal2 => Op::GetLocal { index: 2 },
            OpCode::GetLocal3 => Op::GetLocal { index: 3 },
            OpCode::GetProperty => Op::GetProperty {
                index: self.read_index()?,
            },
            OpCode::GetScopeObject => Op::GetScopeObject {
                index: self.read_u8()?,
            },
            OpCode::GetSlot => Op::GetSlot {
                index: self.read_u30()?,
            },
            OpCode::GetSuper => Op::GetSuper {
                index: self.read_index()?,
            },
            OpCode::GreaterEquals => Op::GreaterEquals,
            OpCode::GreaterThan => Op::GreaterThan,
            OpCode::HasNext => Op::HasNext,
            OpCode::HasNext2 => Op::HasNext2 {
                object_register: self.read_u30()?,
                index_register: self.read_u30()?,
            },
            OpCode::IfEq => Op::IfEq {
                offset: self.read_i24()?,
            },
            OpCode::IfFalse => Op::IfFalse {
                offset: self.read_i24()?,
            },
            OpCode::IfGe => Op::IfGe {
                offset: self.read_i24()?,
            },
            OpCode::IfGt => Op::IfGt {
                offset: self.read_i24()?,
            },
            OpCode::IfLe => Op::IfLe {
                offset: self.read_i24()?,
            },
            OpCode::IfLt => Op::IfLt {
                offset: self.read_i24()?,
            },
            OpCode::IfNge => Op::IfNge {
                offset: self.read_i24()?,
            },
            OpCode::IfNgt => Op::IfNgt {
                offset: self.read_i24()?,
            },
            OpCode::IfNle => Op::IfNle {
                offset: self.read_i24()?,
            },
            OpCode::IfNlt => Op::IfNlt {
                offset: self.read_i24()?,
            },
            OpCode::IfNe => Op::IfNe {
                offset: self.read_i24()?,
            },
            OpCode::IfStrictEq => Op::IfStrictEq {
                offset: self.read_i24()?,
            },
            OpCode::IfStrictNe => Op::IfStrictNe {
                offset: self.read_i24()?,
            },
            OpCode::IfTrue => Op::IfTrue {
                offset: self.read_i24()?,
            },
            OpCode::In => Op::In,
            OpCode::IncLocal => Op::IncLocal {
                index: self.read_u30()?,
            },
            OpCode::IncLocalI => Op::IncLocalI {
                index: self.read_u30()?,
            },
            OpCode::Increment => Op::Increment,
            OpCode::IncrementI => Op::IncrementI,
            OpCode::InitProperty => Op::InitProperty {
                index: self.read_index()?,
            },
            OpCode::InstanceOf => Op::InstanceOf,
            OpCode::IsType => Op::IsType {
                index: self.read_index()?,
            },
            OpCode::IsTypeLate => Op::IsTypeLate,
            OpCode::Jump => Op::Jump {
                offset: self.read_i24()?,
            },
            OpCode::Kill => Op::Kill {
                index: self.read_u30()?,
            },
            OpCode::Label => Op::Label,
            OpCode::LessEquals => Op::LessEquals,
            OpCode::LessThan => Op::LessThan,
            OpCode::LookupSwitch => Op::LookupSwitch {
                default_offset: self.read_i24()?,
                case_offsets: {
                    let num_cases = self.read_u30()? + 1;
                    let mut case_offsets = vec![];
                    for _ in 0..num_cases {
                        case_offsets.push(self.read_i24()?);
                    }
                    case_offsets
                },
            },
            OpCode::LShift => Op::LShift,
            OpCode::Modulo => Op::Modulo,
            OpCode::Multiply => Op::Multiply,
            OpCode::MultiplyI => Op::MultiplyI,
            OpCode::Negate => Op::Negate,
            OpCode::NegateI => Op::NegateI,
            OpCode::NewActivation => Op::NewActivation,
            OpCode::NewArray => Op::NewArray {
                num_args: self.read_u30()?,
            },
            OpCode::NewCatch => Op::NewCatch {
                index: self.read_index()?,
            },
            OpCode::NewClass => Op::NewClass {
                index: self.read_index()?,
            },
            OpCode::NewFunction => Op::NewFunction {
                index: self.read_index()?,
            },
            OpCode::NewObject => Op::NewObject {
                num_args: self.read_u30()?,
            },
            OpCode::NextName => Op::NextName,
            OpCode::NextValue => Op::NextValue,
            OpCode::Nop => Op::Nop,
            OpCode::Not => Op::Not,
            OpCode::Pop => Op::Pop,
            OpCode::PopScope => Op::PopScope,
            OpCode::PushByte => Op::PushByte {
                value: self.read_u8()?,
            },
            OpCode::PushDouble => Op::PushDouble {
                value: self.read_index()?,
            },
            OpCode::PushFalse => Op::PushFalse,
            OpCode::PushInt => Op::PushInt {
                value: self.read_index()?,
            },
            OpCode::PushNamespace => Op::PushNamespace {
                value: self.read_index()?,
            },
            OpCode::PushNaN => Op::PushNaN,
            OpCode::PushNull => Op::PushNull,
            OpCode::PushScope => Op::PushScope,
            OpCode::PushShort => Op::PushShort {
                value: self.read_u30()? as i16,
            },
            OpCode::PushString => Op::PushString {
                value: self.read_index()?,
            },
            OpCode::PushTrue => Op::PushTrue,
            OpCode::PushUint => Op::PushUint {
                value: self.read_index()?,
            },
            OpCode::PushUndefined => Op::PushUndefined,
            OpCode::PushWith => Op::PushWith,
            OpCode::ReturnValue => Op::ReturnValue,
            OpCode::ReturnVoid => Op::ReturnVoid,
            OpCode::RShift => Op::RShift,
            OpCode::SetLocal => Op::SetLocal {
                index: self.read_u30()?,
            },
            OpCode::SetLocal0 => Op::SetLocal { index: 0 },
            OpCode::SetLocal1 => Op::SetLocal { index: 1 },
            OpCode::SetLocal2 => Op::SetLocal { index: 2 },
            OpCode::SetLocal3 => Op::SetLocal { index: 3 },
            OpCode::SetGlobalSlot => Op::SetGlobalSlot {
                index: self.read_u30()?,
            },
            OpCode::SetProperty => Op::SetProperty {
                index: self.read_index()?,
            },
            OpCode::SetSlot => Op::SetSlot {
                index: self.read_u30()?,
            },
            OpCode::SetSuper => Op::SetSuper {
                index: self.read_index()?,
            },
            OpCode::StrictEquals => Op::StrictEquals,
            OpCode::Subtract => Op::Subtract,
            OpCode::SubtractI => Op::SubtractI,
            OpCode::Swap => Op::Swap,
            OpCode::Throw => Op::Throw,
            OpCode::TypeOf => Op::TypeOf,
            OpCode::URShift => Op::URShift,
        };

        Ok(Some(op))
    }

    fn read_exception(&mut self) -> Result<Exception> {
        Ok(Exception {
            from_offset: self.read_u30()?,
            to_offset: self.read_u30()?,
            target_offset: self.read_u30()?,
            type_name: self.read_index()?,
            variable_name: self.read_index()?,
        })
    }
}

impl<'a, R: 'a + Read> SwfReadExt for Reader<R> {
    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        self.input.read_u8()
    }

    #[inline]
    fn read_u16(&mut self) -> io::Result<u16> {
        self.input.read_u16::<LittleEndian>()
    }

    #[inline]
    fn read_u32(&mut self) -> io::Result<u32> {
        self.input.read_u32::<LittleEndian>()
    }

    #[inline]
    fn read_u64(&mut self) -> io::Result<u64> {
        self.input.read_u64::<LittleEndian>()
    }

    #[inline]
    fn read_i8(&mut self) -> io::Result<i8> {
        self.input.read_i8()
    }

    #[inline]
    fn read_i16(&mut self) -> io::Result<i16> {
        self.input.read_i16::<LittleEndian>()
    }

    #[inline]
    fn read_i32(&mut self) -> io::Result<i32> {
        self.input.read_i32::<LittleEndian>()
    }

    #[inline]
    fn read_f32(&mut self) -> io::Result<f32> {
        self.input.read_f32::<LittleEndian>()
    }

    #[inline]
    fn read_f64(&mut self) -> io::Result<f64> {
        self.input.read_f64::<LittleEndian>()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test_data;

    pub fn read_abc_from_file(path: &str) -> Vec<u8> {
        use crate::types::Tag;
        let data = std::fs::read(path).unwrap();
        let swf_buf = crate::decompress_swf(&data[..]).unwrap();
        let swf = crate::parse_swf(&swf_buf).unwrap();
        for tag in swf.tags {
            if let Tag::DoAbc(do_abc) = tag {
                return do_abc.data.to_vec();
            }
        }
        panic!("ABC tag not found in {}", path);
    }

    #[test]
    fn read_abc() {
        for (_, abc_file, bytes) in test_data::avm2_tests() {
            let mut reader = Reader::new(&bytes[..]);
            let parsed = reader.read().unwrap();
            if parsed != abc_file {
                // Failed, result doesn't match.
                panic!(
                    "Incorrectly parsed ABC.\nRead:\n{:?}\n\nExpected:\n{:?}",
                    parsed, abc_file
                );
            }
        }
    }
}
