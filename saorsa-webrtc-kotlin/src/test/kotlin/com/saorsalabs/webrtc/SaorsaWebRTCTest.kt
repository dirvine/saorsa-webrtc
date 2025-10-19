package com.saorsalabs.webrtc

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class SaorsaWebRTCTest {
    
    // MARK: - Initialization Tests
    
    @Test
    fun `init with valid identity succeeds`() {
        val service = SaorsaWebRTC("alice")
        assertNotNull(service)
        service.close()
    }
    
    @Test
    fun `init with empty identity throws`() {
        assertThrows<IllegalArgumentException> {
            SaorsaWebRTC("")
        }
    }
    
    @Test
    fun `init with complex identity succeeds`() {
        val service = SaorsaWebRTC("alice-bob-charlie-david")
        assertNotNull(service)
        service.close()
    }
    
    // MARK: - Call Tests
    
    @Test
    fun `call with valid peer returns call ID`() {
        SaorsaWebRTC("alice").use { service ->
            val callId = service.call("bob")
            
            assertFalse(callId.isEmpty())
            assertTrue(callId.startsWith("call-"))
        }
    }
    
    @Test
    fun `call with empty peer throws`() {
        SaorsaWebRTC("alice").use { service ->
            assertThrows<IllegalArgumentException> {
                service.call("")
            }
        }
    }
    
    @Test
    fun `multiple calls return different IDs`() {
        SaorsaWebRTC("alice").use { service ->
            val callId1 = service.call("bob")
            val callId2 = service.call("charlie")
            
            assertNotEquals(callId1, callId2)
        }
    }
    
    // MARK: - Call State Tests
    
    @Test
    fun `get call state for active call succeeds`() {
        SaorsaWebRTC("alice").use { service ->
            val callId = service.call("bob")
            
            val state = service.getCallState(callId)
            
            // Should be connecting or active
            assertTrue(state == CallState.CONNECTING || state == CallState.ACTIVE)
        }
    }
    
    @Test
    fun `get call state with empty ID throws`() {
        SaorsaWebRTC("alice").use { service ->
            assertThrows<IllegalArgumentException> {
                service.getCallState("")
            }
        }
    }
    
    // MARK: - End Call Tests
    
    @Test
    fun `end call succeeds`() {
        SaorsaWebRTC("alice").use { service ->
            val callId = service.call("bob")
            
            // Should not throw
            service.endCall(callId)
        }
    }
    
    @Test
    fun `end call with empty ID throws`() {
        SaorsaWebRTC("alice").use { service ->
            assertThrows<IllegalArgumentException> {
                service.endCall("")
            }
        }
    }
    
    // MARK: - Call Lifecycle Tests
    
    @Test
    fun `full call lifecycle works`() {
        SaorsaWebRTC("alice").use { service ->
            // Initiate call
            val callId = service.call("bob")
            assertFalse(callId.isEmpty())
            
            // Check state
            val state = service.getCallState(callId)
            assertTrue(state == CallState.CONNECTING || state == CallState.ACTIVE)
            
            // End call
            service.endCall(callId)
        }
    }
    
    // MARK: - Error Type Tests
    
    @Test
    fun `error types can be distinguished`() {
        val error1 = SaorsaError.InvalidParameter()
        val error2 = SaorsaError.OutOfMemory()
        val error3 = SaorsaError.ConnectionFailed()
        
        assertTrue(error1 is SaorsaError.InvalidParameter)
        assertTrue(error2 is SaorsaError.OutOfMemory)
        assertTrue(error3 is SaorsaError.ConnectionFailed)
    }
    
    @Test
    fun `call state enum values are correct`() {
        assertEquals(CallState.CONNECTING, CallState.fromInt(0))
        assertEquals(CallState.ACTIVE, CallState.fromInt(1))
        assertEquals(CallState.ENDED, CallState.fromInt(2))
        assertEquals(CallState.FAILED, CallState.fromInt(3))
        
        assertNotEquals(CallState.CONNECTING, CallState.ACTIVE)
        assertNotEquals(CallState.ACTIVE, CallState.ENDED)
    }
    
    // MARK: - Resource Management Tests
    
    @Test
    fun `service can be closed multiple times safely`() {
        val service = SaorsaWebRTC("alice")
        service.close()
        service.close() // Should not crash
    }
    
    @Test
    fun `use block automatically closes service`() {
        var callId: String? = null
        
        SaorsaWebRTC("alice").use { service ->
            callId = service.call("bob")
        }
        
        // Service should be closed here
        assertNotNull(callId)
    }
}
