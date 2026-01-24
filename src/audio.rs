//! Audio module for 2D games (placeholder for future implementation).
//!
//! This module provides components and types for audio playback.
//! Currently this is a skeleton that defines the API - actual audio
//! implementation will be added later using a backend like `kira` or `rodio`.
//!
//! # Example (Future API)
//!
//! ```ignore
//! use graviplex::audio::{AudioSource, AudioClip, AudioMixer};
//!
//! // Load audio clips
//! let jump_sound = audio.load_clip("assets/sounds/jump.ogg");
//! let background_music = audio.load_clip("assets/music/theme.ogg");
//!
//! // Play one-shot sound effect
//! audio.play(jump_sound);
//!
//! // Play looping background music
//! audio.play_music(background_music);
//!
//! // Attach audio to an entity (spatial audio)
//! world.spawn((
//!     Transform::from_position(Vec2::new(100.0, 0.0)),
//!     AudioSource::new(footsteps_clip).looping(),
//! ));
//! ```

use crate::core::math::Vec2;

// =============================================================================
// AUDIO CLIP
// =============================================================================

/// Handle to a loaded audio clip.
///
/// This is a lightweight handle that can be cheaply copied.
/// The actual audio data is stored in the AudioManager.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AudioClip(pub(crate) u32);

impl AudioClip {
    /// Invalid/null audio clip.
    pub const NONE: Self = Self(u32::MAX);

    /// Check if this is a valid clip.
    pub fn is_valid(&self) -> bool {
        self.0 != u32::MAX
    }
}

// =============================================================================
// AUDIO SOURCE COMPONENT
// =============================================================================

/// Settings for how audio should play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaybackSettings {
    /// Volume (0.0 = silent, 1.0 = full volume).
    pub volume: f32,
    /// Playback speed (1.0 = normal speed).
    pub speed: f32,
    /// Whether the sound should loop.
    pub looping: bool,
    /// Pan (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f32,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            looping: false,
            pan: 0.0,
        }
    }
}

impl PlaybackSettings {
    /// Create settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    /// Set the playback speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.1);
        self
    }

    /// Enable looping.
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    /// Set the stereo pan.
    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }
}

/// Audio source component for entities that emit sound.
///
/// Attach this to entities that should play sounds.
/// For 3D/spatial audio, the entity also needs a [`Transform`](crate::Transform).
///
/// # Example
///
/// ```ignore
/// // One-shot sound effect
/// world.spawn((
///     Transform::from_position(enemy_pos),
///     AudioSource::new(explosion_clip),
/// ));
///
/// // Looping ambient sound
/// world.spawn((
///     Transform::from_position(fire_pos),
///     AudioSource::new(fire_loop).looping(),
/// ));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AudioSource {
    /// The audio clip to play.
    pub clip: AudioClip,
    /// Playback settings.
    pub settings: PlaybackSettings,
    /// Whether to use spatial audio (position-based).
    pub spatial: bool,
    /// Range for spatial audio falloff.
    pub spatial_range: f32,
    /// Current playback state.
    pub state: PlaybackState,
}

/// Current state of audio playback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackState {
    /// Not playing.
    #[default]
    Stopped,
    /// Currently playing.
    Playing,
    /// Paused (can resume).
    Paused,
}

impl AudioSource {
    /// Create a new audio source with default settings.
    pub fn new(clip: AudioClip) -> Self {
        Self {
            clip,
            settings: PlaybackSettings::default(),
            spatial: false,
            spatial_range: 500.0,
            state: PlaybackState::Stopped,
        }
    }

    /// Set the volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.settings.volume = volume.clamp(0.0, 2.0);
        self
    }

    /// Enable looping.
    pub fn looping(mut self) -> Self {
        self.settings.looping = true;
        self
    }

    /// Enable spatial audio with default range.
    pub fn spatial(mut self) -> Self {
        self.spatial = true;
        self
    }

    /// Enable spatial audio with custom range.
    pub fn spatial_with_range(mut self, range: f32) -> Self {
        self.spatial = true;
        self.spatial_range = range;
        self
    }

    /// Start playing (marks for playback in next audio update).
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause playback.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    /// Resume paused playback.
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
        }
    }

    /// Stop playback.
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    /// Check if currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }
}

// =============================================================================
// AUDIO LISTENER COMPONENT
// =============================================================================

