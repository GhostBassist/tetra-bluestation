use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseError;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;
use crate::entities::cmce::components::type3_fields::CmceType3Field;

/// Representation of the D-STATUS PDU (Clause 14.7.1.11).
/// This PDU shall be the PDU for receiving a pre-coded status message.
/// Response expected: None
/// Response to: None

// Note 1: Shall be conditional on the value of Calling Party Type Identifier (CPTI): CPTI = 1 → include Calling Party SSI only; CPTI = 2 → include both SSI and Calling Party Extension.
#[derive(Debug)]
pub struct DStatus {
    /// Type1, 2 bits, Calling party type identifier
    pub calling_party_type_identifier: u8,
    /// Conditional 24 bits, Calling party address SSI condition: calling_party_type_identifier == 1 || calling_party_type_identifier == 2
    pub calling_party_address_ssi: Option<u64>,
    /// Conditional 24 bits, Calling party extension condition: calling_party_type_identifier == 2
    pub calling_party_extension: Option<u64>,
    /// Type1, 16 bits, Pre-coded status
    pub pre_coded_status: u16,
    /// Type3, External subscriber number
    pub external_subscriber_number: Option<CmceType3Field>,
    /// Type3, DM-MS address
    pub dm_ms_address: Option<CmceType3Field>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DStatus {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DStatus)?;

        // Type1
        let calling_party_type_identifier = buffer.read_field(2, "calling_party_type_identifier")? as u8;
        // Conditional
        let calling_party_address_ssi = if calling_party_type_identifier == 1 || calling_party_type_identifier == 2 { 
            Some(buffer.read_field(24, "calling_party_address_ssi")?) 
        } else { None };
        // Conditional
        let calling_party_extension = if calling_party_type_identifier == 2 { 
            Some(buffer.read_field(24, "calling_party_extension")?) 
        } else { None };
        // Type1
        let pre_coded_status = buffer.read_field(16, "pre_coded_status")? as u16;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;


        // Type3
        let external_subscriber_number = if obit { 
            CmceType3Field::parse(buffer, "external_subscriber_number")? as Option<CmceType3Field>
        } else { None };

        // Type3
        let dm_ms_address = if obit { 
            CmceType3Field::parse(buffer, "dm_ms_address")? as Option<CmceType3Field>
        } else { None };

        
        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(DStatus { 
            calling_party_type_identifier, 
            calling_party_address_ssi, 
            calling_party_extension, 
            pre_coded_status, 
            external_subscriber_number, 
            dm_ms_address 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DStatus.into_raw(), 5);
        // Type1
        buffer.write_bits(self.calling_party_type_identifier as u64, 2);
        // Conditional
        if let Some(ref value) = self.calling_party_address_ssi {
            buffer.write_bits(*value, 24);
        }
        // Conditional
        if let Some(ref value) = self.calling_party_extension {
            buffer.write_bits(*value, 24);
        }
        // Type1
        buffer.write_bits(self.pre_coded_status as u64, 16);

        // Check if any optional field present and place o-bit
        let obit_val = self.external_subscriber_number.is_some() || self.dm_ms_address.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type3
        if let Some(ref value) = self.external_subscriber_number {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.dm_ms_address {
            CmceType3Field::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for DStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DStatus {{ calling_party_type_identifier: {:?} calling_party_address_ssi: {:?} calling_party_extension: {:?} pre_coded_status: {:?} external_subscriber_number: {:?} dm_ms_address: {:?} }}",
            self.calling_party_type_identifier,
            self.calling_party_address_ssi,
            self.calling_party_extension,
            self.pre_coded_status,
            self.external_subscriber_number,
            self.dm_ms_address,
        )
    }
}
