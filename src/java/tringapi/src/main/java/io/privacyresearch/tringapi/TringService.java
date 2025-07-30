 package io.privacyresearch.tringapi;

import java.util.List;

 /**
  * Implementations of this interface provides the access points for the application to interact with
  * RingRTC. 
  * Methods here are invoked by TringBridge, which is invoked by Equation.
  * When RingRTC wants to call back into the application, the TringApi interface
  * is used.
  * @author johan
  */
public interface TringService {

    public void setApi(TringApi api);
    public void setSelfUuid(byte[] aci);

    public void acceptCall();
    public void ignoreCall();
    public void hangupCall();

    public void proceed(long callId, String iceUser, String icePwd, String hostname, List<byte[]> ice);

    public void receivedIce(long callId, int senderDeviceId, List<byte[]> ice);

    public void receivedOffer(String peerId, long callId, int senderDeviceId, int receiverDeviceId,
            byte[] senderKey, byte[] receiverKey, byte[] opaque);

    public void receivedOpaqueMessage(byte[] senderUuid, int senderDeviceId, 
            int localDeviceId, byte[] opaque, long age);

    public void receivedAnswer(String peerId, long callId, int senderDeviceId,
            byte[] senderKey, byte[] receiverKey, byte[] opaque);
    public long startOutgoingCall(long callId, String peerId, int localDeviceId, boolean enableVideo);
    
    public default String getVersionInfo() {
        return "Unresolved TringService";
    }

    /**
     * Disable or enable outgoing audio.
     * @param enable true if we want to enable outgoing audio, false otherwise
     */
    public void enableOutgoingAudio(boolean enable);

    /**
     * Disable or enable outgoing video.
     * @param enable true if we want to enable outgoing video, false otherwise
     */
    public void enableOutgoingVideo(boolean enable);

    /**
     * Get a videoframe from the other side.
     * @param skip if true, ignore all old frames, and return the most recent one
     * @return a frame
     */
    public TringFrame getRemoteVideoFrame(long demuxId, boolean skip);

    public default TringFrame getRemoteVideoFrame(long demuxId) {
        return getRemoteVideoFrame(demuxId, false);
    }
    public void sendVideoFrame(int w, int h, int pixelFormat, byte[] raw);

    public void setArray();

    public void peekGroupCall(byte[] membershipProof, byte[] members);

    public int createGroupCallClient(byte[] groupId, String sfu, byte[] hkdf);

    public void joinGroupCall();

    public void setGroupBandWidth(int clientId, int bandwidthMode);

    public byte[] getCallLinkBytes(String url);

}
