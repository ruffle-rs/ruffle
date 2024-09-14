use crate::avm2::types::*;
use crate::error::{Error, Result};
use crate::extensions::ReadSwfExt;
use std::io::Read;

pub struct Reader<'a> {
    input: &'a [u8],
}

impl<'a> ReadSwfExt<'a> for Reader<'a> {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut &'a [u8] {
        &mut self.input
    }

    #[inline(always)]
    fn as_slice(&self) -> &'a [u8] {
        self.input
    }
}

impl<'a> Reader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    #[inline]
    pub fn seek(&mut self, data: &'a [u8], relative_offset: i32) {
        ReadSwfExt::seek(self, data, relative_offset as isize)
    }

    #[inline]
    pub fn seek_absolute(&mut self, data: &'a [u8], pos: usize) {
        ReadSwfExt::seek_absolute(self, data, pos)
    }

    pub fn read(&mut self) -> Result<AbcFile> {
        let minor_version = self.read_u16()?;
        let major_version = self.read_u16()?;
        let constant_pool = self.read_constant_pool()?;

        let len = self.read_u30()?;
        let mut methods = Vec::with_capacity(len as usize);
        for _ in 0..len {
            methods.push(self.read_method()?);
        }

        let len = self.read_u30()?;
        let metadata = self.read_metadata(len)?;

        let len = self.read_u30()?;
        let mut instances = Vec::with_capacity(len as usize);
        for _ in 0..len {
            instances.push(self.read_instance()?);
        }

        let mut classes = Vec::with_capacity(len as usize);
        for _ in 0..len {
            classes.push(self.read_class()?);
        }

        let len = self.read_u30()?;
        let mut scripts = Vec::with_capacity(len as usize);
        for _ in 0..len {
            scripts.push(self.read_script()?);
        }

        let len = self.read_u30()?;
        let mut method_bodies = Vec::with_capacity(len as usize);
        for body_idx in 0..len {
            let body = self.read_method_body()?;
            if methods[body.method.0 as usize].body.is_some() {
                // TODO: this should somehow throw error 1121 in FP.
                return Err(Error::invalid_data("Duplicate method body"));
            }
            methods[body.method.0 as usize].body = Some(Index::new(body_idx));
            method_bodies.push(body);
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
        self.read_encoded_u32()
    }

    fn read_i24(&mut self) -> Result<i32> {
        Ok(i32::from(self.read_u8()?)
            | (i32::from(self.read_u8()?) << 8)
            | (i32::from(self.read_u8()? as i8) << 16))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_encoded_u32()? as i32)
    }

    fn read_string(&mut self) -> Result<Vec<u8>> {
        let len = self.read_u30()?;
        // TODO: Avoid allocating a String.
        let mut s = Vec::with_capacity(len as usize);
        self.read_slice(len as usize)?.read_to_end(&mut s)?;
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
        let len = self.read_u30()?;
        let mut namespace_set = Vec::with_capacity(len as usize);
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
            0x1d => {
                let base_type = self.read_index()?;
                let count = self.read_u30()?;
                let mut parameters = Vec::with_capacity(count as usize);

                for _ in 0..count {
                    parameters.push(self.read_index()?);
                }

                Multiname::TypeName {
                    base_type,
                    parameters,
                }
            }
            _ => return Err(Error::invalid_data("Invalid multiname kind")),
        })
    }

    fn read_constant_pool(&mut self) -> Result<ConstantPool> {
        let len = self.read_u30()?.saturating_sub(1);
        let mut ints = Vec::with_capacity(len as usize);
        for _ in 0..len {
            ints.push(self.read_i32()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut uints = Vec::with_capacity(len as usize);
        for _ in 0..len {
            uints.push(self.read_u30()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut doubles = Vec::with_capacity(len as usize);
        for _ in 0..len {
            doubles.push(self.read_f64()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut strings = Vec::with_capacity(len as usize);
        for _ in 0..len {
            strings.push(self.read_string()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut namespaces = Vec::with_capacity(len as usize);
        for _ in 0..len {
            namespaces.push(self.read_namespace()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut namespace_sets = Vec::with_capacity(len as usize);
        for _ in 0..len {
            namespace_sets.push(self.read_namespace_set()?);
        }

        let len = self.read_u30()?.saturating_sub(1);
        let mut multinames = Vec::with_capacity(len as usize);
        for _ in 0..len {
            multinames.push(self.read_multiname()?);
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
        let num_params = self.read_u30()?;
        let return_type = self.read_index()?;
        let mut params = Vec::with_capacity(num_params as usize);
        for _ in 0..num_params {
            params.push(MethodParam {
                kind: self.read_index()?,
                name: None,
                default_value: None,
            })
        }
        let name = self.read_index()?;
        let flags = MethodFlags::from_bits_truncate(self.read_u8()?);

        if flags.contains(MethodFlags::HAS_OPTIONAL) {
            let num_optional_params = self.read_u30()? as usize;
            if let Some(start) = params.len().checked_sub(num_optional_params) {
                for param in &mut params[start..] {
                    param.default_value = Some(self.read_constant_value()?);
                }
            } else {
                return Err(Error::invalid_data("Too many optional parameters"));
            }
        }

        if flags.contains(MethodFlags::HAS_PARAM_NAMES) {
            for param in &mut params {
                param.name = Some(self.read_index()?);
            }
        }

        Ok(Method {
            name,
            params,
            return_type,
            flags,
            body: None,
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

    fn read_metadata(&mut self, len: u32) -> Result<Vec<Metadata>> {
        let mut metadata = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = self.read_index()?;
            let num_items = self.read_u30()?;
            let mut key_value_data = Vec::with_capacity(num_items as usize * 2);

            // Data includes the keys and values
            for _ in 0..num_items * 2 {
                key_value_data.push(self.read_index()?);
            }

            // Split them up here
            let mut items = Vec::with_capacity(num_items as usize);
            for i in 0..num_items {
                items.push(MetadataItem {
                    key: key_value_data[i as usize],
                    value: key_value_data[(num_items + i) as usize],
                })
            }

            metadata.push(Metadata { name, items });
        }

        Ok(metadata)
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

        let num_interfaces = self.read_u30()?;
        let mut interfaces = Vec::with_capacity(num_interfaces as usize);
        for _ in 0..num_interfaces {
            interfaces.push(self.read_index()?);
        }

        let init_method = self.read_index()?;

        let num_traits = self.read_u30()?;
        let mut traits = Vec::with_capacity(num_traits as usize);
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
        let num_traits = self.read_u30()?;
        let mut traits = Vec::with_capacity(num_traits as usize);
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
        let num_traits = self.read_u30()?;
        let mut traits = Vec::with_capacity(num_traits as usize);
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
            let num_metadata = self.read_u30()?;
            metadata.reserve(num_metadata as usize);
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
        // TODO: Avoid allocating a Vec.
        let code = self.read_slice(code_len as usize)?.to_vec();

        let num_exceptions = self.read_u30()?;
        let mut exceptions = Vec::with_capacity(num_exceptions as usize);
        for _ in 0..num_exceptions {
            exceptions.push(self.read_exception()?);
        }

        let num_traits = self.read_u30()?;
        let mut traits = Vec::with_capacity(num_traits as usize);
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

    pub fn read_op(&mut self) -> Result<Op> {
        use crate::avm2::opcode::OpCode;
        use num_traits::FromPrimitive;

        let byte = self.read_u8()?;
        let opcode = match OpCode::from_u8(byte) {
            Some(o) => o,
            None => return Err(Error::invalid_data(format!("Unknown ABC opcode {byte:#x}"))),
        };

        let op = match opcode {
            OpCode::Add => Op::Add,
            OpCode::AddI => Op::AddI,
            OpCode::ApplyType => Op::ApplyType {
                num_types: self.read_u30()?,
            },
            OpCode::AsType => Op::AsType {
                type_name: self.read_index()?,
            },
            OpCode::AsTypeLate => Op::AsTypeLate,
            OpCode::BitAnd => Op::BitAnd,
            OpCode::BitNot => Op::BitNot,
            OpCode::BitOr => Op::BitOr,
            OpCode::BitXor => Op::BitXor,
            OpCode::Bkpt => Op::Bkpt,
            OpCode::BkptLine => Op::BkptLine {
                line_num: self.read_u30()?,
            },
            OpCode::Call => Op::Call {
                num_args: self.read_u30()?,
            },
            OpCode::CallMethod => Op::CallMethod {
                index: self.read_u30()?,
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
            OpCode::CoerceB => Op::CoerceB,
            OpCode::CoerceD => Op::CoerceD,
            OpCode::CoerceI => Op::CoerceI,
            OpCode::CoerceO => Op::CoerceO,
            OpCode::CoerceS => Op::CoerceS,
            OpCode::CoerceU => Op::CoerceU,
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
            OpCode::FindDef => Op::FindDef {
                index: self.read_index()?,
            },
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
            OpCode::GetOuterScope => Op::GetOuterScope {
                index: self.read_u30()?,
            },
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
            OpCode::Lf32 => Op::Lf32,
            OpCode::Lf64 => Op::Lf64,
            OpCode::Li16 => Op::Li16,
            OpCode::Li32 => Op::Li32,
            OpCode::Li8 => Op::Li8,
            OpCode::LookupSwitch => Op::LookupSwitch(Box::new(LookupSwitch {
                default_offset: self.read_i24()?,
                case_offsets: {
                    let num_cases = self.read_u30()? + 1;
                    let mut case_offsets = Vec::with_capacity(num_cases as usize);
                    for _ in 0..num_cases {
                        case_offsets.push(self.read_i24()?);
                    }
                    case_offsets.into()
                },
            })),
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
            OpCode::Sf32 => Op::Sf32,
            OpCode::Sf64 => Op::Sf64,
            OpCode::Si16 => Op::Si16,
            OpCode::Si32 => Op::Si32,
            OpCode::Si8 => Op::Si8,
            OpCode::StrictEquals => Op::StrictEquals,
            OpCode::Subtract => Op::Subtract,
            OpCode::SubtractI => Op::SubtractI,
            OpCode::Swap => Op::Swap,
            OpCode::Sxi1 => Op::Sxi1,
            OpCode::Sxi16 => Op::Sxi16,
            OpCode::Sxi8 => Op::Sxi8,
            OpCode::Timestamp => Op::Timestamp,
            OpCode::Throw => Op::Throw,
            OpCode::TypeOf => Op::TypeOf,
            OpCode::URShift => Op::URShift,
        };

        Ok(op)
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

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
pub mod tests {
    use super::*;
    use crate::test_data;

    pub fn read_abc_from_file(path: &str) -> Vec<u8> {
        use crate::types::Tag;
        let data = std::fs::read(path).unwrap();
        let swf_buf = crate::decompress_swf(&data[..]).unwrap();
        let swf = crate::parse_swf(&swf_buf).unwrap();
        for tag in swf.tags {
            if let Tag::DoAbc2(do_abc) = tag {
                return do_abc.data.to_vec();
            }
        }
        panic!("ABC tag not found in {path}");
    }

    #[test]
    fn read_abc() {
        for (_, abc_file, bytes) in test_data::avm2_tests() {
            let mut reader = Reader::new(&bytes[..]);
            let parsed = reader.read().unwrap();
            assert_eq!(
                parsed, abc_file,
                "Incorrectly parsed ABC.\nRead:\n{parsed:?}\n\nExpected:\n{abc_file:?}",
            );
        }
    }

    #[test]
    fn test_round_trip_default_value() {
        use crate::avm2::write::Writer;

        let orig_bytes = read_abc_from_file("tests/swfs/Avm2DefaultValue.swf");
        let mut reader = Reader::new(&orig_bytes[..]);
        let parsed = reader.read().unwrap();

        let mut out = vec![];
        let mut writer = Writer::new(&mut out);
        writer.write(parsed).unwrap();

        assert_eq!(
            orig_bytes, out,
            "Incorrectly written Avm2DefaultValue class"
        );
    }

    #[test]
    fn read_u30() {
        let read = |data: &[u8]| Reader::new(data).read_u30().unwrap();
        assert_eq!(read(&[0]), 0);
        assert_eq!(read(&[2]), 2);
        assert_eq!(read(&[0b1_0000001, 0b0_0000001]), 129);
        assert_eq!(
            read(&[0b1_0000001, 0b1_0000001, 0b0_1100111]),
            0b1100111_0000001_0000001
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b0000_1111
            ]),
            0b1111_0000000_0000000_0000000_0000000
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1111_1111
            ]),
            0b1111_0000000_0000000_0000000_0000000
        );
    }

    #[test]
    fn read_i24() {
        let read = |data: &[u8]| Reader::new(data).read_i24().unwrap();
        assert_eq!(read(&[0, 0, 0]), 0);
        assert_eq!(read(&[2, 0, 0]), 2);
        assert_eq!(read(&[0b1101_0001, 0b0010_1111, 0b0000_0001]), 77777);
        assert_eq!(read(&[0b0010_1111, 0b1101_0000, 0b1111_1110]), -77777);
    }

    #[test]
    fn read_i32() {
        let read = |data: &[u8]| Reader::new(data).read_i32().unwrap();
        assert_eq!(read(&[0]), 0);
        assert_eq!(read(&[2]), 2);
        assert_eq!(read(&[0b1_0000001, 0b0_0000001]), 129);
        assert_eq!(
            read(&[
                0b1_0000001,
                0b1_0000001,
                0b1_0000001,
                0b1_0000001,
                0b0000_0100
            ]),
            1075855489
        );

        // Note that the value is NOT sign-extended, unlike what the AVM2 spec suggests.
        // Negatives must take up the full 5 bytes.
        assert_eq!(read(&[0b0_1000000]), 64);
        assert_eq!(read(&[0b1_0000000, 0b0_1000000]), 8192);
        assert_eq!(read(&[0b1_0000000, 0b1_0000000, 0b0_1000000]), 1048576);
        assert_eq!(
            read(&[0b1_0000000, 0b1_0000000, 0b1_0000000, 0b0_1000000]),
            134217728
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b0000_0100
            ]),
            1073741824
        );
        assert_eq!(
            read(&[
                0b1_1111111,
                0b1_1111111,
                0b1_1111111,
                0b1_1111111,
                0b0000_0111
            ]),
            2147483647
        );

        assert_eq!(
            read(&[
                0b1_1000000,
                0b1_1111111,
                0b1_1111111,
                0b1_1111111,
                0b0000_1111,
            ]),
            -64
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_1000000,
                0b1_1111111,
                0b1_1111111,
                0b0000_1111
            ]),
            -8192
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_1000000,
                0b1_1111111,
                0b0000_1111
            ]),
            -1048576
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_1000000,
                0b0000_1111
            ]),
            -134217728
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b0000_1000
            ]),
            -2147483648
        );

        assert_eq!(read(&[0b1_0000100, 0b1_0000111, 0b0_0000100,]), 66436);

        assert_eq!(
            read(&[0b1_0000100, 0b1_0000111, 0b1_0000000, 0b0_1111111,]),
            266339204
        );

        // Final 4 bytes of a 5-byte value are unimportant.
        assert_eq!(
            read(&[
                0b1_0000100,
                0b1_0000100,
                0b1_0000100,
                0b1_0000100,
                0b1111_0111
            ]),
            1887502852
        );
    }
}
