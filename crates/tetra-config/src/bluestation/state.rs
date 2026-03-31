use std::collections::{HashMap, HashSet};
use tetra_core::TimeslotAllocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriberChannelState {
    Mcch,
    AssignedTraffic { call_id: u16, gssi: u32, ts: u8 },
    AssignedControl { call_id: u16, gssi: u32, ts: u8 },
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub issi: u32,
    // Set of attached GSSIs
    pub attached_groups: HashSet<u32>,
    pub channel_state: SubscriberChannelState,
}

/// Centralized subscriber registry tracking locally registered ISSIs and their group affiliations.
#[derive(Debug, Clone)]
pub struct SubscriberRegistry {
    /// Registered ISSIs → Subscriber information
    subscribers: HashMap<u32, Subscriber>,
    /// Set of all GSSIs with at least one local affiliate
    all_attached_groups: HashSet<u32>,
}

impl SubscriberRegistry {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            all_attached_groups: HashSet::new(),
        }
    }

    pub fn is_registered(&self, issi: u32) -> bool {
        self.subscribers.contains_key(&issi)
    }

    /// Tolerant registration; if ISSI already registered, we overwrite it with a fresh Subscriber struct
    pub fn register(&mut self, issi: u32) {
        self.deregister(issi); // Clean up any existing registration to prevent stale affiliations
        self.subscribers.insert(
            issi,
            Subscriber {
                issi,
                attached_groups: HashSet::new(),
                channel_state: SubscriberChannelState::Mcch,
            },
        );
    }

    /// Gets mutable ref to subscriber. If not registered, a default Subscriber is inserted.
    pub fn get_subscriber_mut(&mut self, issi: u32) -> &mut Subscriber {
        self.subscribers.entry(issi).or_insert_with(|| Subscriber {
            issi,
            attached_groups: HashSet::new(),
            channel_state: SubscriberChannelState::Mcch,
        })
    }

    pub fn get_channel_state(&self, issi: u32) -> Option<SubscriberChannelState> {
        self.subscribers.get(&issi).map(|subscriber| subscriber.channel_state)
    }

    pub fn set_channel_state(&mut self, issi: u32, channel_state: SubscriberChannelState) {
        self.get_subscriber_mut(issi).channel_state = channel_state;
    }

    pub fn clear_channel_state(&mut self, issi: u32) {
        if let Some(subscriber) = self.subscribers.get_mut(&issi) {
            subscriber.channel_state = SubscriberChannelState::Mcch;
        }
    }

    pub fn clear_channel_state_if_call(&mut self, issi: u32, call_id: u16) {
        let Some(subscriber) = self.subscribers.get_mut(&issi) else {
            return;
        };

        let matches_call = match subscriber.channel_state {
            SubscriberChannelState::AssignedTraffic { call_id: active_call_id, .. }
            | SubscriberChannelState::AssignedControl { call_id: active_call_id, .. } => active_call_id == call_id,
            SubscriberChannelState::Mcch => false,
        };

        if matches_call {
            subscriber.channel_state = SubscriberChannelState::Mcch;
        }
    }

    pub fn set_group_channel_state(&mut self, gssi: u32, channel_state: SubscriberChannelState) {
        for subscriber in self.subscribers.values_mut() {
            if subscriber.attached_groups.contains(&gssi) {
                subscriber.channel_state = channel_state;
            }
        }
    }

    pub fn clear_group_channel_state_if_call(&mut self, gssi: u32, call_id: u16) {
        for subscriber in self.subscribers.values_mut() {
            if !subscriber.attached_groups.contains(&gssi) {
                continue;
            }

            let matches_call = match subscriber.channel_state {
                SubscriberChannelState::AssignedTraffic {
                    call_id: active_call_id,
                    gssi: active_gssi,
                    ..
                }
                | SubscriberChannelState::AssignedControl {
                    call_id: active_call_id,
                    gssi: active_gssi,
                    ..
                } => active_call_id == call_id && active_gssi == gssi,
                SubscriberChannelState::Mcch => false,
            };

            if matches_call {
                subscriber.channel_state = SubscriberChannelState::Mcch;
            }
        }
    }

    pub fn get_group_channel_state(&self, gssi: u32) -> Option<SubscriberChannelState> {
        let mut assigned_control = None;

        for subscriber in self.subscribers.values() {
            if !subscriber.attached_groups.contains(&gssi) {
                continue;
            }

            match subscriber.channel_state {
                state @ SubscriberChannelState::AssignedTraffic { .. } => return Some(state),
                state @ SubscriberChannelState::AssignedControl { .. } => {
                    if assigned_control.is_none() {
                        assigned_control = Some(state);
                    }
                }
                SubscriberChannelState::Mcch => {}
            }
        }

        assigned_control
    }

    /// Deregister an ISSI, removing it from the registry and cleaning up any group affiliations
    pub fn deregister(&mut self, issi: u32) {
        if let Some(subscriber) = self.subscribers.remove(&issi) {
            // Clean up global group affiliations for this subscriber
            for gssi in &subscriber.attached_groups {
                // Check if any other subscriber is still affiliated with this group
                let still_has_members = self.subscribers.values().any(|s| s.attached_groups.contains(gssi));
                if !still_has_members {
                    self.all_attached_groups.remove(gssi);
                }
            }
        }
    }

    /// Add GSSI to subscriber's attached groups and global set
    pub fn affiliate(&mut self, issi: u32, gssi: u32) {
        let subscriber = self.get_subscriber_mut(issi);
        subscriber.attached_groups.insert(gssi);
        self.all_attached_groups.insert(gssi);
    }

    /// Remove GSSI from subscriber's attached groups. Update global set if no more subscribers are affiliated with this GSSI.
    pub fn deaffiliate(&mut self, issi: u32, gssi: u32) {
        let subscriber = self.get_subscriber_mut(issi);
        if subscriber.attached_groups.remove(&gssi) {
            // Check if any other subscriber is still affiliated with this group
            let still_has_members = self.subscribers.values().any(|s| s.attached_groups.contains(&gssi));
            if !still_has_members {
                self.all_attached_groups.remove(&gssi);
            }
        }
    }

    /// Check if any subscriber is affiliated with the given GSSI
    pub fn has_group_members(&self, gssi: u32) -> bool {
        self.all_attached_groups.contains(&gssi)
    }
}

