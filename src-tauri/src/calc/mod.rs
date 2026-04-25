mod attributes;
mod charges;
mod conversions;
mod defense;
mod offense;
mod perform;
mod setup;

pub use attributes::{calc_attributes, inject_attribute_bonuses};
pub use conversions::{apply_conversion, build_conversion_table, ConversionTable, DamageSet};
pub use defense::calc_defense;
pub use offense::calc_offence;
pub use perform::{calculate, AttributeResult, CalcResult, DefenceResult, OffenceResult};
pub use setup::setup_moddb;
