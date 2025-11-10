use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_ul::CmcePduTypeUl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;


/// Representation of the U-RELEASE PDU (Clause 14.7.2.9).
/// This PDU shall be the acknowledgement to a disconnection.
/// Response expected: -
/// Response to: D-DISCONNECT

#[derive(Debug)]
pub struct URelease {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 5 bits, Disconnect cause
    pub disconnect_cause: u8,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl URelease {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeUl::URelease)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let disconnect_cause = buffer.read_field(5, "disconnect_cause")? as u8;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;


        // Type3
        let facility = if obit { 
        CmceType3Field::parse(buffer, "facility")? as Option<CmceType3Field>
        } else { None };
        
        // Type3
        let proprietary = if obit { 
        CmceType3Field::parse(buffer, "proprietary")? as Option<CmceType3Field>
        } else { None };
        

        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(URelease { 
            call_identifier, 
            disconnect_cause, 
            facility, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeUl::URelease.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.disconnect_cause as u64, 5);

        // Check if any optional field present and place o-bit
        let obit_val = self.facility.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type3
        if let Some(ref value) = self.facility {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.proprietary {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for URelease {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "URelease {{ call_identifier: {:?} disconnect_cause: {:?} facility: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.disconnect_cause,
            self.facility,
            self.proprietary,
        )
    }
}
