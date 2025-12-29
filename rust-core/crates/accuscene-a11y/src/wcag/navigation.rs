//! Keyboard navigation support for WCAG compliance
//!
//! Implements WCAG 2.1 Success Criterion 2.1.1 (Keyboard)
//! and 2.1.2 (No Keyboard Trap)

use crate::error::{A11yError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Keyboard navigation patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NavigationPattern {
    /// Arrow keys navigation
    ArrowKeys,
    /// Tab key navigation
    Tab,
    /// Enter/Space activation
    Activation,
    /// Escape key dismissal
    Escape,
    /// Home/End navigation
    HomeEnd,
    /// Page Up/Down navigation
    PageUpDown,
    /// Custom pattern
    Custom,
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Tab,
    Enter,
    Space,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Backspace,
}

impl KeyCode {
    /// Convert from key string
    pub fn from_str(key: &str) -> Option<Self> {
        match key {
            "Tab" => Some(Self::Tab),
            "Enter" => Some(Self::Enter),
            " " | "Space" => Some(Self::Space),
            "Escape" | "Esc" => Some(Self::Escape),
            "ArrowUp" => Some(Self::ArrowUp),
            "ArrowDown" => Some(Self::ArrowDown),
            "ArrowLeft" => Some(Self::ArrowLeft),
            "ArrowRight" => Some(Self::ArrowRight),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "Delete" => Some(Self::Delete),
            "Backspace" => Some(Self::Backspace),
            _ => None,
        }
    }

    /// Convert to key string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tab => "Tab",
            Self::Enter => "Enter",
            Self::Space => "Space",
            Self::Escape => "Escape",
            Self::ArrowUp => "ArrowUp",
            Self::ArrowDown => "ArrowDown",
            Self::ArrowLeft => "ArrowLeft",
            Self::ArrowRight => "ArrowRight",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Delete => "Delete",
            Self::Backspace => "Backspace",
        }
    }
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub element_id: Option<String>,
}

/// Keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub description: String,
    pub action: String,
}

impl KeyboardShortcut {
    /// Create a new keyboard shortcut
    pub fn new(key: KeyCode, description: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::default(),
            description: description.into(),
            action: action.into(),
        }
    }

    /// Add Ctrl modifier
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers.ctrl = true;
        self
    }

    /// Add Alt modifier
    pub fn with_alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }

    /// Add Shift modifier
    pub fn with_shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }

    /// Get human-readable representation
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.meta {
            parts.push("Meta");
        }

        parts.push(self.key.as_str());
        parts.join("+")
    }
}

/// Navigation context for keyboard handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationContext {
    /// Current focused element
    pub focused_element: Option<String>,
    /// Focusable elements in order
    pub focusable_elements: Vec<String>,
    /// Skip links
    pub skip_links: Vec<SkipLink>,
    /// Active keyboard traps (should be empty)
    pub keyboard_traps: Vec<String>,
}

/// Skip link for navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLink {
    pub id: String,
    pub label: String,
    pub target: String,
}

/// Keyboard navigator
pub struct KeyboardNavigator {
    /// Registered shortcuts
    shortcuts: HashMap<String, KeyboardShortcut>,
    /// Supported navigation patterns
    patterns: HashSet<NavigationPattern>,
    /// Navigation history
    history: Vec<String>,
}

impl Default for KeyboardNavigator {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardNavigator {
    /// Create a new keyboard navigator
    pub fn new() -> Self {
        let mut navigator = Self {
            shortcuts: HashMap::new(),
            patterns: HashSet::new(),
            history: Vec::new(),
        };

        // Add default patterns
        navigator.patterns.insert(NavigationPattern::Tab);
        navigator.patterns.insert(NavigationPattern::ArrowKeys);
        navigator.patterns.insert(NavigationPattern::Activation);
        navigator.patterns.insert(NavigationPattern::Escape);

        navigator
    }

    /// Register a keyboard shortcut
    pub fn register_shortcut(&mut self, shortcut: KeyboardShortcut) {
        let key = format!("{:?}_{:?}", shortcut.key, shortcut.modifiers.ctrl);
        self.shortcuts.insert(key, shortcut);
    }

    /// Get all registered shortcuts
    pub fn shortcuts(&self) -> Vec<&KeyboardShortcut> {
        self.shortcuts.values().collect()
    }

    /// Enable a navigation pattern
    pub fn enable_pattern(&mut self, pattern: NavigationPattern) {
        self.patterns.insert(pattern);
    }

    /// Check if pattern is enabled
    pub fn is_pattern_enabled(&self, pattern: NavigationPattern) -> bool {
        self.patterns.contains(&pattern)
    }

