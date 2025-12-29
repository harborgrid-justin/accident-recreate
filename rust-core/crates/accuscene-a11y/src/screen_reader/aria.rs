//! ARIA (Accessible Rich Internet Applications) attribute generation
//!
//! Implements WAI-ARIA 1.2 specification for screen reader support

use crate::error::{A11yError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ARIA role definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AriaRole {
    // Document structure roles
    Article,
    Application,
    Banner,
    Complementary,
    ContentInfo,
    Definition,
    Directory,
    Document,
    Feed,
    Figure,
    Group,
    Heading,
    Img,
    List,
    ListItem,
    Main,
    Math,
    Navigation,
    None,
    Note,
    Presentation,
    Region,
    Search,
    Separator,
    Table,
    Term,
    Toolbar,

    // Widget roles
    Alert,
    AlertDialog,
    Button,
    Checkbox,
    ComboBox,
    Dialog,
    GridCell,
    Link,
    Log,
    Marquee,
    Menu,
    MenuBar,
    MenuItem,
    MenuItemCheckbox,
    MenuItemRadio,
    Option,
    ProgressBar,
    Radio,
    RadioGroup,
    ScrollBar,
    SearchBox,
    Slider,
    SpinButton,
    Status,
    Switch,
    Tab,
    TabList,
    TabPanel,
    TextBox,
    Timer,
    ToolTip,
    Tree,
    TreeGrid,
    TreeItem,
}

impl AriaRole {
    /// Get role as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Application => "application",
            Self::Banner => "banner",
            Self::Complementary => "complementary",
            Self::ContentInfo => "contentinfo",
            Self::Definition => "definition",
            Self::Directory => "directory",
            Self::Document => "document",
            Self::Feed => "feed",
            Self::Figure => "figure",
            Self::Group => "group",
            Self::Heading => "heading",
            Self::Img => "img",
            Self::List => "list",
            Self::ListItem => "listitem",
            Self::Main => "main",
            Self::Math => "math",
            Self::Navigation => "navigation",
            Self::None => "none",
            Self::Note => "note",
            Self::Presentation => "presentation",
            Self::Region => "region",
            Self::Search => "search",
            Self::Separator => "separator",
            Self::Table => "table",
            Self::Term => "term",
            Self::Toolbar => "toolbar",
            Self::Alert => "alert",
            Self::AlertDialog => "alertdialog",
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::ComboBox => "combobox",
            Self::Dialog => "dialog",
            Self::GridCell => "gridcell",
            Self::Link => "link",
            Self::Log => "log",
            Self::Marquee => "marquee",
            Self::Menu => "menu",
            Self::MenuBar => "menubar",
            Self::MenuItem => "menuitem",
            Self::MenuItemCheckbox => "menuitemcheckbox",
            Self::MenuItemRadio => "menuitemradio",
            Self::Option => "option",
            Self::ProgressBar => "progressbar",
            Self::Radio => "radio",
            Self::RadioGroup => "radiogroup",
            Self::ScrollBar => "scrollbar",
            Self::SearchBox => "searchbox",
            Self::Slider => "slider",
            Self::SpinButton => "spinbutton",
            Self::Status => "status",
            Self::Switch => "switch",
            Self::Tab => "tab",
            Self::TabList => "tablist",
            Self::TabPanel => "tabpanel",
            Self::TextBox => "textbox",
            Self::Timer => "timer",
            Self::ToolTip => "tooltip",
            Self::Tree => "tree",
            Self::TreeGrid => "treegrid",
            Self::TreeItem => "treeitem",
        }
    }

    /// Check if role requires specific ARIA attributes
    pub fn required_attributes(&self) -> Vec<&'static str> {
        match self {
            Self::Checkbox | Self::Radio | Self::Switch => vec!["aria-checked"],
            Self::ComboBox => vec!["aria-expanded", "aria-controls"],
            Self::Heading => vec!["aria-level"],
            Self::Slider | Self::SpinButton => vec!["aria-valuenow", "aria-valuemin", "aria-valuemax"],
            Self::ProgressBar => vec!["aria-valuenow"],
            _ => vec![],
        }
    }
}

