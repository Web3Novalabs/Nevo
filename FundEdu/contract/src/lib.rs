#![no_std]

mod contract;
mod errors;
mod storage;
mod types;

pub use contract::{FundEduContract, FundEduError};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
