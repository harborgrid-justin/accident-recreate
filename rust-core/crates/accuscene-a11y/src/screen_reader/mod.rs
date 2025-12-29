//! Screen reader support modules

pub mod announcements;
pub mod aria;

pub use announcements::{AnnouncementPriority, LiveRegion, ScreenReaderAnnouncer};
pub use aria::{AriaBuilder, AriaRole, AriaState};
