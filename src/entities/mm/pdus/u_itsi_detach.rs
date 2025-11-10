use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_ul::MmPduTypeUl;
use crate::entities::mm::enums::type34_elem_id_ul::MmType34ElemIdUl;
use crate::entities::mm::components::type34_fields::MmType3FieldUl;

/// Representation of the U-ITSI DETACH PDU (Clause 16.9.3.3).
/// The MS sends this message to the infrastructure to announce that the MS will be de-activated.
/// Response expected: -/D-MM STATUS
/// Response to: -

#[derive(Debug)]
pub struct UItsiDetach {
    /// Type2, 24 bits, MNI of the MS (MCC followed by MNC)
    pub address_extension: Option<u64>,
    /// Type3, Proprietary
    pub proprietary: Option<MmType3FieldUl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl UItsiDetach {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeUl::UItsiDetach)?;
        
        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let address_extension = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "address_extension")? as Option<u64>
        } else { None };

        // Type3
        let proprietary = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::Proprietary) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(UItsiDetach { 
            address_extension, 
            proprietary
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(MmPduTypeUl::UItsiDetach.into_raw(), 4);

        // Check if any optional field present and place o-bit
        let obit_val = self.address_extension.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.address_extension, 24);

        // Type3
        if let Some(ref value) = self.proprietary {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for UItsiDetach {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UItsiDetach {{ address_extension: {:?} proprietary: {:?} }}",
            self.address_extension,
            self.proprietary,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::common::debug::setup_logging_default;

    use super::*;

    #[test]
    fn test_ui_itsi_detach() {

        setup_logging_default();

        let test_vec = "0001110011001100000101001110010";
        let mut buffer = BitBuffer::from_bitstr(test_vec);
        let parsed = match UItsiDetach::from_bitbuf(&mut buffer) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing UItsiDetach: {:?} {}", e, buffer.dump_bin());
                return;
            }
        };
        tracing::info!("Parsed UItsiDetach: {:?}, buf: {}", parsed, buffer.dump_bin());
    }
}