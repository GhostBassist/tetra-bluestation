use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_ul::MmPduTypeUl;
use crate::entities::mm::enums::type34_elem_id_ul::MmType34ElemIdUl;
use crate::entities::mm::components::type34_fields::{MmType3FieldUl,MmType4FieldUl};

/// Representation of the U-ATTACH/DETACH GROUP IDENTITY ACKNOWLEDGEMENT PDU (Clause 16.9.3.2).
/// The MS sends this message to the infrastructure to acknowledge SwMI initiated attachment/detachment of group identities.
/// Response expected: -
/// Response to: D-ATTACH/DETACH GROUP IDENTITY

#[derive(Debug)]
pub struct UAttachDetachGroupIdentityAcknowledgement {
    /// Type1, 1 bits, Group identity acknowledgement type
    pub group_identity_acknowledgement_type: bool,
    /// Type4, Group identity uplink
    pub group_identity_uplink: Option<MmType4FieldUl>,
    /// Type3, Proprietary
    pub proprietary: Option<MmType3FieldUl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl UAttachDetachGroupIdentityAcknowledgement {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeUl::UAttachDetachGroupIdentityAcknowledgement)?;
        
        // Type1
        let group_identity_acknowledgement_type = buffer.read_field(1, "group_identity_acknowledgement_type")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type4
        let group_identity_uplink = match MmType4FieldUl::parse(buffer, MmType34ElemIdUl::GroupIdentityUplink) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let proprietary = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::Proprietary) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };
        
        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(UAttachDetachGroupIdentityAcknowledgement { 
            group_identity_acknowledgement_type, 
            group_identity_uplink, 
            proprietary
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(MmPduTypeUl::UAttachDetachGroupIdentityAcknowledgement.into_raw(), 4);
        // Type1
        buffer.write_bits(self.group_identity_acknowledgement_type as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.group_identity_uplink.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type4
        if let Some(ref value) = self.group_identity_uplink {
            MmType4FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.proprietary {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for UAttachDetachGroupIdentityAcknowledgement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UAttachDetachGroupIdentityAcknowledgement {{ group_identity_acknowledgement_type: {:?} group_identity_uplink: {:?} proprietary: {:?} }}",
            self.group_identity_acknowledgement_type,
            self.group_identity_uplink,
            self.proprietary,
        )
    }
}
