// Dummy library for rustdoc JSON generation
// This library provides the necessary structure for rustdoc-json to work

pub mod cache;
pub mod error;
pub mod format;
pub mod parser;
pub mod query;
pub mod types;

#[cfg(test)]
mod proptest;
