//
// Copyright 2019-2021 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

//! WebRTC Peer Connection Interface
use std::{ffi::CString, net::SocketAddr};

pub use pc::RffiPeerConnection;

#[cfg(not(feature = "sim"))]
use crate::webrtc::ffi::peer_connection as pc;
#[cfg(feature = "sim")]
use crate::webrtc::sim::peer_connection as pc;
#[cfg(feature = "sim")]
pub use crate::webrtc::sim::peer_connection::BoxedRtpPacketSink;
use crate::{
    common::{units::DataRate, Result},
    core::util::redact_string,
    error::RingRtcError,
    webrtc,
    webrtc::{
        ice_gatherer::IceGatherer,
        media::AudioEncoderConfig,
        network::RffiIpPort,
        peer_connection_factory::RffiPeerConnectionFactoryOwner,
        peer_connection_observer::RffiPeerConnectionObserver,
        rtp,
        sdp_observer::{
            CreateSessionDescriptionObserver, SessionDescription, SetSessionDescriptionObserver,
        },
        stats_observer::StatsObserver,
    },
};

/// Rust wrapper around WebRTC C++ PeerConnection object.
#[derive(Debug)]
pub struct PeerConnection {
    rffi: webrtc::Arc<RffiPeerConnection>,
    // We keep this around as an easy way to make sure the PeerConnectionFactory
    // outlives the PeerConnection.  A PCF must outlive a PC because the PCF
    // owns the threads that the PC relies on.  If the PCF closes those threads,
    // not only will the PC do nothing, but methods called on it will block
    // indefinitely.
    _owner: Option<webrtc::Arc<RffiPeerConnectionFactoryOwner>>,

    // Used for optionally keeping the PeerConnectionObserver around longer
    // than the native PeerConnection.
    _rffi_pc_observer: Option<webrtc::ptr::Unique<RffiPeerConnectionObserver>>,
}

impl Drop for PeerConnection {
    fn drop(&mut self) {
        // Delete the rffi before the _owner and the rffi_pc_observer.
        self.rffi = webrtc::Arc::null();

        // Now it's safe to delete the _owner and the rffi_pc_observer.
    }
}

// See PeerConnection::SetSendRates for more info.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SendRates {
    pub min: Option<DataRate>,
    pub start: Option<DataRate>,
    pub max: Option<DataRate>,
}

pub type RffiAudioLevel = u16;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct RffiReceivedAudioLevel {
    pub demux_id: u32,
    pub level: RffiAudioLevel,
}

pub type AudioLevel = RffiAudioLevel;
pub type ReceivedAudioLevel = RffiReceivedAudioLevel;

pub enum Protocol<'a> {
    Udp,
    Tcp,
    Tls(&'a str),
}

impl PeerConnection {
    pub fn new(
        rffi: webrtc::Arc<RffiPeerConnection>,
        rffi_pc_observer: Option<webrtc::ptr::Unique<RffiPeerConnectionObserver>>,
        owner: Option<webrtc::Arc<RffiPeerConnectionFactoryOwner>>,
    ) -> Self {
        Self {
            rffi,
            _rffi_pc_observer: rffi_pc_observer,
            _owner: owner,
        }
    }

    #[cfg(feature = "sim")]
    pub fn set_rtp_packet_sink(&self, rtp_packet_sink: BoxedRtpPacketSink) {
        unsafe { self.rffi.as_borrowed().as_ref() }
            .unwrap()
            .set_rtp_packet_sink(rtp_packet_sink)
    }

    pub fn update_transceivers(&self, remote_demux_ids: &[u32]) -> Result<()> {
        let success = unsafe {
            pc::Rust_updateTransceivers(
                self.rffi.as_borrowed(),
                webrtc::ptr::Borrowed::from_ptr(remote_demux_ids.as_ptr()),
                remote_demux_ids.len(),
            )
        };

        if success {
            Ok(())
        } else {
            Err(RingRtcError::UpdateTransceivers.into())
        }
    }

    /// Rust wrapper around C++ webrtc::CreateSessionDescription(kOffer).
    pub fn create_offer(&self, csd_observer: &CreateSessionDescriptionObserver) {
        unsafe { pc::Rust_createOffer(self.rffi.as_borrowed(), csd_observer.rffi().as_borrowed()) }
    }

    /// Rust wrapper around C++ PeerConnection::SetLocalDescription().
    pub fn set_local_description(
        &self,
        ssd_observer: &SetSessionDescriptionObserver,
        session_description: SessionDescription,
    ) {
        // Rust_setLocalDescription takes ownership of the local description
        // We take out the interface (with take_rffi) so that when the SessionDescriptionInterface
        // is deleted, we don't double delete.
        unsafe {
            pc::Rust_setLocalDescription(
                self.rffi.as_borrowed(),
                ssd_observer.rffi().as_borrowed(),
                session_description.take_rffi().into_owned(),
            )
        }
    }

