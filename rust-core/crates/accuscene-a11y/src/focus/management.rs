//! Focus management system for accessibility
//!
//! Implements WCAG 2.1 Success Criterion 2.4.3 (Focus Order)
//! and 2.4.7 (Focus Visible)

use crate::error::{A11yError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Focus node in the focus tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusNode {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Tab index
    pub tab_index: i32,
    /// Whether node is focusable
    pub focusable: bool,
    /// Whether node is currently focused
    pub focused: bool,
    /// Parent node ID
    pub parent: Option<String>,
    /// Child node IDs
    pub children: Vec<String>,
    /// Custom data
    pub data: HashMap<String, String>,
}

impl FocusNode {
    /// Create a new focus node
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tab_index: 0,
            focusable: true,
            focused: false,
            parent: None,
            children: Vec::new(),
            data: HashMap::new(),
        }
    }

    /// Set tab index
    pub fn with_tab_index(mut self, index: i32) -> Self {
        self.tab_index = index;
        self
    }

    /// Set focusable state
    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Add child node
    pub fn add_child(&mut self, child_id: String) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove child node
    pub fn remove_child(&mut self, child_id: &str) {
        self.children.retain(|id| id != child_id);
    }
}

/// Focus manager
pub struct FocusManager {
    /// All focus nodes by ID
    nodes: HashMap<String, FocusNode>,
    /// Currently focused node ID
    current_focus: Option<String>,
    /// Focus history
    history: VecDeque<String>,
    /// Maximum history size
    max_history: usize,
    /// Active focus traps
    active_traps: Vec<String>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            current_focus: None,
            history: VecDeque::new(),
            max_history: 50,
            active_traps: Vec::new(),
        }
    }

    /// Register a focus node
    pub fn register_node(&mut self, node: FocusNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Unregister a focus node
    pub fn unregister_node(&mut self, id: &str) -> Option<FocusNode> {
        // Remove from parent's children
        if let Some(node) = self.nodes.get(id) {
            if let Some(parent_id) = &node.parent {
                if let Some(parent) = self.nodes.get_mut(parent_id) {
                    parent.remove_child(id);
                }
            }
        }

        // Remove node
        self.nodes.remove(id)
    }

    /// Get a focus node
    pub fn get_node(&self, id: &str) -> Option<&FocusNode> {
        self.nodes.get(id)
    }

    /// Get a mutable focus node
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut FocusNode> {
        self.nodes.get_mut(id)
    }

    /// Set focus to a node
    pub fn focus(&mut self, id: &str) -> Result<()> {
        // Check if node exists and is focusable
        let node = self.nodes.get(id).ok_or_else(|| {
            A11yError::FocusError(format!("Node not found: {}", id))
        })?;

        if !node.focusable {
            return Err(A11yError::FocusError(format!(
                "Node is not focusable: {}",
                id
            )));
        }

        // Check focus trap constraints
        if !self.active_traps.is_empty() {
            let is_in_trap = self.is_node_in_trap(id);
            if !is_in_trap {
                return Err(A11yError::FocusError(
                    "Cannot focus outside active focus trap".to_string(),
                ));
            }
        }

        // Unfocus current node
        if let Some(current_id) = &self.current_focus {
            if let Some(current_node) = self.nodes.get_mut(current_id) {
                current_node.focused = false;
            }
        }

        // Focus new node
        if let Some(new_node) = self.nodes.get_mut(id) {
            new_node.focused = true;
        }

        // Update focus and history
        self.current_focus = Some(id.to_string());
        self.add_to_history(id);

        Ok(())
    }

    /// Blur (unfocus) current node
    pub fn blur(&mut self) -> Result<()> {
        if let Some(current_id) = &self.current_focus {
            if let Some(node) = self.nodes.get_mut(current_id) {
                node.focused = false;
            }
            self.current_focus = None;
        }
        Ok(())
    }

    /// Get currently focused node
    pub fn get_focused(&self) -> Option<&FocusNode> {
        self.current_focus
            .as_ref()
            .and_then(|id| self.nodes.get(id))
    }

    /// Move focus to next focusable node
    pub fn focus_next(&mut self) -> Result<()> {
        let focusable = self.get_focusable_nodes();
        if focusable.is_empty() {
            return Ok(());
        }

        let current_index = self
            .current_focus
            .as_ref()
            .and_then(|id| focusable.iter().position(|n| n.id == *id))
            .unwrap_or(0);

        let next_index = (current_index + 1) % focusable.len();
        let next_id = focusable[next_index].id.clone();

        self.focus(&next_id)
    }

    /// Move focus to previous focusable node
    pub fn focus_previous(&mut self) -> Result<()> {
        let focusable = self.get_focusable_nodes();
        if focusable.is_empty() {
            return Ok(());
        }

        let current_index = self
            .current_focus
            .as_ref()
            .and_then(|id| focusable.iter().position(|n| n.id == *id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            focusable.len() - 1
        } else {
            current_index - 1
        };

        let prev_id = focusable[prev_index].id.clone();
        self.focus(&prev_id)
    }

    /// Restore focus to previous node
    pub fn restore_focus(&mut self) -> Result<()> {
        if let Some(prev_id) = self.history.iter().rev().nth(1) {
            self.focus(prev_id)
        } else {
            Ok(())
        }
    }

    /// Get all focusable nodes in tab order
    fn get_focusable_nodes(&self) -> Vec<&FocusNode> {
        let mut nodes: Vec<&FocusNode> = self
            .nodes
            .values()
            .filter(|n| n.focusable)
            .collect();

        nodes.sort_by_key(|n| n.tab_index);
        nodes
    }

    /// Add to focus history
    fn add_to_history(&mut self, id: &str) {
        self.history.push_back(id.to_string());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Create a focus trap
    pub fn create_trap(&mut self, trap_id: String) {
        if !self.active_traps.contains(&trap_id) {
            self.active_traps.push(trap_id);
        }
    }

    /// Release a focus trap
    pub fn release_trap(&mut self, trap_id: &str) {
        self.active_traps.retain(|id| id != trap_id);
    }

    /// Check if node is within an active trap
    fn is_node_in_trap(&self, node_id: &str) -> bool {
        // For simplicity, check if node_id starts with any trap_id
        self.active_traps.iter().any(|trap_id| node_id.starts_with(trap_id))
    }

    /// Get focus history
    pub fn history(&self) -> &VecDeque<String> {
        &self.history
    }

    /// Clear focus history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Focus trap for modals and dialogs
pub struct FocusTrap {
    /// Trap identifier
    pub id: String,
    /// Container node ID
    pub container_id: String,
    /// Focusable elements within trap
    pub focusable_elements: Vec<String>,
    /// Element that had focus before trap activated
    pub restore_element: Option<String>,
}

impl FocusTrap {
    /// Create a new focus trap
    pub fn new(id: impl Into<String>, container_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            container_id: container_id.into(),
            focusable_elements: Vec::new(),
            restore_element: None,
        }
    }

    /// Add focusable element to trap
    pub fn add_element(&mut self, element_id: String) {
        if !self.focusable_elements.contains(&element_id) {
            self.focusable_elements.push(element_id);
        }
    }

    /// Activate the trap
    pub fn activate(&mut self, manager: &mut FocusManager, restore_to: Option<String>) -> Result<()> {
        self.restore_element = restore_to.or_else(|| {
            manager.get_focused().map(|n| n.id.clone())
        });

        manager.create_trap(self.id.clone());

        // Focus first element in trap
        if let Some(first_id) = self.focusable_elements.first() {
            manager.focus(first_id)?;
        }

        Ok(())
    }

    /// Deactivate the trap
    pub fn deactivate(&mut self, manager: &mut FocusManager) -> Result<()> {
        manager.release_trap(&self.id);

        // Restore previous focus
        if let Some(restore_id) = &self.restore_element {
            manager.focus(restore_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager() {
        let mut manager = FocusManager::new();

        let node1 = FocusNode::new("node1", "First Node");
        let node2 = FocusNode::new("node2", "Second Node");

        manager.register_node(node1);
        manager.register_node(node2);

        manager.focus("node1").unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "node1");

        manager.focus("node2").unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "node2");
    }

    #[test]
    fn test_focus_navigation() {
        let mut manager = FocusManager::new();

        manager.register_node(FocusNode::new("node1", "Node 1").with_tab_index(0));
        manager.register_node(FocusNode::new("node2", "Node 2").with_tab_index(1));
        manager.register_node(FocusNode::new("node3", "Node 3").with_tab_index(2));

        manager.focus("node1").unwrap();
        manager.focus_next().unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "node2");

        manager.focus_next().unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "node3");

        manager.focus_previous().unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "node2");
    }

    #[test]
    fn test_focus_trap() {
        let mut manager = FocusManager::new();
        manager.register_node(FocusNode::new("outside", "Outside"));
        manager.register_node(FocusNode::new("trap_inside", "Inside Trap"));

        let mut trap = FocusTrap::new("trap", "trap");
        trap.add_element("trap_inside".to_string());

        manager.focus("outside").unwrap();
        trap.activate(&mut manager, None).unwrap();

        // Should not be able to focus outside trap
        let result = manager.focus("outside");
        assert!(result.is_err());

        trap.deactivate(&mut manager).unwrap();

        // Should now be able to focus outside
        manager.focus("outside").unwrap();
        assert_eq!(manager.get_focused().unwrap().id, "outside");
    }
}
