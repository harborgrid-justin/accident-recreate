//! Responsive layout engine
//!
//! Manages widget positioning and responsive layout across different breakpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use crate::config::Breakpoint;
use crate::error::{DashboardError, DashboardResult, LayoutError, LayoutResult};

/// Widget position and size in the grid
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate)]
pub struct WidgetLayout {
    /// Widget unique identifier
    pub widget_id: String,

    /// Column position (0-indexed)
    #[validate(range(min = 0))]
    pub x: u32,

    /// Row position (0-indexed)
    #[validate(range(min = 0))]
    pub y: u32,

    /// Width in columns
    #[validate(range(min = 1, max = 24))]
    pub width: u32,

    /// Height in rows
    #[validate(range(min = 1, max = 100))]
    pub height: u32,

    /// Minimum width constraint
    #[validate(range(min = 1, max = 24))]
    pub min_width: Option<u32>,

    /// Maximum width constraint
    #[validate(range(min = 1, max = 24))]
    pub max_width: Option<u32>,

    /// Minimum height constraint
    #[validate(range(min = 1, max = 100))]
    pub min_height: Option<u32>,

    /// Maximum height constraint
    #[validate(range(min = 1, max = 100))]
    pub max_height: Option<u32>,

    /// Whether widget is static (cannot be moved)
    pub is_static: bool,

    /// Whether widget can be resized
    pub is_resizable: bool,

    /// Z-index for layering
    pub z_index: i32,
}

impl WidgetLayout {
    /// Create a new widget layout
    pub fn new(widget_id: String, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            widget_id,
            x,
            y,
            width,
            height,
            min_width: Some(1),
            max_width: None,
            min_height: Some(1),
            max_height: None,
            is_static: false,
            is_resizable: true,
            z_index: 0,
        }
    }

    /// Check if this layout overlaps with another
    pub fn overlaps_with(&self, other: &WidgetLayout) -> bool {
        !(self.x >= other.x + other.width
            || self.x + self.width <= other.x
            || self.y >= other.y + other.height
            || self.y + self.height <= other.y)
    }

    /// Check if layout is within grid bounds
    pub fn is_within_bounds(&self, columns: u32, rows: Option<u32>) -> bool {
        let within_columns = self.x + self.width <= columns;
        let within_rows = rows.map_or(true, |r| self.y + self.height <= r);
        within_columns && within_rows
    }

    /// Validate constraints
    pub fn validate_constraints(&self) -> LayoutResult<()> {
        if let Some(min_w) = self.min_width {
            if self.width < min_w {
                return Err(LayoutError::InvalidGrid(
                    format!("Width {} is less than minimum {}", self.width, min_w)
                ));
            }
        }

        if let Some(max_w) = self.max_width {
            if self.width > max_w {
                return Err(LayoutError::InvalidGrid(
                    format!("Width {} exceeds maximum {}", self.width, max_w)
                ));
            }
        }

        if let Some(min_h) = self.min_height {
            if self.height < min_h {
                return Err(LayoutError::InvalidGrid(
                    format!("Height {} is less than minimum {}", self.height, min_h)
                ));
            }
        }

        if let Some(max_h) = self.max_height {
            if self.height > max_h {
                return Err(LayoutError::InvalidGrid(
                    format!("Height {} exceeds maximum {}", self.height, max_h)
                ));
            }
        }

        Ok(())
    }

    /// Clamp dimensions to constraints
    pub fn clamp_to_constraints(&mut self) {
        if let Some(min_w) = self.min_width {
            self.width = self.width.max(min_w);
        }
        if let Some(max_w) = self.max_width {
            self.width = self.width.min(max_w);
        }
        if let Some(min_h) = self.min_height {
            self.height = self.height.max(min_h);
        }
        if let Some(max_h) = self.max_height {
            self.height = self.height.min(max_h);
        }
    }
}

/// Layout configuration for a specific breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointLayout {
    /// Breakpoint identifier
    pub breakpoint: Breakpoint,

    /// Widget layouts for this breakpoint
    pub widgets: Vec<WidgetLayout>,

    /// Number of columns in grid
    pub columns: u32,

    /// Auto-compact enabled (moves widgets up to fill gaps)
    pub auto_compact: bool,
}

impl BreakpointLayout {
    /// Create a new breakpoint layout
    pub fn new(breakpoint: Breakpoint, columns: u32) -> Self {
        Self {
            breakpoint,
            widgets: Vec::new(),
            columns,
            auto_compact: true,
        }
    }

