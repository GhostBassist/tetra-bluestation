use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_dl::MmPduTypeDl;
use crate::entities::mm::enums::type34_elem_id_dl::MmType34ElemIdDl;
use crate::entities::mm::components::type34_fields::MmType3FieldDl;

/// Representation of the D-LOCATION UPDATE PROCEEDING PDU (Clause 16.9.2.10).
/// The infrastructure sends this message to the MS on registration at accepted migration to assign a (V)ASSI.
/// Response expected: -
/// Response to: U-LOCATION UPDATE DEMAND

#[derive(Debug)]
pub struct DLocationUpdateProceeding {
    /// Type1, 24 bits, (V)ASSI of the MS,
    pub ssi: u32,
    /// Type1, 24 bits, MNI of the MS,
    pub address_extension: u32,
    /// Type3, Proprietary
    pub proprietary: Option<MmType3FieldDl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DLocationUpdateProceeding {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeDl::DLocationUpdateProceeding)?;
        
        // Type1
        let ssi = buffer.read_field(24, "ssi")? as u32;
        // Type1
        let address_extension = buffer.read_field(24, "address_extension")? as u32;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type3
        let proprietary = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::Proprietary) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(DLocationUpdateProceeding { 
            ssi, 
            address_extension, 
            proprietary
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(MmPduTypeDl::DLocationUpdateProceeding.into_raw(), 4);
        // Type1
        buffer.write_bits(self.ssi as u64, 24);
        // Type1
        buffer.write_bits(self.address_extension as u64, 24);

        // Check if any optional field present and place o-bit
        let obit_val = self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type3
        if let Some(ref value) = self.proprietary {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for DLocationUpdateProceeding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DLocationUpdateProceeding {{ ssi: {:?} address_extension: {:?} proprietary: {:?} }}",
            self.ssi,
            self.address_extension,
            self.proprietary,
        )
    }
}