    /// Rust wrapper around C++ webrtc::CreateSessionDescription(kAnswer).
    pub fn create_answer(&self, csd_observer: &CreateSessionDescriptionObserver) {
        unsafe {
            pc::Rust_createAnswer(self.rffi.as_borrowed(), csd_observer.rffi().as_borrowed())
        };
    }

    /// Rust wrapper around C++ PeerConnection::SetRemoteDescription().
    pub fn set_remote_description(
        &self,
        ssd_observer: &SetSessionDescriptionObserver,
        session_description: SessionDescription,
    ) {
        // Rust_setRemoteDescription takes ownership of the local description
        // We take out the interface (with take_rffi) so that when the SessionDescriptionInterface
        // is deleted, we don't double delete.
        unsafe {
            pc::Rust_setRemoteDescription(
                self.rffi.as_borrowed(),
                ssd_observer.rffi().as_borrowed(),
                session_description.take_rffi().into_owned(),
            )
        };
    }

    /// Does something like:
    /// let sender = pc.get_audio_sender();
    /// sender.set_parameters({active: enabled});
    /// Which disables/enables the sending of any audio.
    /// Must be called *after* the answer has been set via
    /// set_remote_description or set_local_description.
    pub fn set_outgoing_media_enabled(&self, enabled: bool) {
        unsafe {
            pc::Rust_setOutgoingMediaEnabled(self.rffi.as_borrowed(), enabled);
        }
    }

    pub fn set_incoming_media_enabled(&self, enabled: bool) {
        unsafe {
            pc::Rust_setIncomingMediaEnabled(self.rffi.as_borrowed(), enabled);
        }
    }

    pub fn set_audio_playout_enabled(&self, enabled: bool) {
        unsafe { pc::Rust_setAudioPlayoutEnabled(self.rffi.as_borrowed(), enabled) };
    }

    pub fn set_audio_recording_enabled(&self, enabled: bool) {
        unsafe { pc::Rust_setAudioRecordingEnabled(self.rffi.as_borrowed(), enabled) };
    }

    /// Rust wrapper around C++ PeerConnection::AddIceCandidate().
    pub fn add_ice_candidate_from_sdp(&self, sdp: &str) -> Result<()> {
        info!("Remote ICE candidate: {}", redact_string(sdp));

        let sdp_c = CString::new(sdp)?;
        let add_ok = unsafe {
            pc::Rust_addIceCandidateFromSdp(
                self.rffi.as_borrowed(),
                webrtc::ptr::Borrowed::from_ptr(sdp_c.as_ptr()),
            )
        };
        if add_ok {
            Ok(())
        } else {
            Err(RingRtcError::AddIceCandidate.into())
        }
    }

    pub fn add_ice_candidate_from_server(
        &self,
        ip: std::net::IpAddr,
        port: u16,
        protocol: Protocol,
    ) -> Result<()> {
        let (tcp, hostname_c) = match protocol {
            Protocol::Udp => (false, None),
            Protocol::Tcp => (true, None),
            Protocol::Tls(hostname) => (true, Some(CString::new(hostname)?)),
        };

        let add_ok = unsafe {
            let hostname_ptr = hostname_c
                .as_ref()
                .map_or(webrtc::ptr::Borrowed::null(), |h| {
                    webrtc::ptr::Borrowed::from_ptr(h.as_ptr())
                });

            pc::Rust_addIceCandidateFromServer(
                self.rffi.as_borrowed(),
                ip.into(),
                port,
                tcp,
                hostname_ptr,
            )
        };

        if add_ok {
            Ok(())
        } else {
            Err(RingRtcError::AddIceCandidate.into())
        }
    }

    /// Rust wrapper around C++ PeerConnection::RemoveIceCandidates.
    pub fn remove_ice_candidates(&self, removed_addresses: impl Iterator<Item = SocketAddr>) {
        let removed_addresses: Vec<RffiIpPort> =
            removed_addresses.map(|address| address.into()).collect();

        unsafe {
            pc::Rust_removeIceCandidates(
                self.rffi.as_borrowed(),
                webrtc::ptr::Borrowed::from_ptr(removed_addresses.as_ptr()),
                removed_addresses.len(),
            )
        };
    }

    // Rust wrapper around C++ PeerConnection::CreateSharedIceGatherer().
    pub fn create_shared_ice_gatherer(&self) -> Result<IceGatherer> {
        let rffi_ice_gatherer = webrtc::Arc::from_owned(unsafe {
            pc::Rust_createSharedIceGatherer(self.rffi.as_borrowed())
        });
        if rffi_ice_gatherer.is_null() {
            return Err(RingRtcError::CreateIceGatherer.into());
        }

        let ice_gatherer = IceGatherer::new(rffi_ice_gatherer);

        Ok(ice_gatherer)
    }

