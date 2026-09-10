// Sanity module — client, data models, and GROQ queries.
//
// `crate::sanity` is the public path; submodules are accessed as
// `crate::sanity::client`, `crate::sanity::models`, etc.

pub mod client;
pub mod models;
pub mod queries;

pub use client::SanityClient;
