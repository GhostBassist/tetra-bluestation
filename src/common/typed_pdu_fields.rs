/// Helper functions for dealing with type2, type3 and type4 fields for MLE, CMCE, MM and SNDCP PDUs.

pub mod delimiters {
    use crate::common::{bitbuffer::BitBuffer, pdu_parse_error::PduParseError};

    /// Read the o-bit between type1 and type2/type3 elements
    pub fn read_obit(buffer: &mut BitBuffer) -> Result<bool, PduParseError> {
        Ok(buffer.read_field(1, "obit")? == 1)
    }

    /// Write the o-bit between type1 and type2/type3 elements
    pub fn write_obit(buffer: &mut BitBuffer, val: u8) {
        buffer.write_bit(val);
    }

    /// Read a p-bit preceding a type2 element
    pub fn read_pbit(buffer: &mut BitBuffer) -> Result<bool, PduParseError>{
        Ok(buffer.read_field(1, "pbit")? == 1)
    }

    /// Write the p-bit preceding a type2 element
    pub fn write_pbit(buffer: &mut BitBuffer, val: u8) {
        buffer.write_bit(val);
    }

    /// Read an m-bit found before a type3 or type4 element, and trailing the message
    pub fn read_mbit(buffer: &mut BitBuffer) -> Result<bool, PduParseError>{
        Ok(buffer.read_field(1, "mbit")? == 1)
    }

    /// Write the m-bit before a type3 or type4 element, and trailing the message
    pub fn write_mbit(buffer: &mut BitBuffer, val: u8) {
        buffer.write_bit(val);
    }
}

pub mod type2 {
    use crate::common::{bitbuffer::BitBuffer, pdu_parse_error::PduParseError};

    use super::delimiters;

    pub fn parse(buffer: &mut BitBuffer, num_bits: usize, field_name: &'static str) -> Result<Option<u64>, PduParseError> {
        match delimiters::read_pbit(buffer) {
            Ok(true) => {
                match buffer.read_field(num_bits, field_name) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(e),
                }
            },
            Ok(false) => Ok(None), // Field not present
            Err(e) => Err(e),
        }
    }

    /// Write one Type-2 element.
    /// If `value` is `Some(v)`, writes P-bit=1 then `len` bits of `v`. If `None`, writes P-bit=0.
    pub fn write(buffer: &mut BitBuffer, value: Option<u64>, len: usize) {
        match value {
            Some(v) => {
                delimiters::write_pbit(buffer, 1);
                buffer.write_bits(v, len);
            }
            None => {
                delimiters::write_pbit(buffer, 0);
            }
        }
    }
}

pub mod type34 {
    use crate::common::{bitbuffer::BitBuffer, typed_pdu_fields::delimiters::write_mbit};

    #[derive(Debug, PartialEq, Eq)]
    pub enum Type34Err {
        FieldNotPresent,
        InvalidId,
        OutOfBounds,
    }

    /// Read the m-bit for a type3 or type4 element without advancing the buffer pos
    pub fn check_peek_mbit(buffer: &BitBuffer) -> Result<bool, Type34Err> {
        match buffer.peek_bits(1) {
            Some(0) => Err(Type34Err::FieldNotPresent),
            Some(1) => Ok(true), // Field is present
            None => Err(Type34Err::OutOfBounds),
            _ => panic!() // Never happens
        }
    }

    pub fn check_peek_id(buffer: &BitBuffer, expected_id: u64) -> Result<(), Type34Err> {
        let id_bits = match buffer.peek_bits_posoffset(1, 4) {
            Some(x) => x,
            None => return Err(Type34Err::OutOfBounds),
        };

        if id_bits == expected_id {
            Ok(())
        } else {
            Err(Type34Err::FieldNotPresent)
        }
    }

    pub fn parse_type3_generic(buffer: &mut BitBuffer, expected_id: u64) -> Result<(usize, u64), Type34Err> { 

        // Check that more elements are present. Returns FieldNotPresent if mbit is 0
        check_peek_mbit(buffer)?;

        // Check that next element is our searched id
        check_peek_id(buffer, expected_id)?;

        // Target field is present. Advance buffer position and read field contents
        buffer.seek_rel(5);
        let len_bits = match buffer.read_bits(11) {
            Some(x) => x as usize,
            None => return Err(Type34Err::OutOfBounds),
        };
        let data = match buffer.read_bits(len_bits) {
            Some(x) => x,
            None => return Err(Type34Err::OutOfBounds),
        };
        Ok((len_bits, data))
    }


    pub fn parse_type4_header_generic(buffer: &mut BitBuffer, expected_id: u64) -> Result<(usize, usize), Type34Err> { 
        // Check that more elements are present. Returns FieldNotPresent if mbit is 0
        check_peek_mbit(buffer)?;

        // Check that next element is our searched id
        check_peek_id(buffer, expected_id)?;

        // Target field is present. Advance buffer position and read field contents
        buffer.seek_rel(5);
        let len_bits = match buffer.read_bits(11) {
            Some(x) => x as usize,
            None => return Err(Type34Err::OutOfBounds),
        };
        // tracing::debug!("MmType4FieldUl: len_bits: {}", len_bits);
        let num_elems = match buffer.read_bits(6) {
            Some(x) => x as usize,
            None => return Err(Type34Err::OutOfBounds),
        };

        Ok((num_elems, len_bits-6))
    }

    
    pub fn write_type4_header_generic(buffer: &mut BitBuffer, field_type: u64) {
        // mbit + id
        write_mbit(buffer, 1);
        buffer.write_bits(field_type, 4);
    }
}