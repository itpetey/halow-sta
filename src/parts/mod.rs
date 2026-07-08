//! Custom part definitions for the MM8108-MF15457 SPI reference design.
//!
//! Parts use `#[derive(Component)]` to implement the [`copperleaf::Block`]
//! trait, following the conventions established by `copperleaf_parts`.

pub use mm8108_mf15457::Mm8108Mf15457;
pub use rp2354a::Rp2354a;
pub use w5500::W5500;

pub mod mm8108_mf15457;
pub mod rp2354a;
pub mod w5500;
