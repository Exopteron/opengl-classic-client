/*
This file and other files in the `protocol` module are derivatives of work done by feather-rs. 
A copy of the Apache-2.0 license can be found in FEATHER_LICENSE.md
*/
use anyhow::anyhow;

pub mod codec;
pub mod io;
pub mod packets;
pub mod client;
#[doc(inline)]
pub use codec::Codec;
pub use io::{Readable, Writeable};
#[doc(inline)]
pub use packets::{
    VariantOf,
};