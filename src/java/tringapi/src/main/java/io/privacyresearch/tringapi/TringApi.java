package io.privacyresearch.tringapi;

import java.util.List;
import java.util.UUID;

/**
 *
 * @author johan
 */
public interface TringApi {
        
    void statusCallback(long callId, long peerId, int dir, int type);
    
    void answerCallback(byte[] opaque);

    void offerCallback(byte[] opaque);

    void iceUpdateCallback(List<byte[]> iceCandidates);

    void groupCallUpdateRing(byte[] groupId, long ringId, byte[] senderBytes, int status);
    // void getVideoFrame(int w, int h, byte[] raw);

    public void receivedGroupCallPeekForRingingCheck(PeekInfo peekInfo);

    public byte[] requestGroupMembershipToken(byte[] groupId);

    public byte[] requestGroupMemberInfo(byte[] groupId);

    public void sendOpaqueGroupCallMessage(byte[] groupIdentifier, byte[] opaque, int urgency);

    public void sendOpaqueCallMessage(byte[] recipientIdentifier, byte[] opaque, int urgency);

    public void updateRemoteDevices(List<Long> demuxIds);

}