    /// Add a widget layout
    pub fn add_widget(&mut self, layout: WidgetLayout) -> LayoutResult<()> {
        // Validate constraints
        layout.validate_constraints()?;

        // Check bounds
        if !layout.is_within_bounds(self.columns, None) {
            return Err(LayoutError::OutOfBounds {
                x: layout.x,
                y: layout.y,
                max_x: self.columns,
                max_y: u32::MAX,
            });
        }

        // Check for overlaps
        for existing in &self.widgets {
            if layout.overlaps_with(existing) {
                return Err(LayoutError::PositionConflict {
                    x: layout.x,
                    y: layout.y,
                });
            }
        }

        self.widgets.push(layout);

        if self.auto_compact {
            self.compact();
        }

        Ok(())
    }

    /// Remove a widget by ID
    pub fn remove_widget(&mut self, widget_id: &str) -> Option<WidgetLayout> {
        if let Some(index) = self.widgets.iter().position(|w| w.widget_id == widget_id) {
            let removed = self.widgets.remove(index);
            if self.auto_compact {
                self.compact();
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Update widget position
    pub fn update_widget(&mut self, widget_id: &str, x: u32, y: u32) -> LayoutResult<()> {
        let index = self.widgets
            .iter()
            .position(|w| w.widget_id == widget_id)
            .ok_or_else(|| LayoutError::InvalidGrid(format!("Widget not found: {}", widget_id)))?;

        let mut updated = self.widgets[index].clone();
        updated.x = x;
        updated.y = y;

        // Validate new position
        if !updated.is_within_bounds(self.columns, None) {
            return Err(LayoutError::OutOfBounds {
                x,
                y,
                max_x: self.columns,
                max_y: u32::MAX,
            });
        }

        // Check for overlaps with other widgets
        for (i, existing) in self.widgets.iter().enumerate() {
            if i != index && updated.overlaps_with(existing) {
                return Err(LayoutError::PositionConflict { x, y });
            }
        }

        self.widgets[index] = updated;
        Ok(())
    }

    /// Compact layout by moving widgets up to fill gaps
    pub fn compact(&mut self) {
        if self.widgets.is_empty() {
            return;
        }

        // Sort widgets by y position, then x position
        self.widgets.sort_by(|a, b| {
            a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x))
        });

        // Try to move each widget up as much as possible
        for i in 0..self.widgets.len() {
            if self.widgets[i].is_static {
                continue;
            }

            let mut new_y = 0;
            let widget = &self.widgets[i];
            let test_layout = WidgetLayout {
                widget_id: widget.widget_id.clone(),
                x: widget.x,
                y: new_y,
                width: widget.width,
                height: widget.height,
                min_width: widget.min_width,
                max_width: widget.max_width,
                min_height: widget.min_height,
                max_height: widget.max_height,
                is_static: widget.is_static,
                is_resizable: widget.is_resizable,
                z_index: widget.z_index,
            };

            // Find the highest position without overlaps
            while new_y < widget.y {
                let mut test = test_layout.clone();
                test.y = new_y;

                let has_overlap = self.widgets.iter()
                    .enumerate()
                    .any(|(j, other)| j != i && test.overlaps_with(other));

                if !has_overlap {
                    break;
                }
                new_y += 1;
            }

            self.widgets[i].y = new_y;
        }
    }

    /// Get total height of the layout
    pub fn total_height(&self) -> u32 {
        self.widgets
            .iter()
            .map(|w| w.y + w.height)
            .max()
            .unwrap_or(0)
    }

    /// Get widget by ID
    pub fn get_widget(&self, widget_id: &str) -> Option<&WidgetLayout> {
        self.widgets.iter().find(|w| w.widget_id == widget_id)
    }
}

/// Responsive layout manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveLayout {
    /// Dashboard ID
    pub dashboard_id: String,

    /// Layouts per breakpoint
    pub breakpoint_layouts: HashMap<Breakpoint, BreakpointLayout>,

    /// Current active breakpoint
    pub current_breakpoint: Breakpoint,
}

impl ResponsiveLayout {
    /// Create a new responsive layout
    pub fn new(dashboard_id: String) -> Self {
        let mut breakpoint_layouts = HashMap::new();

        // Initialize layouts for all breakpoints
        for breakpoint in &[
            Breakpoint::Mobile,
            Breakpoint::MobileLandscape,
            Breakpoint::Tablet,
            Breakpoint::Desktop,
            Breakpoint::DesktopLarge,
            Breakpoint::DesktopXL,
        ] {
            let columns = breakpoint.default_columns();
            breakpoint_layouts.insert(*breakpoint, BreakpointLayout::new(*breakpoint, columns));
        }

        Self {
            dashboard_id,
            breakpoint_layouts,
            current_breakpoint: Breakpoint::Desktop,
        }
    }

