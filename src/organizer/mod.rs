// src/organizer/mod.rs - Organizer plugin module

pub mod folders;
pub mod mame;
pub mod standard;
pub mod plugin;
pub mod rules;

pub use plugin::OrganizerPlugin;
pub use mame::MameOrganizer;
pub use standard::StandardOrganizer;