use scroll::{ctx, Pread, Uleb128};

use crate::cache::Ref;
use crate::class::ClassId;
use crate::encoded_item::EncodedItem;
use crate::encoded_item::EncodedItemArray;
use crate::jtype::Type;
use crate::string::JString;
use crate::uint;
use crate::ulong;
use crate::ushort;

#[derive(Debug)]
pub struct Field {
    name: Ref<JString>,
    jtype: Type,
    class: ClassId,
    access_flags: ulong,
}

impl Field {
    pub(crate) fn try_from_dex<S: AsRef<[u8]>>(
        dex: &super::Dex<S>,
        encoded_field: &EncodedField,
    ) -> super::Result<Self> {
        let field_item = dex.get_field_item(encoded_field.field_id)?;
        Ok(Self {
            name: dex.get_string(field_item.name_idx)?,
            jtype: dex.get_type(uint::from(field_item.type_idx))?,
            class: uint::from(field_item.class_idx),
            access_flags: encoded_field.access_flags,
        })
    }
}

pub(crate) type EncodedFieldArray = EncodedItemArray<EncodedField>;

#[derive(Pread, Debug)]
pub(crate) struct FieldIdItem {
    class_idx: ushort,
    type_idx: ushort,
    name_idx: crate::string::StringId,
}

impl FieldIdItem {
    pub(crate) fn try_from_dex<T: AsRef<[u8]>>(
        dex: &super::Dex<T>,
        offset: ulong,
    ) -> super::Result<Self> {
        let source = dex.source.as_ref();
        Ok(source.pread_with(offset as usize, dex.get_endian())?)
    }
}

pub type FieldId = ulong;

pub(crate) struct EncodedField {
    pub(crate) field_id: FieldId,
    access_flags: ulong,
}

impl EncodedItem for EncodedField {
    fn get_id(&self) -> ulong {
        self.field_id
    }
}

impl<'a> ctx::TryFromCtx<'a, ulong> for EncodedField {
    type Error = crate::error::Error;
    type Size = usize;

    fn try_from_ctx(source: &'a [u8], prev_id: ulong) -> super::Result<(Self, Self::Size)> {
        let offset = &mut 0;
        let id = Uleb128::read(source, offset)?;
        let access_flags = Uleb128::read(source, offset)?;
        Ok((
            Self {
                field_id: prev_id + id,
                access_flags,
            },
            *offset,
        ))
    }
}