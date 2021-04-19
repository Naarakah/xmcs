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
#![allow(clippy::inline_always)]


pub mod substr;
pub mod set;
pub mod dag;
