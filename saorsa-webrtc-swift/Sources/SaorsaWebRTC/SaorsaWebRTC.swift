import Foundation
#if canImport(SaorsaWebRTCFFI)
import SaorsaWebRTCFFI
#endif

/// Errors that can occur in Saorsa WebRTC
public enum SaorsaError: Error, Equatable {
    case invalidParameter(String)
    case outOfMemory
    case notInitialized
    case alreadyInitialized
    case connectionFailed
    case internalError
    case invalidHandle
    case callNotFound
    
    init(result: Int32) {
        switch result {
        case 1: self = .invalidParameter("Invalid parameter")
        case 2: self = .outOfMemory
        case 3: self = .notInitialized
        case 4: self = .alreadyInitialized
        case 5: self = .connectionFailed
        case 99: self = .internalError
        default: self = .internalError
        }
    }
}

/// Call state
public enum CallState: Equatable {
    case connecting
    case active
    case ended
    case failed
    
    #if canImport(SaorsaWebRTCFFI)
    init(rawValue: SaorsaWebRTCFFI.CallState) {
        switch rawValue {
        case CALL_STATE_CONNECTING: self = .connecting
        case CALL_STATE_ACTIVE: self = .active
        case CALL_STATE_ENDED: self = .ended
        case CALL_STATE_FAILED: self = .failed
        default: self = .failed
        }
    }
    #endif
}

/// WebRTC service for making calls
public final class SaorsaWebRTC {
    #if canImport(SaorsaWebRTCFFI)
    private var handle: UnsafeMutableRawPointer?
    #endif
    private let identity: String
    
    /// Initialize with an identity
    /// - Parameter identity: Four-word identity string
    /// - Throws: SaorsaError if initialization fails
    public init(identity: String) throws {
        guard !identity.isEmpty else {
            throw SaorsaError.invalidParameter("Identity cannot be empty")
        }
        
        self.identity = identity
        
        #if canImport(SaorsaWebRTCFFI)
        self.handle = saorsa_init(identity)
        
        if self.handle == nil {
            throw SaorsaError.invalidParameter("Failed to initialize")
        }
        #endif
    }
    
    deinit {
        #if canImport(SaorsaWebRTCFFI)
        if let handle = handle {
            saorsa_free(handle)
        }
        #endif
    }
    
    /// Initiate a call to a peer
    /// - Parameter peer: Peer identity to call
    /// - Returns: Call ID string
    /// - Throws: SaorsaError if call fails
    public func call(peer: String) throws -> String {
        guard !peer.isEmpty else {
            throw SaorsaError.invalidParameter("Peer cannot be empty")
        }
        
        #if canImport(SaorsaWebRTCFFI)
        guard let handle = handle else {
            throw SaorsaError.invalidHandle
        }
        
        guard let callIdPtr = saorsa_call(handle, peer) else {
            throw SaorsaError.connectionFailed
        }
        
        let callId = String(cString: callIdPtr)
        saorsa_free_string(callIdPtr)
        return callId
        #else
        // Mock for testing without FFI
        return "call-mock-\(UUID().uuidString)"
        #endif
    }
    
    /// Get the state of a call
    /// - Parameter callId: Call ID from call()
    /// - Returns: Current call state
    /// - Throws: SaorsaError if call not found
    public func getCallState(callId: String) throws -> CallState {
        guard !callId.isEmpty else {
            throw SaorsaError.invalidParameter("Call ID cannot be empty")
        }
        
        #if canImport(SaorsaWebRTCFFI)
        guard let handle = handle else {
            throw SaorsaError.invalidHandle
        }
        
        let state = saorsa_call_state(handle, callId)
        return CallState(rawValue: state)
        #else
        // Mock for testing
        return .active
        #endif
    }
    
    /// End a call
    /// - Parameter callId: Call ID to end
    /// - Throws: SaorsaError if call not found
    public func endCall(callId: String) throws {
        guard !callId.isEmpty else {
            throw SaorsaError.invalidParameter("Call ID cannot be empty")
        }
        
        #if canImport(SaorsaWebRTCFFI)
        guard let handle = handle else {
            throw SaorsaError.invalidHandle
        }
        
        let result = saorsa_end_call(handle, callId)
        if result != SAORSA_SUCCESS {
            throw SaorsaError(result: Int32(result.rawValue))
        }
        #endif
    }
}
