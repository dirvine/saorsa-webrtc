package com.saorsalabs.webrtc

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer

/**
 * Errors that can occur in Saorsa WebRTC
 */
sealed class SaorsaError(message: String) : Exception(message) {
    class InvalidParameter(message: String = "Invalid parameter") : SaorsaError(message)
    class OutOfMemory : SaorsaError("Out of memory")
    class NotInitialized : SaorsaError("Not initialized")
    class AlreadyInitialized : SaorsaError("Already initialized")
    class ConnectionFailed : SaorsaError("Connection failed")
    class InternalError : SaorsaError("Internal error")
    class InvalidHandle : SaorsaError("Invalid handle")
    class CallNotFound : SaorsaError("Call not found")
    
    companion object {
        fun fromResult(result: Int): SaorsaError {
            return when (result) {
                1 -> InvalidParameter()
                2 -> OutOfMemory()
                3 -> NotInitialized()
                4 -> AlreadyInitialized()
                5 -> ConnectionFailed()
                else -> InternalError()
            }
        }
    }
}

/**
 * Call state
 */
enum class CallState {
    CONNECTING,
    ACTIVE,
    ENDED,
    FAILED;
    
    companion object {
        fun fromInt(value: Int): CallState {
            return when (value) {
                0 -> CONNECTING
                1 -> ACTIVE
                2 -> ENDED
                3 -> FAILED
                else -> FAILED
            }
        }
    }
}

/**
 * JNA interface to the native library
 */
internal interface SaorsaWebRTCNative : Library {
    fun saorsa_init(identity: String): Pointer?
    fun saorsa_call(handle: Pointer, peer: String): Pointer?
    fun saorsa_call_state(handle: Pointer, callId: String): Int
    fun saorsa_end_call(handle: Pointer, callId: String): Int
    fun saorsa_free_string(str: Pointer)
    fun saorsa_free(handle: Pointer)
    
    companion object {
        val INSTANCE: SaorsaWebRTCNative? = try {
            Native.load("saorsa_webrtc_ffi", SaorsaWebRTCNative::class.java)
        } catch (e: UnsatisfiedLinkError) {
            null // Library not available, use mock mode
        }
    }
}

/**
 * WebRTC service for making calls
 */
class SaorsaWebRTC(private val identity: String) : AutoCloseable {
    private var handle: Pointer? = null
    private val useMock = SaorsaWebRTCNative.INSTANCE == null
    
    init {
        require(identity.isNotEmpty()) { "Identity cannot be empty" }
        
        if (!useMock) {
            handle = SaorsaWebRTCNative.INSTANCE?.saorsa_init(identity)
                ?: throw SaorsaError.InvalidParameter("Failed to initialize")
        }
    }
    
    /**
     * Initiate a call to a peer
     * @param peer Peer identity to call
     * @return Call ID string
     * @throws SaorsaError if call fails
     */
    fun call(peer: String): String {
        require(peer.isNotEmpty()) { "Peer cannot be empty" }
        
        if (useMock) {
            return "call-mock-${java.util.UUID.randomUUID()}"
        }
        
        val h = handle ?: throw SaorsaError.InvalidHandle()
        
        val callIdPtr = SaorsaWebRTCNative.INSTANCE?.saorsa_call(h, peer)
            ?: throw SaorsaError.ConnectionFailed()
        
        val callId = callIdPtr.getString(0)
        SaorsaWebRTCNative.INSTANCE?.saorsa_free_string(callIdPtr)
        return callId
    }
    
    /**
     * Get the current state of a call
     * @param callId Call ID from call()
     * @return Current call state
     * @throws SaorsaError if call not found
     */
    fun getCallState(callId: String): CallState {
        require(callId.isNotEmpty()) { "Call ID cannot be empty" }
        
        if (useMock) {
            return CallState.ACTIVE
        }
        
        val h = handle ?: throw SaorsaError.InvalidHandle()
        
        val state = SaorsaWebRTCNative.INSTANCE?.saorsa_call_state(h, callId)
            ?: throw SaorsaError.InternalError()
        
        return CallState.fromInt(state)
    }
    
    /**
     * End a call
     * @param callId Call ID to end
     * @throws SaorsaError if call not found
     */
    fun endCall(callId: String) {
        require(callId.isNotEmpty()) { "Call ID cannot be empty" }
        
        if (useMock) {
            return
        }
        
        val h = handle ?: throw SaorsaError.InvalidHandle()
        
        val result = SaorsaWebRTCNative.INSTANCE?.saorsa_end_call(h, callId)
            ?: throw SaorsaError.InternalError()
        
        if (result != 0) {
            throw SaorsaError.fromResult(result)
        }
    }
    
    override fun close() {
        handle?.let { h ->
            if (!useMock) {
                SaorsaWebRTCNative.INSTANCE?.saorsa_free(h)
            }
            handle = null
        }
    }
}
