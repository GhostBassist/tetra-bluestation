use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_dl::MmPduTypeDl;
use crate::entities::mm::enums::type34_elem_id_dl::MmType34ElemIdDl;
use crate::entities::mm::components::type34_fields::{MmType3FieldDl,MmType4FieldDl};
use crate::entities::mm::fields::group_identity_downlink::GroupIdentityDownlink;

/// Representation of the D-ATTACH/DETACH GROUP IDENTITY ACKNOWLEDGEMENT PDU (Clause 16.9.2.2).
/// The infrastructure sends this message to the MS to acknowledge MS-initiated attachment/detachment of group identities.
/// Response expected: -
/// Response to: U-ATTACH/DETACH GROUP IDENTITY

// Note: The MS shall accept the type 3/4 information elements both in the numerical order as described in annex E and in the order shown in this table.
#[derive(Debug)]
pub struct DAttachDetachGroupIdentityAcknowledgement {
    /// Type1, 1 bits, Group identity accept/reject
    pub group_identity_accept_reject: u8,
    /// Type1, 1 bits, Reserved
    pub reserved: bool,
    /// Type3, See note,
    pub proprietary: Option<MmType3FieldDl>,
    /// Type4, See note,
    pub group_identity_downlink: Option<Vec<GroupIdentityDownlink>>,
    /// Type4, See ETSI EN 300 392-7 [8] and note,
    pub group_identity_security_related_information: Option<MmType4FieldDl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DAttachDetachGroupIdentityAcknowledgement {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeDl::DAttachDetachGroupIdentityAcknowledgement)?;
        
        // Type1
        let group_identity_accept_reject = buffer.read_field(1, "group_identity_accept_reject")? as u8;
        // Type1
        let reserved = buffer.read_field(1, "reserved")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type3
        // let proprietary = if mbit { MmType3FieldDl::parse(buffer, "proprietary")? as Option<MmType3FieldDl> } else { None };
        let proprietary = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::Proprietary) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };
        
        let type4_field = MmType4FieldDl::parse_header(buffer, MmType34ElemIdDl::GroupIdentityDownlink);
        let group_identity_downlink = match type4_field {
            Ok((num_elems, _len_bits)) => {
                let mut elems = Vec::with_capacity(num_elems);
                for _ in 0..num_elems {
                    elems.push(GroupIdentityDownlink::from_bitbuf(buffer)?);
                }
                Some(elems)
            },
            Err(_) => None
        };
        
        // Type4
        let group_identity_security_related_information = match MmType4FieldDl::parse(buffer, MmType34ElemIdDl::GroupIdentitySecurityRelatedInformation) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };            

        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseError::InvalidObitValue);
        }

        Ok(DAttachDetachGroupIdentityAcknowledgement { 
            group_identity_accept_reject, 
            reserved, 
            proprietary, 
            group_identity_downlink, 
            group_identity_security_related_information
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(MmPduTypeDl::DAttachDetachGroupIdentityAcknowledgement.into_raw(), 4);
        // Type1
        buffer.write_bits(self.group_identity_accept_reject as u64, 1);
        // Type1
        buffer.write_bits(self.reserved as u64, 1);

        // Check if any optional field present and place o-bit
        let obit_val = self.proprietary.is_some() || self.group_identity_downlink.is_some() || self.group_identity_security_related_information.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type3
        if let Some(ref value) = self.proprietary {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }

        // Type4
        if let Some(value) = &self.group_identity_downlink {
            MmType4FieldDl::write_field(buffer, MmType34ElemIdDl::GroupIdentityDownlink, value);
        }

        // Type4
        if let Some(ref value) = self.group_identity_security_related_information {
            MmType4FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for DAttachDetachGroupIdentityAcknowledgement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DAttachDetachGroupIdentityAcknowledgement {{ group_identity_accept_reject: {:?} reserved: {:?} proprietary: {:?} group_identity_downlink: {:?} group_identity_security_related_information: {:?} }}",
            self.group_identity_accept_reject,
            self.reserved,
            self.proprietary,
            self.group_identity_downlink,
            self.group_identity_security_related_information,
        )
    }
}



#[cfg(test)]
mod tests {
    use crate::common::debug::setup_logging_default;

    use super::*;

    #[test]
    fn decode_encode_test() {

        setup_logging_default();
        // Vec from lab
        let test_vec = "10110011011100000100110000001011100000000110101000110011100000";

        // 10110011011100000100110000001011100000000110101000110011100000
        // |--| identifier
        //     | accept/reject
        //      | reserved 
        //       || obit, mbit
        //         |--| identifier: 0x7 GroupIdentityDownlink
        //             |---------| len: 38
        //                        |------------------------------------| field
        //
        // 000001 01110010000001010100011001110000
        // |----| num elems: 1
        //        | attach/detach type identifier
        //         || lifetime: until next location update
        //           |-| class of usage: 4
        //              || type identifier
        //                |----------------| gssi: 0x000000
        
        let mut buffer = BitBuffer::from_bitstr(test_vec);

        let pdu = match DAttachDetachGroupIdentityAcknowledgement::from_bitbuf(&mut buffer) {
            Ok(pdu) => {
                tracing::debug!("<- {:?}", pdu);
                pdu
            }
            Err(e) => {
                tracing::warn!("Failed parsing DAttachDetachGroupIdentityAcknowledgement: {:?} {}", e, buffer.dump_bin());
                return;
            }
        };
        
        tracing::info!("Parsed: {:?}", pdu);
        tracing::info!("Buf at end: {}", buffer.dump_bin());

        let mut buf = BitBuffer::new_autoexpand(32);
        pdu.to_bitbuf(&mut buf).unwrap();
        tracing::info!("Serialized: {}", buf.dump_bin());
        assert_eq!(buf.to_bitstr(), test_vec);

    }
}
