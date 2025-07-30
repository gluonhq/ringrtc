package io.privacyresearch.tring;

import io.privacyresearch.tringapi.TringApi;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.util.Arrays;
import java.util.List;
import java.util.logging.Logger;

public class TringAppInterface {

    private static final Logger LOG = Logger.getLogger(TringAppInterface.class.getName());

    private final TringExecutorService executorService;
    private final TringApi tringApi;
    private final Arena scope;

    private long nativeCallEndpoint;

    private int clientId = -1;
    private byte[] localGroupId;
    private long activeCallId;

    public TringAppInterface(TringExecutorService executorService, TringApi tringApi, Arena scope) {
        this.executorService = executorService;
        this.tringApi = tringApi;
        this.scope = scope;
    }

    public void setNativeCallEndpoint(long nativeCallEndpoint) {
        LOG.info("[TringAppInterface::setNativeCallEndpoint] nativeCallEndpoint=" + nativeCallEndpoint);

        this.nativeCallEndpoint = nativeCallEndpoint;
    }

    public void setLocalGroupId(byte[] localGroupId) {
        this.localGroupId = localGroupId;
    }

    public byte[] getLocalGroupId() {
        return localGroupId;
    }

    public void setClientId(int clientId) {
        this.clientId = clientId;
    }

    public void setActiveCallId(long activeCallId) {
        this.activeCallId = activeCallId;
    }

    public long getActiveCallId() {
        return activeCallId;
    }

    public void destroy() {
        LOG.info("[TringAppInterface::destroy]");
    }

    public void groupConnectionStateChanged(int clientId, int connectionState) {
        LOG.info("[TringAppInterface::groupConnectionStateChanged] clientId=" + clientId + ", connectionState=" + connectionState);
    }

    public void groupEnded(int clientId, int reason) {
        LOG.info("[TringAppInterface::groupEnded] clientId=" + clientId + ", reason=" + reason);

        tringlib_h.deleteGroupCallClient(nativeCallEndpoint, clientId);
        this.clientId = -1;
    }

    public void groupJoinStateChanged(int clientId, int joinState) {
        LOG.info("[TringAppInterface::groupJoinStateChanged] clientId=" + clientId + ", joinState=" + joinState);

        if (joinState == 3) {
            LOG.info("We joined, now do a group call ring");
            tringlib_h.group_ring(nativeCallEndpoint, 1);
        }
    }

    public void groupRequestGroupMembers(int clientId) {
        LOG.info("[TringAppInterface::groupRequestGroupMembers] clientId=" + clientId);
        byte[] groupMemberInfo = tringApi.requestGroupMemberInfo(localGroupId);
        tringlib_h.setGroupMembers(nativeCallEndpoint, clientId, TringServiceImpl.toJByteArray(scope, groupMemberInfo));
    }

    public void groupRequestMembershipProof(int clientId) {
        LOG.info("[TringAppInterface::groupRequestMembershipProof] clientId=" + clientId);

        executorService.executeRequest(() -> {
            LOG.info("Requesting membership proof, groupid = " + Arrays.toString(TringAppInterface.this.localGroupId));
            byte[] groupMembershipToken = tringApi.requestGroupMembershipToken(TringAppInterface.this.localGroupId);
            tringlib_h.setMembershipProof(nativeCallEndpoint, clientId, TringServiceImpl.toJByteArray(scope, groupMembershipToken));
        });
    }

    public void groupRing(MemorySegment groupId, long ringId, MemorySegment senderId, int update) {
        byte[] bGroupId = TringServiceImpl.fromJArrayByte(groupId);
        byte[] bSenderId = TringServiceImpl.fromJArrayByte(senderId);

        LOG.info("[TringAppInterface::groupRing] groupId=" + Arrays.toString(bGroupId) + ", ringId=" + ringId + ", senderId=" + Arrays.toString(bSenderId) + ", update=" + update);

        this.localGroupId = bGroupId;
        tringApi.groupCallUpdateRing(bGroupId, ringId, bSenderId, update);
    }

    public void sendCallMessage(MemorySegment recipient, MemorySegment message, int urgency) {
        byte[] bRecipient = TringServiceImpl.fromJArrayByte(recipient);
        byte[] bMessage = TringServiceImpl.fromJArrayByte(message);

        LOG.info("[TringAppInterface::sendCallMessage] recipient=" + Arrays.toString(bRecipient) + ", urgency=" + urgency);

        tringApi.sendOpaqueCallMessage(bRecipient, bMessage, urgency);
    }

    public void sendCallMessageToGroup(MemorySegment groupId, MemorySegment message, int urgency) {
        byte[] bGroupId = TringServiceImpl.fromJArrayByte(groupId);
        byte[] bMessage = TringServiceImpl.fromJArrayByte(message);

        LOG.info("[TringAppInterface::sendCallMessageToGroup] groupId=" + Arrays.toString(bGroupId) + ", urgency=" + urgency);

        tringApi.sendOpaqueGroupCallMessage(bGroupId, bMessage, urgency);
    }

    public void signalingMessageAnswer(MemorySegment answer) {
        byte[] bAnswer = TringServiceImpl.fromJArrayByte(answer);

        LOG.info("[TringAppInterface::signalingMessageAnswer] answer=" + Arrays.toString(bAnswer));

        tringApi.answerCallback(bAnswer);

        acknowledgeSignalingMessageSent();
    }

    public void signalingMessageIce(MemorySegment ice) {
        byte[] bIce = TringServiceImpl.fromJArrayByte(ice);

        LOG.info("[TringAppInterface::signalingMessageIce] ice=" + Arrays.toString(bIce));

        tringApi.iceUpdateCallback(List.of(bIce));

        acknowledgeSignalingMessageSent();
    }

    public void signalingMessageOffer(MemorySegment offer) {
        byte[] bOffer = TringServiceImpl.fromJArrayByte(offer);

        LOG.info("[TringAppInterface::signalingMessageOffer] offer=" + Arrays.toString(bOffer));

        tringApi.offerCallback(bOffer);

        acknowledgeSignalingMessageSent();
    }

    // We need to inform ringrtc that we handled a message, so that it is ok
    // with sending the next message
    public void acknowledgeSignalingMessageSent() {
        LOG.info("Acknowledging signaling message sent");
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment callId = arena.allocateFrom(ValueLayout.JAVA_LONG, activeCallId);
            tringlib_h.signalMessageSent(nativeCallEndpoint, callId);
        }
        LOG.info("Acknowledging signaling message sent complete");
    }
}
