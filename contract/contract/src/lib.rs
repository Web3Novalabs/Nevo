#![no_std]

/// The mod base;.
pub mod base;
/// The mod crowdfunding;.
pub mod contract;
pub mod crowdfunding;
mod interfaces;

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[path = "../test/mod.rs"]
mod test;
