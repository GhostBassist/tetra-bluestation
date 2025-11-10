use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_ul::CmcePduTypeUl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the U-TX DEMAND PDU (Clause 14.7.2.12).
/// This PDU shall be the message to the SwMI that a transmission is requested.
/// Response expected: -
/// Response to: D-TX GRANTED

// note 1: This information element is not used in this version of the present document and its value shall be set to "0".
#[derive(Debug)]
pub struct UTxDemand {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 2 bits, TX demand priority
    pub tx_demand_priority: u8,
    /// Type1, 1 bits, Encryption control
    pub encryption_control: bool,
    /// Type1, 1 bits, See note,
    pub reserved: bool,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, DM-MS address
    pub dm_ms_address: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl UTxDemand {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeUl::UTxDemand)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let tx_demand_priority = buffer.read_field(2, "tx_demand_priority")? as u8;
        // Type1
        let encryption_control = buffer.read_field(1, "encryption_control")? != 0;
        // Type1
        let reserved = buffer.read_field(1, "reserved")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;


        // Type3
        let facility = if obit { 
        CmceType3Field::parse(buffer, "facility")? as Option<CmceType3Field>
        } else { None };
        
        // Type3
        let dm_ms_address = if obit { 
        CmceType3Field::parse(buffer, "dm_ms_address")? as Option<CmceType3Field>
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

        Ok(UTxDemand { 
            call_identifier, 
            tx_demand_priority, 
            encryption_control, 
            reserved, 
            facility, 
            dm_ms_address, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeUl::UTxDemand.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.tx_demand_priority as u64, 2);
        // Type1
        buffer.write_bits(self.encryption_control as u64, 1);
        // Type1
        buffer.write_bits(self.reserved as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.facility.is_some() || self.dm_ms_address.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type3
        if let Some(ref value) = self.facility {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.dm_ms_address {
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

impl fmt::Display for UTxDemand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UTxDemand {{ call_identifier: {:?} tx_demand_priority: {:?} encryption_control: {:?} reserved: {:?} facility: {:?} dm_ms_address: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.tx_demand_priority,
            self.encryption_control,
            self.reserved,
            self.facility,
            self.dm_ms_address,
            self.proprietary,
        )
    }
}