    // Rust wrapper around C++ PeerConnection::UseSharedIceGatherer().
    pub fn use_shared_ice_gatherer(&self, ice_gatherer: &IceGatherer) -> Result<()> {
        let ok = unsafe {
            pc::Rust_useSharedIceGatherer(
                self.rffi.as_borrowed(),
                ice_gatherer.rffi().as_borrowed(),
            )
        };
        if ok {
            Ok(())
        } else {
            Err(RingRtcError::UseIceGatherer.into())
        }
    }

    // Rust wrapper around C++ PeerConnection::GetStats().
    pub fn get_stats(&self, stats_observer: &StatsObserver) -> Result<()> {
        unsafe { pc::Rust_getStats(self.rffi.as_borrowed(), stats_observer.rffi().as_borrowed()) };

        Ok(())
    }

    // Rust wrapper around C++ PeerConnection::SetBitrate().
    // The meaning is a bit complicated, but it's close to something like:
    // - If you don't set the min, you get a default min which is very low or 0.
    // - If you don't set the max, you get a default max which is high (2mbps or above).
    // - If you don't set the start, you keep it how it is.
    // - The whole thing is no-op unless you change something from the last set of values.
    pub fn set_send_rates(&self, rates: SendRates) -> Result<()> {
        let as_bps = |rate: Option<DataRate>| rate.map(|rate| rate.as_bps() as i32).unwrap_or(-1);
        unsafe {
            pc::Rust_setSendBitrates(
                self.rffi.as_borrowed(),
                as_bps(rates.min),
                as_bps(rates.start),
                as_bps(rates.max),
            )
        };

        Ok(())
    }

    // Warning: this blocks on the WebRTC network thread, so avoid calling it
    // while holding a lock, especially a lock also taken in a callback
    // from the network thread.
    pub fn send_rtp(&self, header: rtp::Header, payload: &[u8]) -> Result<()> {
        let rtp::Header {
            pt,
            seqnum,
            timestamp,
            ssrc,
        } = header;
        let ok = unsafe {
            pc::Rust_sendRtp(
                self.rffi.as_borrowed(),
                pt,
                seqnum,
                timestamp,
                ssrc,
                webrtc::ptr::Borrowed::from_ptr(payload.as_ptr()),
                payload.len(),
            )
        };
        if ok {
            Ok(())
        } else {
            Err(RingRtcError::SendRtp.into())
        }
    }

    // Must be called after either SetLocalDescription or SetRemoteDescription.
    // Received RTP with the matching PT will be sent to PeerConnectionObserver::handle_rtp_received.
    // Warning: this blocks on the WebRTC network thread, so avoid calling it while holding
    // a lock, especially a lock also taken in a callback from the network thread.
    pub fn receive_rtp(&self, pt: rtp::PayloadType, enable_incoming: bool) -> Result<()> {
        let ok = unsafe { pc::Rust_receiveRtp(self.rffi.as_borrowed(), pt, enable_incoming) };
        if ok {
            Ok(())
        } else {
            Err(RingRtcError::ReceiveRtp.into())
        }
    }

    pub fn configure_audio_encoders(&self, audio_encoder_config: &AudioEncoderConfig) {
        info!(
            "PeerConnection.configure_audio_encoders({:?})",
            audio_encoder_config
        );
        unsafe {
            pc::Rust_configureAudioEncoders(
                self.rffi.as_borrowed(),
                webrtc::ptr::Borrowed::from_ptr(&audio_encoder_config.rffi()),
            )
        };
    }

    pub fn get_audio_levels(&self) -> (AudioLevel, Vec<RffiReceivedAudioLevel>) {
        let captured_level: RffiAudioLevel = 0;
        let mut received_levels: Vec<RffiReceivedAudioLevel> = Vec::with_capacity(100);
        unsafe {
            let len = 0usize;
            pc::Rust_getAudioLevels(
                self.rffi.as_borrowed(),
                webrtc::ptr::Borrowed::from_ptr(&captured_level),
                webrtc::ptr::Borrowed::from_ptr(received_levels.as_mut_ptr()),
                received_levels.capacity(),
                webrtc::ptr::Borrowed::from_ptr(&len),
            );
            received_levels.set_len(len);
        }
        (captured_level, received_levels)
    }

    pub fn get_last_bandwidth_estimate(&self) -> DataRate {
        let bps = unsafe { pc::Rust_getLastBandwidthEstimateBps(self.rffi.as_borrowed()) };

        DataRate::from_bps(bps.into())
    }

    pub fn close(&self) {
        unsafe { pc::Rust_closePeerConnection(self.rffi.as_borrowed()) };
    }
}