/// ARIA state and property types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AriaState {
    // Widget attributes
    Checked(AriaCheckedState),
    Disabled(bool),
    Expanded(bool),
    Hidden(bool),
    Invalid(AriaInvalidState),
    Pressed(AriaPressedState),
    Selected(bool),

    // Live region attributes
    Atomic(bool),
    Busy(bool),
    Live(AriaLiveState),
    Relevant(Vec<String>),

    // Drag and drop
    Grabbed(bool),
    DropEffect(Vec<String>),

    // Relationship attributes
    ActiveDescendant(String),
    Controls(Vec<String>),
    DescribedBy(Vec<String>),
    Details(String),
    ErrorMessage(String),
    FlowTo(Vec<String>),
    LabelledBy(Vec<String>),
    Owns(Vec<String>),

    // Value attributes
    ValueNow(f64),
    ValueMin(f64),
    ValueMax(f64),
    ValueText(String),

    // Other
    Label(String),
    Level(u32),
    Modal(bool),
    MultiSelectable(bool),
    Orientation(AriaOrientation),
    Placeholder(String),
    ReadOnly(bool),
    Required(bool),
}

/// ARIA checked states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaCheckedState {
    True,
    False,
    Mixed,
}

impl AriaCheckedState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Mixed => "mixed",
        }
    }
}

/// ARIA pressed states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaPressedState {
    True,
    False,
    Mixed,
}

impl AriaPressedState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Mixed => "mixed",
        }
    }
}

/// ARIA invalid states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaInvalidState {
    True,
    False,
    Grammar,
    Spelling,
}

impl AriaInvalidState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Grammar => "grammar",
            Self::Spelling => "spelling",
        }
    }
}

/// ARIA live region politeness levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaLiveState {
    Off,
    Polite,
    Assertive,
}

impl AriaLiveState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Polite => "polite",
            Self::Assertive => "assertive",
        }
    }
}

/// ARIA orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriaOrientation {
    Horizontal,
    Vertical,
}

impl AriaOrientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// ARIA attributes builder
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AriaBuilder {
    role: Option<AriaRole>,
    states: Vec<AriaState>,
    attributes: HashMap<String, String>,
}

