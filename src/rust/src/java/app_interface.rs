use crate::{
    core::{
        group_call::{ClientId, EndReason},
    },
    java::{
        jtypes::JArrayByte,
    },
};

#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct AppInterface {
    /// Java object clean up method.
    pub destroy: extern "C" fn(),

    pub groupConnectionStateChanged: extern "C" fn(clientId: ClientId, connection_state: i32),
    pub groupEnded: extern "C" fn(clientId: ClientId, reason: EndReason),
    pub groupJoinStateChanged: extern "C" fn(clientId: ClientId, join_state: i32),
    pub groupRequestGroupMembers: extern "C" fn(clientId: ClientId),
    pub groupRequestMembershipProof: extern "C" fn(clientId: ClientId),
    pub groupRing: extern "C" fn(groupId: JArrayByte, ringId: i64, senderId: JArrayByte, update: i32),

    pub sendCallMessage: extern "C" fn(recipient: JArrayByte, message: JArrayByte, urgency: i32),
    pub sendCallMessageToGroup: extern "C" fn(groupId: JArrayByte, message: JArrayByte, urgency: i32),

    pub signalingMessageAnswer: extern "C" fn(answer: JArrayByte),
    pub signalingMessageIce: extern "C" fn(ice: JArrayByte),
    pub signalingMessageOffer: extern "C" fn(offer: JArrayByte),
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppInterface {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppInterface {}

// Rust owns the interface object from Java. Drop it when it goes out
// of scope.
impl Drop for AppInterface {
    fn drop(&mut self) {
        (self.destroy)();
    }
}
