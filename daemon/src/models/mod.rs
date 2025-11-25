//! Data models for Fall Guys log parsing.
//!
//! This module contains all the data structures used to represent
//! parsed game data. The models are organized into submodules:
//!
//! - [`common`]: Common types like platforms and round info
//! - [`dto`]: Data transfer objects for episode completion
//! - [`exports`]: Export types for external integrations
//! - [`messages`]: Game event messages
//! - [`state`]: Game state enumerations

pub mod common;
pub mod dto;
pub mod exports;
pub mod messages;
pub mod state;
