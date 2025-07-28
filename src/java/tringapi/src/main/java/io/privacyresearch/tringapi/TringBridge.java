package io.privacyresearch.tringapi;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import java.util.ServiceLoader;
import java.util.logging.Logger;
import java.util.stream.Collectors;

 /**
  * This class provides the access points for the application to interact with
  * RingRTC. 
  * Methods here are invoked by Equation.
  * When RingRTC wants to call back into the application, the TringApi interface
  * is used.
  * @author johan
  */
public class TringBridge {
    
    private TringService service;
    private static final Logger LOG = Logger.getLogger(TringBridge.class.getName());

    public TringBridge(final TringApi api, final byte[] uuid) {
        ServiceLoader<TringService> loader = ServiceLoader.load(TringService.class);
        Optional<TringService> serviceOpt = loader.findFirst();
        serviceOpt.ifPresentOrElse(s -> {
            this.service = s;
            this.service.setApi(api);
            this.service.setSelfUuid(uuid);
        }, () -> {
            LOG.warning("No tring service!");
        });

    }

    public String getVersionInfo() {
        if (service == null) {
            return "No TringService registered";
        } else {
            return service.getVersionInfo();
        }
    }

    public void acceptCall() {
        service.acceptCall();
    }

    public void ignoreCall() {
        service.ignoreCall();
    }

    public void hangupCall() {
        service.hangupCall();
    }

    public void proceed(long callId, String iceUser, String icePassword, String hostname, List<String> ice) {
        List<byte[]> iceb = ice.stream().map(s -> s.getBytes(StandardCharsets.UTF_8)).collect(Collectors.toList());
        service.proceed(callId, iceUser, icePassword, hostname, iceb);
    }

    public void receivedIce(long callId, int senderDeviceId, List<byte[]> ice) {
        service.receivedIce(callId, senderDeviceId, ice);
    }

    public void receivedOffer(String peerId, long callId, int senderDeviceId, int receiverDeviceId,
            byte[] senderKey, byte[] receiverKey, byte[] opaque) {
        service.receivedOffer(peerId, callId, senderDeviceId, receiverDeviceId, senderKey, receiverKey, opaque);
    }

    public void receivedOpaqueMessage(byte[] uuid, int senderDeviceId, int receiverDeviceId,
            byte[] opaque, long ageMessage) {
        service.receivedOpaqueMessage(uuid, senderDeviceId, receiverDeviceId, opaque, ageMessage);
    }

    public void receivedAnswer(String peerId, long callId, int receiverDeviceId,
            byte[] senderKey, byte[] receiverKey, byte[] opaque) {
        service.receivedAnswer(peerId, callId, receiverDeviceId, senderKey, receiverKey, opaque);
    }

    public long startOutgoingCall(long callId, String peerId, int localDeviceId, boolean enableVideo) {
        return service.startOutgoingCall(callId, peerId, localDeviceId, enableVideo);
    }

    public void enableOutgoingAudio(boolean enable) {
        service.enableOutgoingAudio(enable);
    }

    public void enableOutgoingVideo(boolean enable) {
        service.enableOutgoingVideo(enable);
    }

    public TringFrame getRemoteVideoFrame(long demuxId) {
        return service.getRemoteVideoFrame(demuxId);
    }  

    public void sendVideoFrame(int width, int height, int pixelFormat, byte[] raw) {
        service.sendVideoFrame(width, height, pixelFormat, raw);
    }

    public void setArray() {
        service.setArray();
    }

    public void peekGroupCall(byte[] membershipProof, byte[] members) {
        service.peekGroupCall(membershipProof, members);
    }

    public int createGroupCallClient(byte[] groupId, String sfu, byte[] hkdf) {
        return service.createGroupCallClient(groupId, sfu, hkdf);
    }

    public void joinGroupCall() {
        service.joinGroupCall();
    }

    public void setGroupBandWidth(int clientId, int bandwidthMode) {
        service.setGroupBandWidth(clientId, bandwidthMode);
    }

    public byte[] getCallLinkBytes(String url) {
        System.err.println("[TB] getCallLinkBytes for "+url);
        return service.getCallLinkBytes(url);
    }
}