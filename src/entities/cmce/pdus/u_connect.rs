use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_ul::CmcePduTypeUl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the U-CONNECT PDU (Clause 14.7.2.3).
/// This PDU shall be the acknowledgement to the SwMI that the called MS is ready for through-connection.
/// Response expected: D-CONNECT ACKNOWLEDGE
/// Response to: D-SETUP

#[derive(Debug)]
pub struct UConnect {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 1 bits, Hook method selection
    pub hook_method_selection: bool,
    /// Type1, 1 bits, Simplex/duplex selection
    pub simplex_duplex_selection: bool,
    /// Type2, 8 bits, Basic service information
    pub basic_service_information: Option<u64>,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl UConnect {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeUl::UConnect)?;
        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let hook_method_selection = buffer.read_field(1, "hook_method_selection")? != 0;
        // Type1
        let simplex_duplex_selection = buffer.read_field(1, "simplex_duplex_selection")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let basic_service_information = if obit { 
            typed_pdu_fields::type2::parse(buffer, 8, "basic_service_information")? as Option<u64>
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

        Ok(UConnect { 
            call_identifier, 
            hook_method_selection, 
            simplex_duplex_selection, 
            basic_service_information, 
            facility, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeUl::UConnect.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.hook_method_selection as u64, 1);
        // Type1
        buffer.write_bits(self.simplex_duplex_selection as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.basic_service_information.is_some() || self.facility.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.basic_service_information, 8);

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

impl fmt::Display for UConnect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UConnect {{ call_identifier: {:?} hook_method_selection: {:?} simplex_duplex_selection: {:?} basic_service_information: {:?} facility: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.hook_method_selection,
            self.simplex_duplex_selection,
            self.basic_service_information,
            self.facility,
            self.proprietary,
        )
    }
}
