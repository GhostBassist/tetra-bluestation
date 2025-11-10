
// // Clause 22.3 LLC procedures

// use std::collections::HashMap;

// use crate::{tetra_common::{address::TetraAddress, tetra_common::Todo}, entities::umac::fields::endpoint_id::EndpointId};
// use rand::Rng;

// /// Link identifiers between the service user (MLE) and LLC shall serve to distinguish between the multiple concurrent
// /// services, e.g. among several advanced links and their associated basic links. These identifiers may be local.
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct LinkId {
//     pub id: u32,
// }

// /// When the LLC receives a service request primitive (except TL-RELEASE request) from the MLE, the primitive
// /// includes a local identifier for the service request, referred to as the "handle to the request". The handle should be
// /// retained locally and used for identifying subsequent related service primitives. It refers to all actions required in the
// /// LLC to accomplish the request. LLC shall also pass the handle to the request parameter to the MAC layer. In a similar
// /// way the MAC associates a handle to the request to each data request and the LLC shall use that handle to the request
// /// when it refers to that transmission.
// pub struct ReqHandle {
//     pub id: u32,
// }

// impl ReqHandle {
//     /// Generates a random handle. These may be created by the LLC or MLE.
//     /// TODO FIXME: we rely on chance to avoid collisions.
//     pub fn new() -> Self {
//         Self {
//             id: rand::rng().random()
//         }
//     }
// }

// pub struct BlLink {
//     /// Which MAC resource is used for this link
//     pub endpoint_id: EndpointId,
    
//     pub link_id: LinkId,
    
//     pub handle: ReqHandle,

//     /// If None, no ack is scheduled for transmission
//     /// If Some, holds the sequence number of the ack that needs to be sent (0 or 1)
//     pub ack_that_needs_to_be_sent: Option<u8>,

//     /// If None, no ack is expected
//     /// If Some, holds the sequence number of the ack that is expected (0 or 1)
//     /// We should then receive a BL-ACK or BL-ADATA shortly
//     pub expected_ack: Option<u8>,
    
//     /// Unacked sent PDU that may be retransmitted if ACK is not received
//     pub unacked_txed_pdu: Option<Todo>,

//     // TODO expiry timers
// }

// pub struct BlLinkManager {
//     pub next_req_handle: u32,
//     pub links: HashMap<LinkId, BlLink>,
// }

// impl BlLinkManager {
//     pub fn new() -> Self {
//         Self {
//             next_req_handle: 1,
//             links: HashMap::new(),
//         }
//     }

//     pub fn get_link_by_id(&self, link_id: &LinkId) -> Option<&BlLink> {
//         self.links.get(link_id)
//     }

//     // pub fn add_link(&mut self, link_id: LinkId, link: BlLink) {
//     //     self.links.insert(link_id, link);
//     // }

// }