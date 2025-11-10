use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the D-CALL RESTORE PDU (Clause 14.7.1.3).
/// This PDU shall indicate to the MS that a call has been restored after a temporary break of the call.
/// Response expected: -
/// Response to: U-CALL RESTORE

#[derive(Debug)]
pub struct DCallRestore {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 2 bits, Transmission grant
    pub transmission_grant: u8,
    /// Type1, 1 bits, Transmission request permission
    pub transmission_request_permission: bool,
    /// Type1, 1 bits, Reset call time-out timer (T310)
    pub reset_call_time_out_timer_t310_: bool,
    /// Type2, 14 bits, New call identifier
    pub new_call_identifier: Option<u64>,
    /// Type2, 4 bits, Call time-out
    pub call_time_out: Option<u64>,
    /// Type2, 3 bits, Call status
    pub call_status: Option<u64>,
    /// Type2, 9 bits, Modify
    pub modify: Option<u64>,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, Temporary address
    pub temporary_address: Option<CmceType3Field>,
    /// Type3, DM-MS address
    pub dm_ms_address: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DCallRestore {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DCallRestore)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let transmission_grant = buffer.read_field(2, "transmission_grant")? as u8;
        // Type1
        let transmission_request_permission = buffer.read_field(1, "transmission_request_permission")? != 0;
        // Type1
        let reset_call_time_out_timer_t310_ = buffer.read_field(1, "reset_call_time_out_timer_t310_")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let new_call_identifier = if obit { 
            typed_pdu_fields::type2::parse(buffer, 14, "new_call_identifier")? as Option<u64>
        } else { None };
        // Type2
        let call_time_out = if obit { 
            typed_pdu_fields::type2::parse(buffer, 4, "call_time_out")? as Option<u64>
        } else { None };
        // Type2
        let call_status = if obit { 
            typed_pdu_fields::type2::parse(buffer, 3, "call_status")? as Option<u64>
        } else { None };
        // Type2
        let modify = if obit { 
            typed_pdu_fields::type2::parse(buffer, 9, "modify")? as Option<u64>
        } else { None };
        // Type2
        let notification_indicator = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "notification_indicator")? as Option<u64>
        } else { None };


        // Type3
        let facility = if obit { 
            CmceType3Field::parse(buffer, "facility")? as Option<CmceType3Field>
        } else { None };
        // Type3
        let temporary_address = if obit { 
            CmceType3Field::parse(buffer, "temporary_address")? as Option<CmceType3Field>
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

        Ok(DCallRestore { 
            call_identifier, 
            transmission_grant, 
            transmission_request_permission, 
            reset_call_time_out_timer_t310_, 
            new_call_identifier, 
            call_time_out, 
            call_status, 
            modify, 
            notification_indicator, 
            facility, 
            temporary_address, 
            dm_ms_address, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DCallRestore.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.transmission_grant as u64, 2);
        // Type1
        buffer.write_bits(self.transmission_request_permission as u64, 1);
        // Type1
        buffer.write_bits(self.reset_call_time_out_timer_t310_ as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.new_call_identifier.is_some() || self.call_time_out.is_some() || self.call_status.is_some() || self.modify.is_some() || self.notification_indicator.is_some() || self.facility.is_some() || self.temporary_address.is_some() || self.dm_ms_address.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.new_call_identifier, 14);
        
        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_time_out, 4);
        
        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_status, 3);
        
        // Type2
        typed_pdu_fields::type2::write(buffer, self.modify, 9);
        
        // Type2
        typed_pdu_fields::type2::write(buffer, self.notification_indicator, 6);
        
        // Type3
        if let Some(ref value) = self.facility {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.temporary_address {
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

impl fmt::Display for DCallRestore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DCallRestore {{ call_identifier: {:?} transmission_grant: {:?} transmission_request_permission: {:?} reset_call_time_out_timer_t310_: {:?} new_call_identifier: {:?} call_time_out: {:?} call_status: {:?} modify: {:?} notification_indicator: {:?} facility: {:?} temporary_address: {:?} dm_ms_address: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.transmission_grant,
            self.transmission_request_permission,
            self.reset_call_time_out_timer_t310_,
            self.new_call_identifier,
            self.call_time_out,
            self.call_status,
            self.modify,
            self.notification_indicator,
            self.facility,
            self.temporary_address,
            self.dm_ms_address,
            self.proprietary,
        )
    }
}
