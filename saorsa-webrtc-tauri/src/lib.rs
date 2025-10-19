//! Tauri plugin for desktop integration

#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager,
    Runtime,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type CallMap = Arc<Mutex<HashMap<String, CallInfo>>>;

#[derive(Debug, Clone, serde::Serialize)]
struct CallInfo {
    call_id: String,
    peer: String,
    state: CallState,
}

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum CallState {
    Connecting,
    Active,
    Ended,
}

/// Initialize the WebRTC service
#[tauri::command]
async fn initialize(identity: String) -> Result<(), String> {
    if identity.is_empty() {
        return Err("Identity cannot be empty".to_string());
    }
    
    // In a full implementation, would initialize WebRTC service
    // For now, just validate the identity
    Ok(())
}

/// Initiate a call to a peer
#[tauri::command]
async fn call(
    state: tauri::State<'_, CallMap>,
    peer: String,
) -> Result<String, String> {
    if peer.is_empty() {
        return Err("Peer address cannot be empty".to_string());
    }
    
    // Generate call ID
    let call_id = format!("call-{}", uuid::Uuid::new_v4());
    
    // Store call info
    let call_info = CallInfo {
        call_id: call_id.clone(),
        peer,
        state: CallState::Connecting,
    };
    
    state.lock().await.insert(call_id.clone(), call_info);
    
    Ok(call_id)
}

/// Get the state of a call
#[tauri::command]
async fn get_call_state(
    state: tauri::State<'_, CallMap>,
    call_id: String,
) -> Result<CallState, String> {
    let calls = state.lock().await;
    
    calls.get(&call_id)
        .map(|info| info.state)
        .ok_or_else(|| "Call not found".to_string())
}

/// End a call
#[tauri::command]
async fn end_call(
    state: tauri::State<'_, CallMap>,
    call_id: String,
) -> Result<(), String> {
    let mut calls = state.lock().await;
    
    if let Some(call_info) = calls.get_mut(&call_id) {
        call_info.state = CallState::Ended;
        Ok(())
    } else {
        Err("Call not found".to_string())
    }
}

/// List all active calls
#[tauri::command]
async fn list_calls(
    state: tauri::State<'_, CallMap>,
) -> Result<Vec<CallInfo>, String> {
    let calls = state.lock().await;
    Ok(calls.values().cloned().collect())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let call_map: CallMap = Arc::new(Mutex::new(HashMap::new()));
    
    Builder::new("saorsa-webrtc")
        .invoke_handler(tauri::generate_handler![
            initialize,
            call,
            get_call_state,
            end_call,
            list_calls,
        ])
        .setup(move |app_handle| {
            app_handle.manage(call_map.clone());
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_with_valid_identity() {
        let result = initialize("alice".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_with_empty_identity() {
        let result = initialize("".to_string()).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_call_info_creation() {
        let info = CallInfo {
            call_id: "test-id".to_string(),
            peer: "bob".to_string(),
            state: CallState::Connecting,
        };
        
        assert_eq!(info.call_id, "test-id");
        assert_eq!(info.peer, "bob");
        assert_eq!(info.state, CallState::Connecting);
    }

    #[test]
    fn test_call_state_serialization() {
        // Test that call states can be serialized
        let connecting = CallState::Connecting;
        let active = CallState::Active;
        let ended = CallState::Ended;
        
        // Just ensure they can be compared
        assert_ne!(connecting, active);
        assert_ne!(active, ended);
        assert_ne!(connecting, ended);
    }
}
