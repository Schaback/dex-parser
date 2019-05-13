use scroll::ctx;
use scroll::Pread;
use scroll::Uleb128;

use crate::cache::Ref;
use crate::encoded_item::EncodedItem;
use crate::encoded_item::EncodedItemArray;
use crate::jtype::Type;
use crate::jtype::TypeId;
use crate::string::JString;
use crate::string::StringId;

pub struct Method {
    class_id: Type,
    name: Ref<JString>,
    access_flags: u64,
    params: Option<Vec<Type>>,
    shorty: Ref<JString>,
    return_type: Type,
    code_off: u64,
}

pub type ProtoId = u64;

#[derive(Pread)]
pub(crate) struct ProtoIdItem {
    shorty: StringId,
    return_type: TypeId,
    params_off: u32,
}

impl ProtoIdItem {
    pub(crate) fn from_dex<S: AsRef<[u8]>>(
        dex: &super::Dex<S>,
        offset: u64,
    ) -> super::Result<Self> {
        let source = dex.source.as_ref().as_ref();
        Ok(source.pread_with(offset as usize, dex.get_endian())?)
    }
}

impl Method {
    pub(crate) fn from_dex<S: AsRef<[u8]>>(
        dex: &super::Dex<S>,
        encoded_method: &EncodedMethod,
    ) -> super::Result<Method> {
        let source = dex.source.as_ref().as_ref();
        let method_item = dex.get_method_item(encoded_method.method_id)?;
        let proto_item = dex.get_proto_item(method_item.proto_id)?;
        let shorty = dex.get_string(proto_item.shorty)?;
        let return_type = dex.get_type(proto_item.return_type)?;
        let params = if proto_item.params_off != 0 {
            let mut offset = proto_item.params_off as usize;
            let offset = &mut offset;
            let endian = dex.get_endian();
            let len = source.gread_with::<u32>(offset, endian)?;
            let mut types: Vec<u16> = Vec::with_capacity(len as usize);
            source.gread_inout_with(offset, &mut types, endian)?;
            Some(
                types
                    .into_iter()
                    .map(|s| dex.get_type(s as u32))
                    .collect::<super::Result<Vec<Type>>>()?,
            )
        } else {
            None
        };
        Ok(Self {
            name: dex.get_string(method_item.name_id)?,
            class_id: dex.get_type(method_item.class_id)?,
            access_flags: encoded_method.access_flags,
            shorty,
            return_type,
            params,
            code_off: encoded_method.code_offset,
        })
    }
}

#[derive(Pread)]
pub(crate) struct MethodIdItem {
    class_id: TypeId,
    proto_id: ProtoId,
    name_id: StringId,
}

impl MethodIdItem {
    pub(crate) fn from_dex<S: AsRef<[u8]>>(
        dex: &super::Dex<S>,
        offset: u64,
    ) -> super::Result<Self> {
        let source = dex.source.as_ref().as_ref();
        Ok(source.pread_with(offset as usize, dex.get_endian())?)
    }
}

pub type MethodId = u64;

pub(crate) struct EncodedMethod {
    pub(crate) method_id: MethodId,
    access_flags: u64,
    code_offset: u64,
}

impl EncodedItem for EncodedMethod {
    fn get_id(&self) -> u64 {
        self.method_id
    }
}

pub(crate) type EncodedMethodArray = EncodedItemArray<EncodedMethod>;

impl<'a> ctx::TryFromCtx<'a, u64> for EncodedMethod {
    type Error = crate::error::Error;
    type Size = usize;

    fn try_from_ctx(source: &'a [u8], prev_id: u64) -> super::Result<(Self, Self::Size)> {
        let offset = &mut 0;
        let id = Uleb128::read(source, offset)?;
        let access_flags = Uleb128::read(source, offset)?;
        let code_offset = Uleb128::read(source, offset)?;
        Ok((
            Self {
                method_id: prev_id + id,
                code_offset,
                access_flags,
            },
            *offset,
        ))
    }
}