    /// Handle keyboard event
    pub fn handle_event(&mut self, event: &KeyboardEvent, context: &NavigationContext) -> Result<NavigationAction> {
        // Check for keyboard traps
        if !context.keyboard_traps.is_empty() {
            return Err(A11yError::NavigationError(format!(
                "Keyboard trap detected: {:?}",
                context.keyboard_traps
            )));
        }

        // Handle Tab navigation
        if event.key == KeyCode::Tab {
            return self.handle_tab_navigation(event, context);
        }

        // Handle arrow key navigation
        if matches!(
            event.key,
            KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight
        ) {
            return self.handle_arrow_navigation(event, context);
        }

        // Handle activation keys
        if matches!(event.key, KeyCode::Enter | KeyCode::Space) {
            return Ok(NavigationAction::Activate);
        }

        // Handle Escape key
        if event.key == KeyCode::Escape {
            return Ok(NavigationAction::Dismiss);
        }

        Ok(NavigationAction::None)
    }

    /// Handle Tab key navigation
    fn handle_tab_navigation(
        &mut self,
        event: &KeyboardEvent,
        context: &NavigationContext,
    ) -> Result<NavigationAction> {
        if context.focusable_elements.is_empty() {
            return Ok(NavigationAction::None);
        }

        let current_index = context
            .focused_element
            .as_ref()
            .and_then(|id| context.focusable_elements.iter().position(|e| e == id))
            .unwrap_or(0);

        let next_index = if event.modifiers.shift {
            if current_index == 0 {
                context.focusable_elements.len() - 1
            } else {
                current_index - 1
            }
        } else {
            (current_index + 1) % context.focusable_elements.len()
        };

        let next_element = context.focusable_elements[next_index].clone();
        self.history.push(next_element.clone());

        Ok(NavigationAction::Focus(next_element))
    }

    /// Handle arrow key navigation
    fn handle_arrow_navigation(
        &mut self,
        event: &KeyboardEvent,
        context: &NavigationContext,
    ) -> Result<NavigationAction> {
        if !self.is_pattern_enabled(NavigationPattern::ArrowKeys) {
            return Ok(NavigationAction::None);
        }

        match event.key {
            KeyCode::ArrowUp | KeyCode::ArrowLeft => {
                self.navigate_to_previous(context)
            }
            KeyCode::ArrowDown | KeyCode::ArrowRight => {
                self.navigate_to_next(context)
            }
            _ => Ok(NavigationAction::None),
        }
    }

    /// Navigate to next element
    fn navigate_to_next(&mut self, context: &NavigationContext) -> Result<NavigationAction> {
        if context.focusable_elements.is_empty() {
            return Ok(NavigationAction::None);
        }

        let current_index = context
            .focused_element
            .as_ref()
            .and_then(|id| context.focusable_elements.iter().position(|e| e == id))
            .unwrap_or(0);

        let next_index = (current_index + 1) % context.focusable_elements.len();
        let next_element = context.focusable_elements[next_index].clone();

        self.history.push(next_element.clone());
        Ok(NavigationAction::Focus(next_element))
    }

    /// Navigate to previous element
    fn navigate_to_previous(&mut self, context: &NavigationContext) -> Result<NavigationAction> {
        if context.focusable_elements.is_empty() {
            return Ok(NavigationAction::None);
        }

        let current_index = context
            .focused_element
            .as_ref()
            .and_then(|id| context.focusable_elements.iter().position(|e| e == id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            context.focusable_elements.len() - 1
        } else {
            current_index - 1
        };

        let prev_element = context.focusable_elements[prev_index].clone();

        self.history.push(prev_element.clone());
        Ok(NavigationAction::Focus(prev_element))
    }

    /// Get navigation history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Navigation action result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationAction {
    /// Focus element with given ID
    Focus(String),
    /// Activate current element
    Activate,
    /// Dismiss/close current element
    Dismiss,
    /// No action
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcut() {
        let shortcut = KeyboardShortcut::new(
            KeyCode::Space,
            "Play/Pause",
            "toggle_playback"
        );

        assert_eq!(shortcut.to_string(), "Space");

        let shortcut = shortcut.with_ctrl();
        assert_eq!(shortcut.to_string(), "Ctrl+Space");
    }

    #[test]
    fn test_tab_navigation() {
        let mut navigator = KeyboardNavigator::new();
        let context = NavigationContext {
            focused_element: Some("element1".to_string()),
            focusable_elements: vec![
                "element1".to_string(),
                "element2".to_string(),
                "element3".to_string(),
            ],
            skip_links: vec![],
            keyboard_traps: vec![],
        };

        let event = KeyboardEvent {
            key: KeyCode::Tab,
            modifiers: KeyModifiers::default(),
            element_id: Some("element1".to_string()),
        };

        let action = navigator.handle_event(&event, &context).unwrap();
        assert_eq!(action, NavigationAction::Focus("element2".to_string()));
    }

    #[test]
    fn test_keyboard_trap_detection() {
        let mut navigator = KeyboardNavigator::new();
        let context = NavigationContext {
            focused_element: Some("element1".to_string()),
            focusable_elements: vec!["element1".to_string()],
            skip_links: vec![],
            keyboard_traps: vec!["element1".to_string()],
        };

        let event = KeyboardEvent {
            key: KeyCode::Tab,
            modifiers: KeyModifiers::default(),
            element_id: Some("element1".to_string()),
        };

        let result = navigator.handle_event(&event, &context);
        assert!(result.is_err());
    }
}
