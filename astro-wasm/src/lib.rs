// Module declarations — modular layout (003-arch-layout)
pub mod swe_wrappers;
pub mod utils;
pub mod localization;
pub mod locales;
pub mod engines;
pub mod bridge;

// Re-export the single WASM entry point.
pub use bridge::bridge;

