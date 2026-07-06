//! Generic passive components: capacitors and resistors.
//!
//! These are intentionally minimal 2-pin blocks used to model the explicit
//! decoupling capacitors, pull-up resistors and pull-down resistors shown
//! in the HT-HC01 V2 SPI reference design schematic.

use copperleaf::{Block, Farad, Limits, Ohm, Pin, Qty, Role, Second, UnitExt};

// ── Capacitor ─────────────────────────────────────────────────────────

/// A 2-pin capacitor with a fixed capacitance value.
#[derive(Clone, Debug)]
pub struct Capacitor {
    id: String,
    /// Capacitance (stored for BOM / value annotation; not used by ERC).
    #[allow(dead_code)]
    pub value: Qty<Farad>,
    pins: Vec<Pin>,
}

impl Capacitor {
    /// Create a new capacitor with the given capacitance.
    pub fn new(id: &str, value: Qty<Farad>) -> Self {
        let pass_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 50.0.volt(),
            i_max: 1.0.amp(),
        };
        Self {
            id: id.to_owned(),
            value,
            pins: vec![
                Pin::new("1", Role::PowerIn, pass_limits, None),
                Pin::new("2", Role::Gnd, pass_limits, None),
            ],
        }
    }
}

impl Block for Capacitor {
    fn id(&self) -> &str {
        &self.id
    }
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}

// ── Resistor ──────────────────────────────────────────────────────────

/// A 2-pin resistor with a fixed resistance value.
#[derive(Clone, Debug)]
pub struct Resistor {
    id: String,
    /// Resistance (stored for BOM / value annotation; not used by ERC).
    #[allow(dead_code)]
    pub value: Qty<Ohm>,
    pins: Vec<Pin>,
}

impl Resistor {
    /// Create a new resistor with the given resistance.
    pub fn new(id: &str, value: Qty<Ohm>) -> Self {
        let pass_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 50.0.volt(),
            i_max: 0.5.amp(),
        };
        Self {
            id: id.to_owned(),
            value,
            pins: vec![
                Pin::new("1", Role::DigitalIO, pass_limits, None),
                Pin::new("2", Role::DigitalIO, pass_limits, None),
            ],
        }
    }
}

impl Block for Resistor {
    fn id(&self) -> &str {
        &self.id
    }
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}

// ── Crystal ─────────────────────────────────────────────────────────

/// A 2-pin quartz crystal with a specified frequency.
///
/// Both pins are modeled as symmetric analog inputs (passive resonator).
/// The frequency is stored for BOM/value annotation; the actual resonance
/// is established by the driving circuit (e.g. the W5500's internal
/// oscillator amplifier).
#[derive(Clone, Debug)]
pub struct Crystal {
    id: String,
    /// Crystal frequency, stored as a period in seconds.
    /// Use `25.0.mhz()` etc. to construct.
    #[allow(dead_code)]
    pub frequency: Qty<Second>,
    pins: Vec<Pin>,
}

impl Crystal {
    /// Create a new crystal with the given frequency.
    ///
    /// # Example
    /// ```
    /// use copperleaf::UnitExt;
    /// let y = Crystal::new("Y2", 25.0.mhz());
    /// ```
    pub fn new(id: &str, frequency: Qty<Second>) -> Self {
        let xtal_limits = Limits {
            v_min: 0.0.volt(),
            v_max: 3.63.volt(),
            i_max: 0.001.amp(),
        };
        Self {
            id: id.to_owned(),
            frequency,
            pins: vec![
                Pin::new("1", Role::AnalogIn, xtal_limits, None),
                Pin::new("2", Role::AnalogIn, xtal_limits, None),
            ],
        }
    }
}

impl Block for Crystal {
    fn id(&self) -> &str {
        &self.id
    }
    fn pins(&self) -> &[Pin] {
        &self.pins
    }
}