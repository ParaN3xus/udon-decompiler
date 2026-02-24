use crate::odin::model::{OdinError, OdinGuid, OdinString, Result, StringEncoding};

pub(crate) struct ByteReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) fn has_remaining(&self) -> bool {
        self.offset < self.data.len()
    }

    pub(crate) fn read_exact(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.offset + count > self.data.len() {
            return Err(OdinError::new(format!(
                "Unexpected end of stream at offset {}, need {} more bytes.",
                self.offset,
                self.offset + count - self.data.len()
            )));
        }
        let start = self.offset;
        self.offset += count;
        Ok(&self.data[start..self.offset])
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    pub(crate) fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub(crate) fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.read_exact(2)?;
        Ok(i16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_exact(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_exact(8)?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub(crate) fn read_f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub(crate) fn read_decimal_bits(&mut self) -> Result<[u32; 4]> {
        Ok([
            self.read_i32()? as u32,
            self.read_i32()? as u32,
            self.read_i32()? as u32,
            self.read_i32()? as u32,
        ])
    }

    pub(crate) fn read_guid(&mut self) -> Result<OdinGuid> {
        let bytes = self.read_exact(16)?;
        let mut value = [0_u8; 16];
        value.copy_from_slice(bytes);
        Ok(OdinGuid(value))
    }

    pub(crate) fn read_string_value(&mut self) -> Result<OdinString> {
        let start_offset = self.offset;
        let char_size_flag = self.read_u8()?;
        let length = self.read_i32()?;
        if length < 0 {
            return Err(OdinError::new(format!(
                "Negative string length {} at offset {}.",
                length, start_offset
            )));
        }
        let length = length as usize;
        match char_size_flag {
            0 => {
                let bytes = self.read_exact(length)?;
                let value = bytes.iter().map(|b| char::from(*b)).collect::<String>();
                Ok(OdinString {
                    value,
                    encoding: StringEncoding::SingleByte,
                })
            }
            1 => {
                let byte_len = length.checked_mul(2).ok_or_else(|| {
                    OdinError::new(format!(
                        "String byte length overflow at offset {}.",
                        start_offset
                    ))
                })?;
                let bytes = self.read_exact(byte_len)?;
                let mut units = Vec::with_capacity(length);
                for chunk in bytes.chunks_exact(2) {
                    units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                }
                let value = String::from_utf16_lossy(&units);
                Ok(OdinString {
                    value,
                    encoding: StringEncoding::Utf16,
                })
            }
            _ => Err(OdinError::new(format!(
                "Unsupported string char-size flag {} at offset {}.",
                char_size_flag, start_offset
            ))),
        }
    }
}

pub(crate) struct ByteWriter {
    data: Vec<u8>,
}

impl ByteWriter {
    pub(crate) fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub(crate) fn write_raw(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub(crate) fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub(crate) fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub(crate) fn write_i16(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_i64(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub(crate) fn write_f32(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }

    pub(crate) fn write_f64(&mut self, value: f64) {
        self.write_u64(value.to_bits());
    }

    pub(crate) fn write_decimal_bits(&mut self, bits: [u32; 4]) {
        self.write_u32(bits[0]);
        self.write_u32(bits[1]);
        self.write_u32(bits[2]);
        self.write_u32(bits[3]);
    }

    pub(crate) fn write_guid(&mut self, guid: OdinGuid) {
        self.data.extend_from_slice(&guid.0);
    }

    pub(crate) fn write_string_value(&mut self, value: &OdinString) -> Result<()> {
        match value.encoding {
            StringEncoding::SingleByte => {
                self.write_u8(0);
                let mut bytes = Vec::with_capacity(value.value.chars().count());
                for ch in value.value.chars() {
                    let code = ch as u32;
                    if code > 0xFF {
                        return Err(OdinError::new(format!(
                            "Cannot encode U+{:04X} in single-byte Odin string.",
                            code
                        )));
                    }
                    bytes.push(code as u8);
                }
                let len = i32::try_from(bytes.len())
                    .map_err(|_| OdinError::new("Single-byte Odin string length overflow."))?;
                self.write_i32(len);
                self.write_raw(&bytes);
            }
            StringEncoding::Utf16 => {
                self.write_u8(1);
                let units = value.value.encode_utf16().collect::<Vec<_>>();
                let len = i32::try_from(units.len())
                    .map_err(|_| OdinError::new("UTF-16 Odin string length overflow."))?;
                self.write_i32(len);
                for unit in units {
                    self.write_u16(unit);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.data
    }
}
