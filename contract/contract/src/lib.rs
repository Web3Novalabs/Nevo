#![no_std]

pub mod base;
pub mod crowdfunding;
mod interfaces;

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[path = "../test/mod.rs"]
mod test;
