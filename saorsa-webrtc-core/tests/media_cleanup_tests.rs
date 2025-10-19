//! Media cleanup and resource management tests

use saorsa_webrtc_core::media::MediaStreamManager;
use saorsa_webrtc_core::types::MediaType;

#[tokio::test]
async fn media_track_remove_is_idempotent() {
    let mut mgr = MediaStreamManager::new();
    mgr.initialize().await.unwrap();

    let audio = mgr.create_audio_track().await.unwrap().clone();
    let video = mgr.create_video_track().await.unwrap().clone();

    assert_eq!(mgr.get_webrtc_tracks().len(), 2);
    assert_eq!(mgr.get_webrtc_tracks()[0].track_type, MediaType::Audio);
    assert_eq!(mgr.get_webrtc_tracks()[1].track_type, MediaType::Video);

    assert!(mgr.remove_track(&audio.id));
    assert!(mgr.remove_track(&video.id));

    assert!(!mgr.remove_track(&audio.id));
    assert!(!mgr.remove_track(&video.id));

    assert!(mgr.get_webrtc_tracks().is_empty());
}

#[tokio::test]
async fn media_manager_multiple_tracks_of_same_type() {
    let mut mgr = MediaStreamManager::new();
    mgr.initialize().await.unwrap();

    let audio1 = mgr.create_audio_track().await.unwrap().clone();
    let audio2 = mgr.create_audio_track().await.unwrap().clone();
    let video1 = mgr.create_video_track().await.unwrap().clone();

    assert_eq!(mgr.get_webrtc_tracks().len(), 3);

    assert_ne!(audio1.id, audio2.id);
    assert_ne!(audio1.id, video1.id);

    mgr.remove_track(&audio1.id);
    assert_eq!(mgr.get_webrtc_tracks().len(), 2);
}

#[tokio::test]
async fn media_manager_initialize_idempotent() {
    let mgr = MediaStreamManager::new();
    mgr.initialize().await.unwrap();
    mgr.initialize().await.unwrap();
}
