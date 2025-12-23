//! Typewriter Text Animation System
//!
//! Provides text that appears character-by-character like a quill writing on parchment.

use bevy::prelude::*;

/// Component for typewriter-style text animation.
/// Reveals text character-by-character over time.
#[derive(Component, Debug, Clone)]
pub struct TypewriterText {
    /// The full text to reveal.
    pub full_text: String,
    /// Current number of characters revealed.
    pub chars_revealed: usize,
    /// Seconds per character.
    pub char_delay: f32,
    /// Time accumulator for character reveals.
    pub timer: f32,
    /// Whether the animation is complete.
    pub complete: bool,
}

impl TypewriterText {
    /// Creates a new typewriter text animation.
    /// 
    /// # Arguments
    /// * `text` - The full text to reveal
    /// * `char_delay` - Seconds between each character (0.05 = 20 chars/sec)
    pub fn new(text: impl Into<String>, char_delay: f32) -> Self {
        Self {
            full_text: text.into(),
            chars_revealed: 0,
            char_delay,
            timer: 0.0,
            complete: false,
        }
    }

    /// Creates a fast typewriter (for short UI text).
    pub fn fast(text: impl Into<String>) -> Self {
        Self::new(text, 0.03) // ~33 chars/sec
    }

    /// Creates a slow typewriter (for dramatic effect).
    pub fn slow(text: impl Into<String>) -> Self {
        Self::new(text, 0.08) // ~12 chars/sec
    }

    /// Returns the currently visible portion of the text.
    pub fn visible_text(&self) -> &str {
        let end = self.chars_revealed.min(self.full_text.len());
        // Handle multi-byte UTF-8 characters properly
        self.full_text
            .char_indices()
            .nth(end)
            .map(|(i, _)| &self.full_text[..i])
            .unwrap_or(&self.full_text)
    }

    /// Returns true if all characters have been revealed.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Instantly reveals all text (skip animation).
    pub fn skip(&mut self) {
        self.chars_revealed = self.full_text.chars().count();
        self.complete = true;
    }

    /// Resets the animation to start over.
    pub fn reset(&mut self) {
        self.chars_revealed = 0;
        self.timer = 0.0;
        self.complete = false;
    }

    /// Updates the animation with delta time.
    pub fn tick(&mut self, delta_seconds: f32) {
        if self.complete {
            return;
        }

        self.timer += delta_seconds;
        let total_chars = self.full_text.chars().count();

        while self.timer >= self.char_delay && self.chars_revealed < total_chars {
            self.timer -= self.char_delay;
            self.chars_revealed += 1;
        }

        if self.chars_revealed >= total_chars {
            self.complete = true;
        }
    }
}

/// System that updates all TypewriterText components.
pub fn typewriter_update_system(time: Res<Time>, mut query: Query<&mut TypewriterText>) {
    let delta = time.delta_secs();
    for mut typewriter in query.iter_mut() {
        typewriter.tick(delta);
    }
}

/// Resource for managing UI typewriter text (for egui integration).
/// Stores active typewriter animations keyed by a string ID.
#[derive(Resource, Default)]
pub struct TypewriterRegistry {
    texts: bevy::utils::HashMap<String, TypewriterText>,
}

impl TypewriterRegistry {
    /// Starts or gets a typewriter animation for the given key.
    /// If the text changed, restarts the animation.
    pub fn get_or_start(&mut self, key: &str, text: &str, char_delay: f32) -> &TypewriterText {
        let entry = self.texts.entry(key.to_string());
        entry.or_insert_with(|| TypewriterText::new(text, char_delay))
    }

    /// Gets the visible text for a key, or the full text if not animating.
    pub fn visible_text(&self, key: &str, full_text: &str) -> String {
        self.texts
            .get(key)
            .map(|t| t.visible_text().to_string())
            .unwrap_or_else(|| full_text.to_string())
    }

    /// Updates all registered typewriter animations.
    pub fn tick_all(&mut self, delta_seconds: f32) {
        for typewriter in self.texts.values_mut() {
            typewriter.tick(delta_seconds);
        }
    }

    /// Removes completed animations to free memory.
    pub fn cleanup_completed(&mut self) {
        self.texts.retain(|_, t| !t.is_complete());
    }
}

/// System that updates the TypewriterRegistry resource.
pub fn typewriter_registry_update_system(time: Res<Time>, mut registry: ResMut<TypewriterRegistry>) {
    registry.tick_all(time.delta_secs());
}
