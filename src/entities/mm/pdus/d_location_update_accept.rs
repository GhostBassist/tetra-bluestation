use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_location_update_accept_type::MmLocationUpdateAcceptType;
use crate::entities::mm::enums::mm_pdu_type_dl::MmPduTypeDl;
use crate::entities::mm::enums::type34_elem_id_dl::MmType34ElemIdDl;
use crate::entities::mm::components::type34_fields::{MmType3FieldDl,MmType4FieldDl};

/// Representation of the D-LOCATION UPDATE ACCEPT PDU (Clause 16.9.2.7).
/// The infrastructure sends this message to the MS to indicate that updating in the network has been completed.
/// Response expected: -
/// Response to: U-LOCATION UPDATE DEMAND

// Note: The MS shall accept the type 3/4 information elements both in the numerical order as described in annex E and in the order shown in this table.
#[derive(Debug)]
pub struct DLocationUpdateAccept {
    /// Type1, 3 bits, Location update accept type
    pub location_update_accept_type: MmLocationUpdateAcceptType,
    /// Type2, 24 bits, ASSI/(V)ASSI of MS,
    pub ssi: Option<u64>,
    /// Type2, 24 bits, MNI of MS,
    pub address_extension: Option<u64>,
    /// Type2, 16 bits, Subscriber class
    pub subscriber_class: Option<u64>,
    /// Type2, 14 bits, Energy saving information
    pub energy_saving_information: Option<u64>,
    /// Type2, 6 bits, SCCH information and distribution on 18th frame
    pub scch_information_and_distribution_on_18th_frame: Option<u64>,
    /// Type4, See note,
    pub new_registered_area: Option<MmType4FieldDl>,
    /// Type3, See ETSI EN 300 392-7 [8],
    pub security_downlink: Option<MmType3FieldDl>,
    /// Type3, See note,
    pub group_identity_location_accept: Option<MmType3FieldDl>,
    /// Type3, See note,
    pub default_group_attachment_lifetime: Option<MmType3FieldDl>,
    /// Type3, See ETSI EN 300 392-7 [8],
    pub authentication_downlink: Option<MmType3FieldDl>,
    /// Type4, See ETSI EN 300 392-7 [8],
    pub group_identity_security_related_information: Option<MmType4FieldDl>,
    /// Type3, Cell type control
    pub cell_type_control: Option<MmType3FieldDl>,
    /// Type3, Proprietary
    pub proprietary: Option<MmType3FieldDl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DLocationUpdateAccept {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeDl::DLocationUpdateAccept)?;
        
        // Type1
        let val: u64 = buffer.read_field(3, "location_update_accept_type")?;
        let result = MmLocationUpdateAcceptType::try_from(val);
        let location_update_accept_type = match result {
            Ok(x) => x,
            Err(_) => return Err(PduParseError::InvalidValue{field: "location_update_accept_type", value: val})
        };

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let ssi = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "ssi")? as Option<u64>
        } else { None };
        // Type2
        let address_extension = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "address_extension")? as Option<u64>
        } else { None };
        // Type2
        let subscriber_class = if obit { 
            typed_pdu_fields::type2::parse(buffer, 16, "subscriber_class")? as Option<u64>
        } else { None };
        // Type2
        let energy_saving_information = if obit { 
            typed_pdu_fields::type2::parse(buffer, 14, "energy_saving_information")? as Option<u64>
        } else { None };
        // Type2
        let scch_information_and_distribution_on_18th_frame = if obit { 
            typed_pdu_fields::type2::parse(buffer, 6, "scch_information_and_distribution_on_18th_frame")? as Option<u64>
        } else { None };

        // Type4
        let new_registered_area = match MmType4FieldDl::parse(buffer, MmType34ElemIdDl::NewRegisteredArea) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };
        
        // Type3
        let security_downlink = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::SecurityDownlink) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let group_identity_location_accept = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::GroupIdentityLocationAccept) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let default_group_attachment_lifetime = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::DefaultGroupAttachLifetime) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let authentication_downlink = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::AuthenticationDownlink) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type4
        let group_identity_security_related_information = match MmType4FieldDl::parse(buffer, MmType34ElemIdDl::GroupIdentitySecurityRelatedInformation) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let cell_type_control = match MmType3FieldDl::parse(buffer, MmType34ElemIdDl::CellTypeControl) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

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

        Ok(DLocationUpdateAccept { 
            location_update_accept_type, 
            ssi, 
            address_extension, 
            subscriber_class, 
            energy_saving_information, 
            scch_information_and_distribution_on_18th_frame, 
            new_registered_area, 
            security_downlink, 
            group_identity_location_accept, 
            default_group_attachment_lifetime, 
            authentication_downlink, 
            group_identity_security_related_information, 
            cell_type_control, 
            proprietary
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) {
        // PDU Type
        buffer.write_bits(MmPduTypeDl::DLocationUpdateAccept.into_raw(), 4);
        // Type1
        buffer.write_bits(self.location_update_accept_type as u64, 3);

        // Check if any optional field present and place o-bit
        let obit_val = self.ssi.is_some() || self.address_extension.is_some() || self.subscriber_class.is_some() || self.energy_saving_information.is_some() || self.scch_information_and_distribution_on_18th_frame.is_some() || self.new_registered_area.is_some() || self.security_downlink.is_some() || self.group_identity_location_accept.is_some() || self.default_group_attachment_lifetime.is_some() || self.authentication_downlink.is_some() || self.group_identity_security_related_information.is_some() || self.cell_type_control.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return; }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.ssi, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.address_extension, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.subscriber_class, 16);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.energy_saving_information, 14);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.scch_information_and_distribution_on_18th_frame, 6);

        // Type4
        if let Some(ref value) = self.new_registered_area {
            MmType4FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.security_downlink {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.group_identity_location_accept {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.default_group_attachment_lifetime {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.authentication_downlink {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type4
        if let Some(ref value) = self.group_identity_security_related_information {
            MmType4FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.cell_type_control {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.proprietary {
            MmType3FieldDl::write(buffer, value.field_type, value.data, value.len);
        }
        // Write terminating m-bit
        typed_pdu_fields::delimiters::write_mbit(buffer, 0);
    }
}

impl fmt::Display for DLocationUpdateAccept {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DLocationUpdateAccept {{ location_update_accept_type: {:?} ssi: {:?} address_extension: {:?} subscriber_class: {:?} energy_saving_information: {:?} scch_information_and_distribution_on_18th_frame: {:?} new_registered_area: {:?} security_downlink: {:?} group_identity_location_accept: {:?} default_group_attachment_lifetime: {:?} authentication_downlink: {:?} group_identity_security_related_information: {:?} cell_type_control: {:?} proprietary: {:?} }}",
            self.location_update_accept_type,
            self.ssi,
            self.address_extension,
            self.subscriber_class,
            self.energy_saving_information,
            self.scch_information_and_distribution_on_18th_frame,
            self.new_registered_area,
            self.security_downlink,
            self.group_identity_location_accept,
            self.default_group_attachment_lifetime,
            self.authentication_downlink,
            self.group_identity_security_related_information,
            self.cell_type_control,
            self.proprietary,
        )
    }
}
