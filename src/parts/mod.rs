//! Custom part definitions for the HT-HC01 V2 SPI reference design.
//!
//! These parts implement the [`copperleaf::Block`] trait, following the
//! conventions established by `copperleaf_parts`.

pub use ht_hc01::HtHc01;
pub use rp2354a::Rp2354a;
pub use w5500::W5500;

pub mod ht_hc01;
pub mod rp2354a;
pub mod w5500;
