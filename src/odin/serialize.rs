use crate::odin::io::ByteWriter;
use crate::odin::model::{
    BinaryEntryType, OdinDocument, OdinError, OdinString, OdinTypeRef, PrimitiveValue, Result,
    Token, TokenPayload, primitive_kind_for_entry_type,
};

impl OdinDocument {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = ByteWriter::new();
        for token in &self.tokens {
            if !token.dirty {
                writer.write_raw(&self.original_bytes[token.span.clone()]);
                continue;
            }
            write_token(&mut writer, token)?;
        }
        Ok(writer.into_vec())
    }
}

fn write_type_ref(writer: &mut ByteWriter, type_ref: &OdinTypeRef) -> Result<()> {
    match type_ref {
        OdinTypeRef::TypeId { id, .. } => {
            writer.write_u8(BinaryEntryType::TypeID as u8);
            writer.write_i32(*id);
        }
        OdinTypeRef::TypeName { id, name } => {
            writer.write_u8(BinaryEntryType::TypeName as u8);
            writer.write_i32(*id);
            writer.write_string_value(name)?;
        }
        OdinTypeRef::Null => {
            writer.write_u8(BinaryEntryType::UnnamedNull as u8);
        }
    }
    Ok(())
}

fn write_name_if_present(writer: &mut ByteWriter, name: &Option<OdinString>) -> Result<()> {
    if let Some(v) = name {
        writer.write_string_value(v)?;
    }
    Ok(())
}

fn write_primitive(writer: &mut ByteWriter, value: &PrimitiveValue) -> Result<()> {
    match value {
        PrimitiveValue::SByte(v) => writer.write_i8(*v),
        PrimitiveValue::Byte(v) => writer.write_u8(*v),
        PrimitiveValue::Short(v) => writer.write_i16(*v),
        PrimitiveValue::UShort(v) => writer.write_u16(*v),
        PrimitiveValue::Int(v) => writer.write_i32(*v),
        PrimitiveValue::UInt(v) => writer.write_u32(*v),
        PrimitiveValue::Long(v) => writer.write_i64(*v),
        PrimitiveValue::ULong(v) => writer.write_u64(*v),
        PrimitiveValue::Float(v) => writer.write_f32(*v),
        PrimitiveValue::Double(v) => writer.write_f64(*v),
        PrimitiveValue::Decimal(v) => writer.write_decimal_bits(*v),
        PrimitiveValue::Char(v) => writer.write_u16(*v),
        PrimitiveValue::String(v) => writer.write_string_value(v)?,
        PrimitiveValue::Guid(v) => writer.write_guid(*v),
        PrimitiveValue::Boolean(v) => writer.write_u8(if *v { 1 } else { 0 }),
    }
    Ok(())
}

