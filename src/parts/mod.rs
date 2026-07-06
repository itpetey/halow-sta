//! Custom part definitions for the HT-HC01 V2 SPI reference design.
//!
//! These parts implement the [`copperleaf::Block`] trait, following the
//! conventions established by `copperleaf_parts`.

pub mod ht_hc01;
pub mod passive;
pub mod rp2354a;
pub mod w5500;

pub use ht_hc01::HtHc01;
pub use passive::{Capacitor, Resistor};
pub use rp2354a::Rp2354a;
pub use w5500::W5500;
