use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the D-TX GRANTED PDU (Clause 14.7.1.15).
/// This PDU shall inform the MS concerned with a call that permission to transmit has been granted by the SwMI to a MS, and to inform that MS that it has been granted permission to transmit. This PDU shall also inform a MS that its request to transmit has been rejected or queued.
/// Response expected: -
/// Response to: U-TX DEMAND

// note 1: This information element is not used in this version of the present document and its value shall be set to "0."
// note 2: Shall be conditional on the value of Transmitting Party Type Identifier (TPTI): TPTI = 1 ⇒ Transmitting Party SSI; TPTI = 2 ⇒ Transmitting Party SSI + Transmitting Party Extension.
#[derive(Debug)]
pub struct DTxGranted {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 2 bits, Transmission grant
    pub transmission_grant: u8,
    /// Type1, 1 bits, Transmission request permission
    pub transmission_request_permission: bool,
    /// Type1, 1 bits, Encryption control
    pub encryption_control: bool,
    /// Type1, 1 bits, See note 1,
    pub reserved: bool,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type2, 2 bits, Transmitting party type identifier
    pub transmitting_party_type_identifier: Option<u64>,
    /// Conditional 24 bits, See note 2, condition: transmitting_party_type_identifier == Some(1) || transmitting_party_type_identifier == Some(2)
    pub transmitting_party_address_ssi: Option<u64>,
    /// Conditional 24 bits, See note 2, condition: transmitting_party_type_identifier == Some(2)
    pub transmitting_party_extension: Option<u64>,
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
impl DTxGranted {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {
        
        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DTxGranted)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let transmission_grant = buffer.read_field(2, "transmission_grant")? as u8;
        // Type1
        let transmission_request_permission = buffer.read_field(1, "transmission_request_permission")? != 0;
        // Type1
        let encryption_control = buffer.read_field(1, "encryption_control")? != 0;
        // Type1
        let reserved = buffer.read_field(1, "reserved")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let notification_indicator = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "notification_indicator")? as Option<u64>
        } else { None };
        // Type2
        let transmitting_party_type_identifier = if obit { 
            typed_pdu_fields::type2::parse(buffer, 2, "transmitting_party_type_identifier")? as Option<u64>
        } else { None };
        // Conditional
        let transmitting_party_address_ssi = if obit && transmitting_party_type_identifier == Some(1) || transmitting_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "transmitting_party_address_ssi")?) 
        } else { None };
        // Conditional
        let transmitting_party_extension = if obit && transmitting_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "transmitting_party_extension")?) 
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

        Ok(DTxGranted { 
            call_identifier, 
            transmission_grant, 
            transmission_request_permission, 
            encryption_control, 
            reserved, 
            notification_indicator, 
            transmitting_party_type_identifier, 
            transmitting_party_address_ssi, 
            transmitting_party_extension, 
            external_subscriber_number, 
            facility, 
            dm_ms_address, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DTxGranted.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.transmission_grant as u64, 2);
        // Type1
        buffer.write_bits(self.transmission_request_permission as u64, 1);
        // Type1
        buffer.write_bits(self.encryption_control as u64, 1);
        // Type1
        buffer.write_bits(self.reserved as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.notification_indicator.is_some() || self.transmitting_party_type_identifier.is_some() || self.external_subscriber_number.is_some() || self.facility.is_some() || self.dm_ms_address.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.notification_indicator, 6);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.transmitting_party_type_identifier, 2);

        // Conditional
        if let Some(ref value) = self.transmitting_party_address_ssi {
            buffer.write_bits(*value, 24);
        }
        // Conditional
        if let Some(ref value) = self.transmitting_party_extension {
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

impl fmt::Display for DTxGranted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DTxGranted {{ call_identifier: {:?} transmission_grant: {:?} transmission_request_permission: {:?} encryption_control: {:?} reserved: {:?} notification_indicator: {:?} transmitting_party_type_identifier: {:?} transmitting_party_address_ssi: {:?} transmitting_party_extension: {:?} external_subscriber_number: {:?} facility: {:?} dm_ms_address: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.transmission_grant,
            self.transmission_request_permission,
            self.encryption_control,
            self.reserved,
            self.notification_indicator,
            self.transmitting_party_type_identifier,
            self.transmitting_party_address_ssi,
            self.transmitting_party_extension,
            self.external_subscriber_number,
            self.facility,
            self.dm_ms_address,
            self.proprietary,
        )
    }
}
