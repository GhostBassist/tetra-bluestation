use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the D-SETUP PDU (Clause 14.7.1.12).
/// This PDU shall be the call set-up message sent to the called MS.
/// Response expected: U-ALERT/U-CONNECT/-
/// Response to: -

// note 1: This information element is used by SS-PC, refer to ETSI EN 300 392-12-10 [15] and SS-PPC and ETSI EN 300 392-12-16 [16].
// note 2: For resolution of possible Facility (Talking Party Identifier)/Calling party identifier conflicts, refer to ETSI EN 300 392-12-3 [12], clause 5.2.1.5 and ETSI EN 300 392-12-1 [11], clause 4.3.5.
// note 3: Shall be conditional on the value of Calling Party Type Identifier (CPTI): • CPTI = 1 ⇒ Calling Party SSI; • CPTI = 2 ⇒ Calling Party SSI + Calling Party Extension.
#[derive(Debug)]
pub struct DSetup {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 4 bits, Call time-out
    pub call_time_out: u8,
    /// Type1, 1 bits, Hook method selection
    pub hook_method_selection: bool,
    /// Type1, 1 bits, Simplex/duplex selection
    pub simplex_duplex_selection: bool,
    /// Type1, 8 bits, Basic service information
    pub basic_service_information: u8,
    /// Type1, 2 bits, Transmission grant
    pub transmission_grant: u8,
    /// Type1, 1 bits, Transmission request permission
    pub transmission_request_permission: bool,
    /// Type1, 4 bits, See note 1,
    pub call_priority: u8,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type2, 24 bits, Temporary address
    pub temporary_address: Option<u64>,
    /// Type2, 2 bits, See note 2,
    pub calling_party_type_identifier: Option<u64>,
    /// Conditional 24 bits, See note 3, condition: calling_party_type_identifier == Some(1) || calling_party_type_identifier == Some(2)
    pub calling_party_address_ssi: Option<u64>,
    /// Conditional 24 bits, See note 3, condition: calling_party_type_identifier == Some(2)
    pub calling_party_extension: Option<u64>,
    /// Type3, External subscriber number
    pub external_subscriber_number: Option<CmceType3Field>,
    /// Type3, Facility
    pub facility: Option<CmceType3Field>,
    /// Type3, DM-MS address
    pub dm_ms_address: Option<CmceType3Field>,
    /// Type3, Proprietary
    pub proprietary: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DSetup {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DSetup)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let call_time_out = buffer.read_field(4, "call_time_out")? as u8;
        // Type1
        let hook_method_selection = buffer.read_field(1, "hook_method_selection")? != 0;
        // Type1
        let simplex_duplex_selection = buffer.read_field(1, "simplex_duplex_selection")? != 0;
        // Type1
        let basic_service_information = buffer.read_field(8, "basic_service_information")? as u8;
        // Type1
        let transmission_grant = buffer.read_field(2, "transmission_grant")? as u8;
        // Type1
        let transmission_request_permission = buffer.read_field(1, "transmission_request_permission")? != 0;
        // Type1
        let call_priority = buffer.read_field(4, "call_priority")? as u8;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let notification_indicator = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "notification_indicator")? as Option<u64>
        } else { None };
        // Type2
        let temporary_address = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "temporary_address")? as Option<u64>
        } else { None };
        // Type2
        let calling_party_type_identifier = if obit { 
            typed_pdu_fields::type2::parse(buffer, 2, "calling_party_type_identifier")? as Option<u64>
        } else { None };
        // Conditional
        let calling_party_address_ssi = if obit && calling_party_type_identifier == Some(1) || calling_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "calling_party_address_ssi")?) 
        } else { None };
        // Conditional
        let calling_party_extension = if obit && calling_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "calling_party_extension")?) 
        } else { None };


        // Type3
        let external_subscriber_number = if obit { 
            CmceType3Field::parse(buffer, "external_subscriber_number")? as Option<CmceType3Field>
        } else { None };
        
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

        Ok(DSetup { 
            call_identifier, 
            call_time_out, 
            hook_method_selection, 
            simplex_duplex_selection, 
            basic_service_information, 
            transmission_grant, 
            transmission_request_permission, 
            call_priority, 
            notification_indicator, 
            temporary_address, 
            calling_party_type_identifier, 
            calling_party_address_ssi, 
            calling_party_extension, 
            external_subscriber_number, 
            facility, 
            dm_ms_address, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DSetup.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.call_time_out as u64, 4);
        // Type1
        buffer.write_bits(self.hook_method_selection as u64, 1);
        // Type1
        buffer.write_bits(self.simplex_duplex_selection as u64, 1);
        // Type1
        buffer.write_bits(self.basic_service_information as u64, 8);
        // Type1
        buffer.write_bits(self.transmission_grant as u64, 2);
        // Type1
        buffer.write_bits(self.transmission_request_permission as u64, 1);
        // Type1
        buffer.write_bits(self.call_priority as u64, 4);

        // Check if any optional field present and place o-bit
        let obit_val = self.notification_indicator.is_some() || self.temporary_address.is_some() || self.calling_party_type_identifier.is_some() || self.external_subscriber_number.is_some() || self.facility.is_some() || self.dm_ms_address.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.notification_indicator, 6);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.temporary_address, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.calling_party_type_identifier, 2);

        // Conditional
        if let Some(ref value) = self.calling_party_address_ssi {
            buffer.write_bits(*value, 24);
        }
        // Conditional
        if let Some(ref value) = self.calling_party_extension {
            buffer.write_bits(*value, 24);
        }
        // Type3
        if let Some(ref value) = self.external_subscriber_number {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
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

impl fmt::Display for DSetup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DSetup {{ call_identifier: {:?} call_time_out: {:?} hook_method_selection: {:?} simplex_duplex_selection: {:?} basic_service_information: {:?} transmission_grant: {:?} transmission_request_permission: {:?} call_priority: {:?} notification_indicator: {:?} temporary_address: {:?} calling_party_type_identifier: {:?} calling_party_address_ssi: {:?} calling_party_extension: {:?} external_subscriber_number: {:?} facility: {:?} dm_ms_address: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.call_time_out,
            self.hook_method_selection,
            self.simplex_duplex_selection,
            self.basic_service_information,
            self.transmission_grant,
            self.transmission_request_permission,
            self.call_priority,
            self.notification_indicator,
            self.temporary_address,
            self.calling_party_type_identifier,
            self.calling_party_address_ssi,
            self.calling_party_extension,
            self.external_subscriber_number,
            self.facility,
            self.dm_ms_address,
            self.proprietary,
        )
    }
}
