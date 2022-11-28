#[macro_use]
extern crate protocol_derive;

pub use protocol::Settings;
pub use protocol::Parcel;

pub mod rabbit;
pub mod validation;