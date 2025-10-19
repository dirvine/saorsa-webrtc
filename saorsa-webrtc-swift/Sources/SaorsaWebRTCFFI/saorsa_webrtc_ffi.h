#ifndef SAORSA_WEBRTC_FFI_H
#define SAORSA_WEBRTC_FFI_H

#include <stdint.h>

// Result codes
typedef enum {
    SAORSA_SUCCESS = 0,
    SAORSA_INVALID_PARAMETER = 1,
    SAORSA_OUT_OF_MEMORY = 2,
    SAORSA_NOT_INITIALIZED = 3,
    SAORSA_ALREADY_INITIALIZED = 4,
    SAORSA_CONNECTION_FAILED = 5,
    SAORSA_INTERNAL_ERROR = 99,
} SaorsaResult;

// Call states
typedef enum {
    CALL_STATE_CONNECTING = 0,
    CALL_STATE_ACTIVE = 1,
    CALL_STATE_ENDED = 2,
    CALL_STATE_FAILED = 3,
} CallState;

// Initialize the library
void* saorsa_init(const char* identity);

// Start a call
char* saorsa_call(void* handle, const char* peer);

// Get call state
CallState saorsa_call_state(void* handle, const char* call_id);

// End a call
SaorsaResult saorsa_end_call(void* handle, const char* call_id);

// Free a string
void saorsa_free_string(char* str);

// Free resources
void saorsa_free(void* handle);

#endif // SAORSA_WEBRTC_FFI_H
