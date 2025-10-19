import XCTest
@testable import SaorsaWebRTC

final class SaorsaWebRTCTests: XCTestCase {
    
    // MARK: - Initialization Tests
    
    func testInitWithValidIdentity() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        XCTAssertNotNil(service)
    }
    
    func testInitWithEmptyIdentityThrows() {
        XCTAssertThrowsError(try SaorsaWebRTC(identity: "")) { error in
            guard case SaorsaError.invalidParameter = error else {
                XCTFail("Expected invalidParameter error, got \(error)")
                return
            }
        }
    }
    
    func testInitWithComplexIdentity() throws {
        let service = try SaorsaWebRTC(identity: "alice-bob-charlie-david")
        XCTAssertNotNil(service)
    }
    
    // MARK: - Call Tests
    
    func testCallWithValidPeer() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        let callId = try service.call(peer: "bob")
        
        XCTAssertFalse(callId.isEmpty)
        XCTAssertTrue(callId.hasPrefix("call-"))
    }
    
    func testCallWithEmptyPeerThrows() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        
        XCTAssertThrowsError(try service.call(peer: "")) { error in
            guard case SaorsaError.invalidParameter = error else {
                XCTFail("Expected invalidParameter error, got \(error)")
                return
            }
        }
    }
    
    func testMultipleCallsReturnDifferentIds() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        
        let callId1 = try service.call(peer: "bob")
        let callId2 = try service.call(peer: "charlie")
        
        XCTAssertNotEqual(callId1, callId2)
    }
    
    // MARK: - Call State Tests
    
    func testGetCallStateForActiveCall() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        let callId = try service.call(peer: "bob")
        
        let state = try service.getCallState(callId: callId)
        
        // Should be connecting or active
        XCTAssertTrue(state == .connecting || state == .active)
    }
    
    func testGetCallStateWithEmptyIdThrows() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        
        XCTAssertThrowsError(try service.getCallState(callId: "")) { error in
            guard case SaorsaError.invalidParameter = error else {
                XCTFail("Expected invalidParameter error, got \(error)")
                return
            }
        }
    }
    
    // MARK: - End Call Tests
    
    func testEndCallSucceeds() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        let callId = try service.call(peer: "bob")
        
        // Should not throw
        try service.endCall(callId: callId)
    }
    
    func testEndCallWithEmptyIdThrows() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        
        XCTAssertThrowsError(try service.endCall(callId: "")) { error in
            guard case SaorsaError.invalidParameter = error else {
                XCTFail("Expected invalidParameter error, got \(error)")
                return
            }
        }
    }
    
    // MARK: - Call Lifecycle Tests
    
    func testFullCallLifecycle() throws {
        let service = try SaorsaWebRTC(identity: "alice")
        
        // Initiate call
        let callId = try service.call(peer: "bob")
        XCTAssertFalse(callId.isEmpty)
        
        // Check state
        let state = try service.getCallState(callId: callId)
        XCTAssertTrue(state == .connecting || state == .active)
        
        // End call
        try service.endCall(callId: callId)
    }
    
    // MARK: - Error Type Tests
    
    func testErrorEquality() {
        XCTAssertEqual(
            SaorsaError.invalidParameter("test"),
            SaorsaError.invalidParameter("test")
        )
        XCTAssertEqual(SaorsaError.outOfMemory, SaorsaError.outOfMemory)
        XCTAssertEqual(SaorsaError.notInitialized, SaorsaError.notInitialized)
    }
    
    func testCallStateEquality() {
        XCTAssertEqual(CallState.connecting, CallState.connecting)
        XCTAssertEqual(CallState.active, CallState.active)
        XCTAssertEqual(CallState.ended, CallState.ended)
        XCTAssertEqual(CallState.failed, CallState.failed)
        
        XCTAssertNotEqual(CallState.connecting, CallState.active)
        XCTAssertNotEqual(CallState.active, CallState.ended)
    }
    
    // MARK: - Cleanup Tests
    
    func testServiceCleanupDoesNotCrash() throws {
        var service: SaorsaWebRTC? = try SaorsaWebRTC(identity: "alice")
        service = nil // Should trigger deinit
        
        // If we get here without crashing, the test passes
        XCTAssertNil(service)
    }
}
