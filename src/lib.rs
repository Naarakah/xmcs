#![deny(unsafe_code)]
#![deny(unreachable_patterns)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::inline_always)]

pub mod dag;
pub mod set;
pub mod substr;
