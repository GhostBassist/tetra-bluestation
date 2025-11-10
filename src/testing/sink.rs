use crate::{common::messagerouter::MessageQueue, saps::sapmsg::SapMsg, common::{tetra_entities::TetraEntity}, entities::TetraEntityTrait};

/// A TETRA component sink for testing purposes
/// Collects all received SapMsg messages for later inspection
pub struct Sink {
    component: TetraEntity,
    msgqueue: Vec<SapMsg>,
}

impl Sink {
    pub fn new(component: TetraEntity) -> Self {
        Self {
            component,
            msgqueue: vec![],
        }
    }

    pub fn take_msgqueue(&mut self) -> Vec<SapMsg> {
        std::mem::take(&mut self.msgqueue)
    }
}

impl TetraEntityTrait for Sink {
    
    fn entity(&self) -> TetraEntity {
        self.component
    }

    fn rx_prim(&mut self, _queue: &mut MessageQueue, message: SapMsg) {
        self.msgqueue.push(message);
    }
}