fn write_token(writer: &mut ByteWriter, token: &Token) -> Result<()> {
    writer.write_u8(token.entry_type as u8);
    match (&token.entry_type, &token.payload) {
        (
            BinaryEntryType::NamedStartOfReferenceNode
            | BinaryEntryType::UnnamedStartOfReferenceNode,
            TokenPayload::StartReferenceNode {
                name,
                type_ref,
                reference_id,
            },
        ) => {
            write_name_if_present(writer, name)?;
            write_type_ref(writer, type_ref)?;
            writer.write_i32(*reference_id);
        }
        (
            BinaryEntryType::NamedStartOfStructNode | BinaryEntryType::UnnamedStartOfStructNode,
            TokenPayload::StartStructNode { name, type_ref },
        ) => {
            write_name_if_present(writer, name)?;
            write_type_ref(writer, type_ref)?;
        }
        (BinaryEntryType::StartOfArray, TokenPayload::StartArray { declared_len }) => {
            writer.write_i64(*declared_len);
        }
        (
            BinaryEntryType::PrimitiveArray,
            TokenPayload::PrimitiveArray {
                element_count,
                bytes_per_element,
                raw,
            },
        ) => {
            writer.write_i32(*element_count);
            writer.write_i32(*bytes_per_element);
            writer.write_raw(raw);
        }
        (
            BinaryEntryType::NamedInternalReference | BinaryEntryType::UnnamedInternalReference,
            TokenPayload::InternalReference { name, value },
        ) => {
            write_name_if_present(writer, name)?;
            writer.write_i32(*value);
        }
        (
            BinaryEntryType::NamedExternalReferenceByIndex
            | BinaryEntryType::UnnamedExternalReferenceByIndex,
            TokenPayload::ExternalReferenceByIndex { name, value },
        ) => {
            write_name_if_present(writer, name)?;
            writer.write_i32(*value);
        }
        (
            BinaryEntryType::NamedExternalReferenceByGuid
            | BinaryEntryType::UnnamedExternalReferenceByGuid,
            TokenPayload::ExternalReferenceByGuid { name, value },
        ) => {
            write_name_if_present(writer, name)?;
            writer.write_guid(*value);
        }
        (
            BinaryEntryType::NamedExternalReferenceByString
            | BinaryEntryType::UnnamedExternalReferenceByString,
            TokenPayload::ExternalReferenceByString { name, value },
        ) => {
            write_name_if_present(writer, name)?;
            writer.write_string_value(value)?;
        }
        (
            BinaryEntryType::NamedSByte
            | BinaryEntryType::UnnamedSByte
            | BinaryEntryType::NamedByte
            | BinaryEntryType::UnnamedByte
            | BinaryEntryType::NamedShort
            | BinaryEntryType::UnnamedShort
            | BinaryEntryType::NamedUShort
            | BinaryEntryType::UnnamedUShort
            | BinaryEntryType::NamedInt
            | BinaryEntryType::UnnamedInt
            | BinaryEntryType::NamedUInt
            | BinaryEntryType::UnnamedUInt
            | BinaryEntryType::NamedLong
            | BinaryEntryType::UnnamedLong
            | BinaryEntryType::NamedULong
            | BinaryEntryType::UnnamedULong
            | BinaryEntryType::NamedFloat
            | BinaryEntryType::UnnamedFloat
            | BinaryEntryType::NamedDouble
            | BinaryEntryType::UnnamedDouble
            | BinaryEntryType::NamedDecimal
            | BinaryEntryType::UnnamedDecimal
            | BinaryEntryType::NamedChar
            | BinaryEntryType::UnnamedChar
            | BinaryEntryType::NamedString
            | BinaryEntryType::UnnamedString
            | BinaryEntryType::NamedGuid
            | BinaryEntryType::UnnamedGuid
            | BinaryEntryType::NamedBoolean
            | BinaryEntryType::UnnamedBoolean,
            TokenPayload::Primitive { name, value },
        ) => {
            if let Some(expected) = primitive_kind_for_entry_type(token.entry_type)
                && expected != value.kind()
            {
                return Err(OdinError::new(format!(
                    "Token {:?} expects {:?} but got {:?}.",
                    token.entry_type,
                    expected,
                    value.kind()
                )));
            }
            write_name_if_present(writer, name)?;
            write_primitive(writer, value)?;
        }
        (
            BinaryEntryType::NamedNull | BinaryEntryType::UnnamedNull,
            TokenPayload::Null { name },
        ) => {
            write_name_if_present(writer, name)?;
        }
        (BinaryEntryType::TypeName, TokenPayload::TypeName { type_id, name }) => {
            writer.write_i32(*type_id);
            writer.write_string_value(name)?;
        }
        (BinaryEntryType::TypeID, TokenPayload::TypeId { type_id }) => writer.write_i32(*type_id),
        (
            BinaryEntryType::EndOfNode | BinaryEntryType::EndOfArray | BinaryEntryType::EndOfStream,
            TokenPayload::Empty,
        ) => {}
        _ => {
            return Err(OdinError::new(format!(
                "Token payload mismatch while writing {:?}.",
                token.entry_type
            )));
        }
    }
    Ok(())
}