impl AriaBuilder {
    /// Create a new ARIA builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ARIA role
    pub fn role(mut self, role: AriaRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Add ARIA state
    pub fn state(mut self, state: AriaState) -> Self {
        self.states.push(state);
        self
    }

    /// Set aria-label
    pub fn label(self, label: impl Into<String>) -> Self {
        self.state(AriaState::Label(label.into()))
    }

    /// Set aria-labelledby
    pub fn labelled_by(self, ids: Vec<String>) -> Self {
        self.state(AriaState::LabelledBy(ids))
    }

    /// Set aria-describedby
    pub fn described_by(self, ids: Vec<String>) -> Self {
        self.state(AriaState::DescribedBy(ids))
    }

    /// Set aria-hidden
    pub fn hidden(self, hidden: bool) -> Self {
        self.state(AriaState::Hidden(hidden))
    }

    /// Set aria-disabled
    pub fn disabled(self, disabled: bool) -> Self {
        self.state(AriaState::Disabled(disabled))
    }

    /// Set aria-expanded
    pub fn expanded(self, expanded: bool) -> Self {
        self.state(AriaState::Expanded(expanded))
    }

    /// Set aria-checked
    pub fn checked(self, checked: AriaCheckedState) -> Self {
        self.state(AriaState::Checked(checked))
    }

    /// Set aria-selected
    pub fn selected(self, selected: bool) -> Self {
        self.state(AriaState::Selected(selected))
    }

    /// Set aria-required
    pub fn required(self, required: bool) -> Self {
        self.state(AriaState::Required(required))
    }

    /// Set aria-invalid
    pub fn invalid(self, invalid: AriaInvalidState) -> Self {
        self.state(AriaState::Invalid(invalid))
    }

    /// Set aria-live
    pub fn live(self, live: AriaLiveState) -> Self {
        self.state(AriaState::Live(live))
    }

    /// Set aria-atomic
    pub fn atomic(self, atomic: bool) -> Self {
        self.state(AriaState::Atomic(atomic))
    }

    /// Build ARIA attributes map
    pub fn build(self) -> Result<HashMap<String, String>> {
        let mut attrs = HashMap::new();

        // Add role
        if let Some(role) = &self.role {
            attrs.insert("role".to_string(), role.as_str().to_string());

            // Check required attributes
            for req_attr in role.required_attributes() {
                let has_attribute = self.states.iter().any(|state| {
                    matches!(
                        (req_attr, state),
                        ("aria-checked", AriaState::Checked(_))
                            | ("aria-expanded", AriaState::Expanded(_))
                            | ("aria-level", AriaState::Level(_))
                            | ("aria-valuenow", AriaState::ValueNow(_))
                            | ("aria-valuemin", AriaState::ValueMin(_))
                            | ("aria-valuemax", AriaState::ValueMax(_))
                    )
                });

                if !has_attribute {
                    return Err(A11yError::MissingAriaAttribute(req_attr.to_string()));
                }
            }
        }

        // Add states
        for state in &self.states {
            match state {
                AriaState::Checked(checked) => {
                    attrs.insert("aria-checked".to_string(), checked.as_str().to_string());
                }
                AriaState::Disabled(disabled) => {
                    attrs.insert("aria-disabled".to_string(), disabled.to_string());
                }
                AriaState::Expanded(expanded) => {
                    attrs.insert("aria-expanded".to_string(), expanded.to_string());
                }
                AriaState::Hidden(hidden) => {
                    attrs.insert("aria-hidden".to_string(), hidden.to_string());
                }
                AriaState::Invalid(invalid) => {
                    attrs.insert("aria-invalid".to_string(), invalid.as_str().to_string());
                }
                AriaState::Pressed(pressed) => {
                    attrs.insert("aria-pressed".to_string(), pressed.as_str().to_string());
                }
                AriaState::Selected(selected) => {
                    attrs.insert("aria-selected".to_string(), selected.to_string());
                }
                AriaState::Label(label) => {
                    attrs.insert("aria-label".to_string(), label.clone());
                }
                AriaState::LabelledBy(ids) => {
                    attrs.insert("aria-labelledby".to_string(), ids.join(" "));
                }
                AriaState::DescribedBy(ids) => {
                    attrs.insert("aria-describedby".to_string(), ids.join(" "));
                }
                AriaState::Live(live) => {
                    attrs.insert("aria-live".to_string(), live.as_str().to_string());
                }
                AriaState::Atomic(atomic) => {
                    attrs.insert("aria-atomic".to_string(), atomic.to_string());
                }
                AriaState::Level(level) => {
                    attrs.insert("aria-level".to_string(), level.to_string());
                }
                AriaState::ValueNow(value) => {
                    attrs.insert("aria-valuenow".to_string(), value.to_string());
                }
                AriaState::ValueMin(value) => {
                    attrs.insert("aria-valuemin".to_string(), value.to_string());
                }
                AriaState::ValueMax(value) => {
                    attrs.insert("aria-valuemax".to_string(), value.to_string());
                }
                AriaState::Required(required) => {
                    attrs.insert("aria-required".to_string(), required.to_string());
                }
                _ => {}
            }
        }

        Ok(attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aria_builder_button() {
        let attrs = AriaBuilder::new()
            .role(AriaRole::Button)
            .label("Click me")
            .disabled(false)
            .build()
            .unwrap();

        assert_eq!(attrs.get("role"), Some(&"button".to_string()));
        assert_eq!(attrs.get("aria-label"), Some(&"Click me".to_string()));
        assert_eq!(attrs.get("aria-disabled"), Some(&"false".to_string()));
    }

    #[test]
    fn test_aria_builder_checkbox() {
        let attrs = AriaBuilder::new()
            .role(AriaRole::Checkbox)
            .checked(AriaCheckedState::True)
            .label("Accept terms")
            .build()
            .unwrap();

        assert_eq!(attrs.get("role"), Some(&"checkbox".to_string()));
        assert_eq!(attrs.get("aria-checked"), Some(&"true".to_string()));
    }

    #[test]
    fn test_missing_required_attribute() {
        let result = AriaBuilder::new()
            .role(AriaRole::Checkbox)
            .label("Incomplete checkbox")
            .build();

        assert!(result.is_err());
    }
}