/// Mutable, stack-editable state (mutex-protected).
#[derive(Debug, Clone)]
pub struct StackState {
    pub timeslot_alloc: TimeslotAllocator,
    /// Backhaul/network connection to SwMI (e.g., Brew/TetraPack). False -> fallback mode.
    pub network_connected: bool,
    /// Centralized subscriber registry for local-first routing decisions.
    pub subscribers: SubscriberRegistry,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_deregister() {
        let mut reg = SubscriberRegistry::new();
        assert!(!reg.is_registered(1001));
        reg.register(1001);
        assert!(reg.is_registered(1001));
        reg.deregister(1001);
        assert!(!reg.is_registered(1001));
    }

    #[test]
    fn test_affiliate_deaffiliate() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.affiliate(1001, 91);
        assert!(reg.has_group_members(91));
        reg.deaffiliate(1001, 91);
        assert!(!reg.has_group_members(91));
    }

    #[test]
    fn test_has_group_members() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.register(1002);
        reg.register(1003);
        reg.affiliate(1001, 100);
        reg.affiliate(1002, 100);
        reg.affiliate(1003, 100);
        assert!(reg.has_group_members(100));

        // Deaffiliate one, should still have members
        reg.deaffiliate(1001, 100);
        assert!(reg.has_group_members(100));

        // Deregister a user, should still have members
        reg.deregister(1002);
        assert!(reg.has_group_members(100));

        // Deregister last user, should have no members
        reg.deregister(1003);
        assert!(!reg.has_group_members(100));
    }

    #[test]
    fn test_has_group_members_empty() {
        let reg = SubscriberRegistry::new();
        assert!(!reg.has_group_members(999));
    }

    #[test]
    fn test_register_overwrites_existing_subscriber() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.affiliate(1001, 91);
        assert!(reg.has_group_members(91));

        reg.register(1001);

        assert!(reg.is_registered(1001));
        reg.deaffiliate(1001, 91);
        assert!(!reg.has_group_members(91));
    }

    #[test]
    fn test_channel_state_per_subscriber() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);

        reg.set_channel_state(
            1001,
            SubscriberChannelState::AssignedTraffic {
                call_id: 18,
                gssi: 91,
                ts: 2,
            },
        );
        assert_eq!(
            reg.get_channel_state(1001),
            Some(SubscriberChannelState::AssignedTraffic {
                call_id: 18,
                gssi: 91,
                ts: 2,
            })
        );

        reg.clear_channel_state_if_call(1001, 18);
        assert_eq!(reg.get_channel_state(1001), Some(SubscriberChannelState::Mcch));
    }

    #[test]
    fn test_group_channel_state_tracks_members() {
        let mut reg = SubscriberRegistry::new();
        reg.register(1001);
        reg.register(1002);
        reg.affiliate(1001, 91);
        reg.affiliate(1002, 91);

        reg.set_group_channel_state(
            91,
            SubscriberChannelState::AssignedControl {
                call_id: 18,
                gssi: 91,
                ts: 2,
            },
        );

        assert_eq!(
            reg.get_group_channel_state(91),
            Some(SubscriberChannelState::AssignedControl {
                call_id: 18,
                gssi: 91,
                ts: 2,
            })
        );

        reg.clear_group_channel_state_if_call(91, 18);
        assert_eq!(reg.get_group_channel_state(91), None);
        assert_eq!(reg.get_channel_state(1001), Some(SubscriberChannelState::Mcch));
        assert_eq!(reg.get_channel_state(1002), Some(SubscriberChannelState::Mcch));
    }
}

impl Default for StackState {
    fn default() -> Self {
        Self {
            timeslot_alloc: TimeslotAllocator::default(),
            network_connected: false,
            subscribers: SubscriberRegistry::new(),
        }
    }
}
