use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the D-INFO PDU (Clause 14.7.1.8).
/// This PDU shall be the general information message to the MS.
/// Response expected: -
/// Response to: -

// note 1: If the message is sent connectionless the call identifier shall be the dummy call identifier.
// note 2: Shall be valid for acknowledged group call only. For other types of calls it shall be set = 0.
// note 3: Shall be valid for acknowledged group call only.
#[derive(Debug)]
pub struct DInfo {
    /// Type1, 14 bits, See note 1,
    pub call_identifier: u16,
    /// Type1, 1 bits, Reset call time-out timer (T310)
    pub reset_call_time_out_timer_t310_: bool,
    /// Type1, 1 bits, See note 2,
    pub poll_request: bool,
    /// Type2, 14 bits, New call identifier
    pub new_call_identifier: Option<u64>,
    /// Type2, 4 bits, Call time-out
    pub call_time_out: Option<u64>,
    /// Type2, 3 bits, Call time-out set-up phase (T301, T302)
    pub call_time_out_set_up_phase_t301_t302_: Option<u64>,
    /// Type2, 1 bits, Call ownership
    pub call_ownership: Option<u64>,
    /// Type2, 9 bits, Modify
    pub modify: Option<u64>,
    /// Type2, 3 bits, Call status
    pub call_status: Option<u64>,
    /// Type2, 24 bits, Temporary address
    pub temporary_address: Option<u64>,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type2, 6 bits, See note 3,
    pub poll_response_percentage: Option<u64>,
    /// Type2, 6 bits, See note 3,
    pub poll_response_number: Option<u64>,
    /// Type3, DTMF
    pub dtmf: Option<CmceType3Field>,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, See note 3,
    pub poll_response_addresses: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DInfo {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DInfo)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let reset_call_time_out_timer_t310_ = buffer.read_field(1, "reset_call_time_out_timer_t310_")? != 0;
        // Type1
        let poll_request = buffer.read_field(1, "poll_request")? != 0;

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
        let call_time_out_set_up_phase_t301_t302_ = if obit { 
            typed_pdu_fields::type2::parse(buffer, 3, "call_time_out_set_up_phase_t301_t302_")? as Option<u64>
        } else { None };
        // Type2
        let call_ownership = if obit { 
            typed_pdu_fields::type2::parse(buffer, 1, "call_ownership")? as Option<u64>
        } else { None };
        // Type2
        let modify = if obit { 
            typed_pdu_fields::type2::parse(buffer, 9, "modify")? as Option<u64>
        } else { None };
        // Type2
        let call_status = if obit { 
            typed_pdu_fields::type2::parse(buffer, 3, "call_status")? as Option<u64>
        } else { None };
        // Type2
        let temporary_address = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "temporary_address")? as Option<u64>
        } else { None };
        // Type2
        let notification_indicator = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "notification_indicator")? as Option<u64>
        } else { None };
        // Type2
        let poll_response_percentage = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "poll_response_percentage")? as Option<u64>
        } else { None };
        // Type2
        let poll_response_number = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "poll_response_number")? as Option<u64>
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
        let poll_response_addresses = if obit { 
            CmceType3Field::parse(buffer, "poll_response_addresses")? as Option<CmceType3Field>
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

        Ok(DInfo { 
            call_identifier, 
            reset_call_time_out_timer_t310_, 
            poll_request, 
            new_call_identifier, 
            call_time_out, 
            call_time_out_set_up_phase_t301_t302_, 
            call_ownership, 
            modify, 
            call_status, 
            temporary_address, 
            notification_indicator, 
            poll_response_percentage, 
            poll_response_number, 
            dtmf, 
            facility, 
            poll_response_addresses, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DInfo.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.reset_call_time_out_timer_t310_ as u64, 1);
        // Type1
        buffer.write_bits(self.poll_request as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.new_call_identifier.is_some() || self.call_time_out.is_some() || self.call_time_out_set_up_phase_t301_t302_.is_some() || self.call_ownership.is_some() || self.modify.is_some() || self.call_status.is_some() || self.temporary_address.is_some() || self.notification_indicator.is_some() || self.poll_response_percentage.is_some() || self.poll_response_number.is_some() || self.dtmf.is_some() || self.facility.is_some() || self.poll_response_addresses.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.new_call_identifier, 14);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_time_out, 4);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_time_out_set_up_phase_t301_t302_, 3);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_ownership, 1);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.modify, 9);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.call_status, 3);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.temporary_address, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.notification_indicator, 6);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.poll_response_percentage, 6);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.poll_response_number, 6);

        // Type3
        if let Some(ref value) = self.dtmf {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.facility {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.poll_response_addresses {
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

impl fmt::Display for DInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DInfo {{ call_identifier: {:?} reset_call_time_out_timer_t310_: {:?} poll_request: {:?} new_call_identifier: {:?} call_time_out: {:?} call_time_out_set_up_phase_t301_t302_: {:?} call_ownership: {:?} modify: {:?} call_status: {:?} temporary_address: {:?} notification_indicator: {:?} poll_response_percentage: {:?} poll_response_number: {:?} dtmf: {:?} facility: {:?} poll_response_addresses: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.reset_call_time_out_timer_t310_,
            self.poll_request,
            self.new_call_identifier,
            self.call_time_out,
            self.call_time_out_set_up_phase_t301_t302_,
            self.call_ownership,
            self.modify,
            self.call_status,
            self.temporary_address,
            self.notification_indicator,
            self.poll_response_percentage,
            self.poll_response_number,
            self.dtmf,
            self.facility,
            self.poll_response_addresses,
            self.proprietary,
        )
    }
}
