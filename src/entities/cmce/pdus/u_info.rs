use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_ul::CmcePduTypeUl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the U-INFO PDU (Clause 14.7.2.6).
/// This PDU shall be the general information message from the MS.
/// Response expected: -
/// Response to: -

// note 1: If the message is sent connectionless then the call identifier shall be equal to the dummy call identifier.
// note 2: Shall be valid for acknowledged group call only. For other types of call it shall be set equal to zero.
#[derive(Debug)]
pub struct UInfo {
    /// Type1, 14 bits, See note 1,
    pub call_identifier: u16,
    /// Type1, 1 bits, See note 2,
    pub poll_response: bool,
    /// Type2, 9 bits, Modify
    pub modify: Option<u64>,
    /// Type3, DTMF
    pub dtmf: Option<CmceType3Field>,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl UInfo {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeUl::UInfo)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let poll_response = buffer.read_field(1, "poll_response")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let modify = if obit { 
            typed_pdu_fields::type2::parse(buffer, 9, "modify")? as Option<u64>
        } else { None };


        // Type3
        let dtmf = if obit { 
        CmceType3Field::parse(buffer, "dtmf")? as Option<CmceType3Field>
        } else { None };
        
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

        Ok(UInfo { 
            call_identifier, 
            poll_response, 
            modify, 
            dtmf, 
            facility, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeUl::UInfo.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.poll_response as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.modify.is_some() || self.dtmf.is_some() || self.facility.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.modify, 9);

        // Type3
        if let Some(ref value) = self.dtmf {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
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

impl fmt::Display for UInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UInfo {{ call_identifier: {:?} poll_response: {:?} modify: {:?} dtmf: {:?} facility: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.poll_response,
            self.modify,
            self.dtmf,
            self.facility,
            self.proprietary,
        )
    }
}
