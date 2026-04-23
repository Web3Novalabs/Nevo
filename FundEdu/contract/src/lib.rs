#![no_std]

mod contract;
mod storage;
mod types;

pub use contract::FundEduContract;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