/// Audio listener component for receiving spatial audio.
///
/// Typically attached to the player or camera entity.
/// There should only be one active listener at a time.
///
/// # Example
///
/// ```ignore
/// // Player entity that hears spatial audio
/// world.spawn((
///     Transform::from_position(Vec2::ZERO),
///     Velocity::default(),
///     PlayerController::default(),
///     AudioListener::default(),
/// ));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AudioListener {
    /// Whether this listener is active.
    pub active: bool,
    /// Master volume multiplier.
    pub volume: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            active: true,
            volume: 1.0,
        }
    }
}

impl AudioListener {
    /// Create a new audio listener.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the master volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    /// Deactivate this listener.
    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }
}

// =============================================================================
// AUDIO MANAGER (Placeholder)
// =============================================================================

/// Audio manager for loading and playing sounds.
///
/// This is a placeholder API - actual implementation will be added
/// when an audio backend (kira/rodio) is integrated.
pub struct AudioManager {
    _master_volume: f32,
    _next_clip_id: u32,
}

impl AudioManager {
    /// Create a new audio manager.
    pub fn new() -> Self {
        Self {
            _master_volume: 1.0,
            _next_clip_id: 0,
        }
    }

    /// Load an audio clip from a file path.
    ///
    /// Returns [`AudioClip::NONE`] until audio backend is implemented.
    pub fn load_clip(&mut self, _path: &str) -> AudioClip {
        // TODO: Implement actual audio loading
        log::warn!("Audio not yet implemented - load_clip() returns placeholder");
        AudioClip::NONE
    }

    /// Play a one-shot sound effect.
    ///
    /// No-op until audio backend is implemented.
    pub fn play(&self, _clip: AudioClip) {
        // TODO: Implement actual audio playback
    }

    /// Play a one-shot sound with settings.
    pub fn play_with_settings(&self, _clip: AudioClip, _settings: PlaybackSettings) {
        // TODO: Implement actual audio playback
    }

    /// Play background music (replaces current music).
    pub fn play_music(&self, _clip: AudioClip) {
        // TODO: Implement music playback
    }

    /// Stop all currently playing sounds.
    pub fn stop_all(&self) {
        // TODO: Implement
    }

    /// Set master volume.
    pub fn set_master_volume(&mut self, volume: f32) {
        self._master_volume = volume.clamp(0.0, 2.0);
    }

    /// Get master volume.
    pub fn master_volume(&self) -> f32 {
        self._master_volume
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SPATIAL AUDIO UTILITIES
// =============================================================================

/// Calculate volume and pan for a sound source relative to a listener.
///
/// Returns (volume_multiplier, pan) where:
/// - volume_multiplier: 0.0 to 1.0 based on distance
/// - pan: -1.0 (left) to 1.0 (right)
pub fn calculate_spatial_audio(
    source_pos: Vec2,
    listener_pos: Vec2,
    max_range: f32,
) -> (f32, f32) {
    let diff = source_pos - listener_pos;
    let distance = diff.length();

    // Volume falloff (linear for now, could be exponential)
    let volume = if distance >= max_range {
        0.0
    } else {
        1.0 - (distance / max_range)
    };

    // Pan based on horizontal position
    let pan = if max_range > 0.0 {
        (diff.x / max_range).clamp(-1.0, 1.0)
    } else {
        0.0
    };

    (volume, pan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_clip_validity() {
        let valid = AudioClip(0);
        let invalid = AudioClip::NONE;

        assert!(valid.is_valid());
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_playback_settings() {
        let settings = PlaybackSettings::new()
            .with_volume(0.5)
            .with_speed(1.5)
            .looping()
            .with_pan(-0.5);

        assert_eq!(settings.volume, 0.5);
        assert_eq!(settings.speed, 1.5);
        assert!(settings.looping);
        assert_eq!(settings.pan, -0.5);
    }

    #[test]
    fn test_audio_source_state() {
        let mut source = AudioSource::new(AudioClip(0));
        assert!(!source.is_playing());

        source.play();
        assert!(source.is_playing());

        source.pause();
        assert_eq!(source.state, PlaybackState::Paused);

        source.resume();
        assert!(source.is_playing());

        source.stop();
        assert_eq!(source.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_spatial_audio_calculation() {
        let listener = Vec2::ZERO;
        let source = Vec2::new(250.0, 0.0); // Right of listener, half range
        let max_range = 500.0;

        let (volume, pan) = calculate_spatial_audio(source, listener, max_range);

        assert!((volume - 0.5).abs() < 0.01);
        assert!((pan - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_spatial_audio_at_max_range() {
        let listener = Vec2::ZERO;
        let source = Vec2::new(500.0, 0.0);
        let max_range = 500.0;

        let (volume, _) = calculate_spatial_audio(source, listener, max_range);
        assert_eq!(volume, 0.0);
    }
}