    /// Get layout for specific breakpoint
    pub fn get_layout(&self, breakpoint: Breakpoint) -> Option<&BreakpointLayout> {
        self.breakpoint_layouts.get(&breakpoint)
    }

    /// Get mutable layout for specific breakpoint
    pub fn get_layout_mut(&mut self, breakpoint: Breakpoint) -> Option<&mut BreakpointLayout> {
        self.breakpoint_layouts.get_mut(&breakpoint)
    }

    /// Get current active layout
    pub fn current_layout(&self) -> Option<&BreakpointLayout> {
        self.get_layout(self.current_breakpoint)
    }

    /// Set current breakpoint
    pub fn set_breakpoint(&mut self, breakpoint: Breakpoint) {
        self.current_breakpoint = breakpoint;
    }

    /// Add widget to all breakpoints with adaptive sizing
    pub fn add_widget_responsive(&mut self, widget_id: String, base_layout: WidgetLayout) -> DashboardResult<()> {
        for (breakpoint, layout) in &mut self.breakpoint_layouts {
            let mut adapted = base_layout.clone();
            adapted.widget_id = widget_id.clone();

            // Adapt size based on breakpoint
            let scale_factor = breakpoint.default_columns() as f32 / Breakpoint::Desktop.default_columns() as f32;
            adapted.width = ((base_layout.width as f32 * scale_factor).ceil() as u32).max(1);
            adapted.clamp_to_constraints();

            layout.add_widget(adapted)
                .map_err(|e| DashboardError::layout(format!("Failed to add widget for {:?}: {}", breakpoint, e)))?;
        }

        Ok(())
    }

    /// Remove widget from all breakpoints
    pub fn remove_widget(&mut self, widget_id: &str) -> DashboardResult<()> {
        for layout in self.breakpoint_layouts.values_mut() {
            layout.remove_widget(widget_id);
        }
        Ok(())
    }

    /// Compact all layouts
    pub fn compact_all(&mut self) {
        for layout in self.breakpoint_layouts.values_mut() {
            layout.compact();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_overlap() {
        let w1 = WidgetLayout::new("w1".to_string(), 0, 0, 2, 2);
        let w2 = WidgetLayout::new("w2".to_string(), 1, 1, 2, 2);
        let w3 = WidgetLayout::new("w3".to_string(), 3, 3, 2, 2);

        assert!(w1.overlaps_with(&w2));
        assert!(!w1.overlaps_with(&w3));
    }

    #[test]
    fn test_add_widget() {
        let mut layout = BreakpointLayout::new(Breakpoint::Desktop, 12);
        let widget = WidgetLayout::new("w1".to_string(), 0, 0, 4, 2);

        assert!(layout.add_widget(widget).is_ok());
        assert_eq!(layout.widgets.len(), 1);
    }

    #[test]
    fn test_widget_overlap_prevention() {
        let mut layout = BreakpointLayout::new(Breakpoint::Desktop, 12);
        let w1 = WidgetLayout::new("w1".to_string(), 0, 0, 4, 2);
        let w2 = WidgetLayout::new("w2".to_string(), 2, 1, 4, 2);

        assert!(layout.add_widget(w1).is_ok());
        assert!(layout.add_widget(w2).is_err());
    }

    #[test]
    fn test_responsive_layout() {
        let mut layout = ResponsiveLayout::new("test-dashboard".to_string());
        let widget = WidgetLayout::new("w1".to_string(), 0, 0, 4, 2);

        assert!(layout.add_widget_responsive("w1".to_string(), widget).is_ok());

        // Check all breakpoints have the widget
        for breakpoint in &[Breakpoint::Mobile, Breakpoint::Desktop] {
            let bp_layout = layout.get_layout(*breakpoint).unwrap();
            assert!(bp_layout.get_widget("w1").is_some());
        }
    }

    #[test]
    fn test_compact() {
        let mut layout = BreakpointLayout::new(Breakpoint::Desktop, 12);
        layout.add_widget(WidgetLayout::new("w1".to_string(), 0, 0, 4, 2)).unwrap();
        layout.add_widget(WidgetLayout::new("w2".to_string(), 4, 5, 4, 2)).unwrap();

        layout.compact();

        // w2 should move up
        let w2 = layout.get_widget("w2").unwrap();
        assert!(w2.y < 5);
    }
}
