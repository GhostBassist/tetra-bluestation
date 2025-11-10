use core::fmt;

use crate::common::pdu_parse_error::PduParseError;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_ul::MmPduTypeUl;
use crate::entities::mm::enums::type34_elem_id_ul::MmType34ElemIdUl;
use crate::entities::mm::components::type34_fields::MmType3FieldUl;

/// Representation of the U-LOCATION UPDATE DEMAND PDU (Clause 16.9.3.4).
/// The MS sends this message to the infrastructure to request update of its location registration.
/// Response expected: D-LOCATION UPDATE ACCEPT/D-LOCATION UPDATE REJECT
/// Response to: -/D-LOCATION UPDATE COMMAND

// note 1: Information element "Ciphering parameters" is not present if "Cipher control" is set to "0" (ciphering off); present if set to "1" (ciphering on).
// note 2: If the "class of MS" or the "extended capabilities" element is not included and the SwMI needs either, it may accept the request and then send a D-LOCATION UPDATE COMMAND PDU.
#[derive(Debug)]
pub struct ULocationUpdateDemand {
    /// Type1, 3 bits, Location update type
    pub location_update_type: u8,
    /// Type1, 1 bits, Request to append LA
    pub request_to_append_la: bool,
    /// Type1, 1 bits, Cipher control
    pub cipher_control: bool,
    /// Conditional 10 bits, Ciphering parameters
    pub ciphering_parameters: Option<u64>,
    /// Type2, 24 bits, See note 2,
    pub class_of_ms: Option<u64>,
    /// Type2, 3 bits, Energy saving mode
    pub energy_saving_mode: Option<u64>,
    /// Type2, LA information
    pub la_information: Option<u64>,
    /// Type2, 24 bits, ISSI of the MS,
    pub ssi: Option<u64>,
    /// Type2, 24 bits, MNI of the MS,
    pub address_extension: Option<u64>,
    /// Type3, Group identity location demand
    pub group_identity_location_demand: Option<MmType3FieldUl>,
    /// Type3, 3 bits, Group report response
    pub group_report_response: Option<MmType3FieldUl>,
    /// Type3, 3 bits, See ETSI EN 300 392-7 [8],
    pub authentication_uplink: Option<MmType3FieldUl>,
    /// Type3, 3 bits, See note 2,
    pub extended_capabilities: Option<MmType3FieldUl>,
    /// Type3, 3 bits, Proprietary
    pub proprietary: Option<MmType3FieldUl>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl ULocationUpdateDemand {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseError> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeUl::ULocationUpdateDemand)?;
        
        // Type1
        let location_update_type = buffer.read_field(3, "location_update_type")? as u8;
        // Type1
        let request_to_append_la = buffer.read_field(1, "request_to_append_la")? != 0;
        // Type1
        let cipher_control = buffer.read_field(1, "cipher_control")? != 0;
        // Conditional
        let ciphering_parameters = if cipher_control { 
            Some(buffer.read_field(10, "ciphering_parameters")?)
        } else { 
            None
        };

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = typed_pdu_fields::delimiters::read_obit(buffer)?;

        // Type2
        let class_of_ms = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "class_of_ms")? as Option<u64>
        } else { None };
        // Type2
        let energy_saving_mode = if obit { 
            typed_pdu_fields::type2::parse(buffer, 3, "energy_saving_mode")? as Option<u64>
        } else { None };
        // Type2
        let la_information = if obit { 
            typed_pdu_fields::type2::parse(buffer, 999, "la_information")? as Option<u64>
        } else { None };
        // Type2
        let ssi = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "ssi")? as Option<u64>
        } else { None };
        // Type2
        let address_extension = if obit { 
            typed_pdu_fields::type2::parse(buffer, 24, "address_extension")? as Option<u64>
        } else { None };


        // Type3
        let group_identity_location_demand = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::GroupIdentityLocationDemand) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let group_report_response = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::GroupReportResponse) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };
        

        // Type3
        let authentication_uplink = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::AuthenticationUplink) {
            Ok(value) => Some(value),
            Err(_) => {None}
        };

        // Type3
        let extended_capabilities = match MmType3FieldUl::parse(buffer, MmType34ElemIdUl::ExtendedCapabilities) {
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

        Ok(ULocationUpdateDemand { 
            location_update_type, 
            request_to_append_la, 
            cipher_control, 
            ciphering_parameters, 
            class_of_ms, 
            energy_saving_mode, 
            la_information, 
            ssi, 
            address_extension, 
            group_identity_location_demand, 
            group_report_response, 
            authentication_uplink, 
            extended_capabilities, 
            proprietary
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseError> {
        // PDU Type
        buffer.write_bits(MmPduTypeUl::ULocationUpdateDemand.into_raw(), 4);
        // Type1
        buffer.write_bits(self.location_update_type as u64, 3);
        // Type1
        buffer.write_bits(self.request_to_append_la as u64, 1);
        // Type1
        buffer.write_bits(self.cipher_control as u64, 1);
        // Conditional
        if let Some(ref value) = self.ciphering_parameters {
            buffer.write_bits(*value, 10);
        }

        // Check if any optional field present and place o-bit
        let obit_val = self.class_of_ms.is_some() || self.energy_saving_mode.is_some() || self.la_information.is_some() || self.ssi.is_some() || self.address_extension.is_some() || self.group_identity_location_demand.is_some() || self.group_report_response.is_some() || self.authentication_uplink.is_some() || self.extended_capabilities.is_some() || self.proprietary.is_some() ;
        typed_pdu_fields::delimiters::write_obit(buffer, obit_val as u8);
        if !obit_val { return Ok(()); }

        // Type2
        typed_pdu_fields::type2::write(buffer, self.class_of_ms, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.energy_saving_mode, 3);

        // Type2
        unimplemented!();
            typed_pdu_fields::type2::write(buffer, self.la_information, 999);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.ssi, 24);

        // Type2
        typed_pdu_fields::type2::write(buffer, self.address_extension, 24);

        // Type3
        if let Some(ref value) = self.group_identity_location_demand {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.group_report_response {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.authentication_uplink {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
        }
        // Type3
        if let Some(ref value) = self.extended_capabilities {
            MmType3FieldUl::write(buffer, value.field_type, value.data, value.len);
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

impl fmt::Display for ULocationUpdateDemand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ULocationUpdateDemand {{ location_update_type: {:?} request_to_append_la: {:?} cipher_control: {:?} ciphering_parameters: {:?} class_of_ms: {:?} energy_saving_mode: {:?} la_information: {:?} ssi: {:?} address_extension: {:?} group_identity_location_demand: {:?} group_report_response: {:?} authentication_uplink: {:?} extended_capabilities: {:?} proprietary: {:?} }}",
            self.location_update_type,
            self.request_to_append_la,
            self.cipher_control,
            self.ciphering_parameters,
            self.class_of_ms,
            self.energy_saving_mode,
            self.la_information,
            self.ssi,
            self.address_extension,
            self.group_identity_location_demand,
            self.group_report_response,
            self.authentication_uplink,
            self.extended_capabilities,
            self.proprietary,
        )
    }
}
