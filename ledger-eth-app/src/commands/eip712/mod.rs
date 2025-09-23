// SPDX-License-Identifier: Apache-2.0

//! EIP-712 command implementations
//!
//! This module contains all EIP-712 related functionality organized by APDU command type.

pub mod encoding;
pub mod filtering;
pub mod high_level;
pub mod signing;
pub mod structs;

// Re-export all public traits and types
pub use encoding::*;
pub use filtering::*;
pub use high_level::*;
pub use signing::*;
pub use structs::*;
