//! Deterministic Semantic Computation Engine (DSCE) — Rust implementation.
//!
//! A sand-and-vials spreading-activation reasoner:
//!
//! - [`facts`]    triples, patterns, unification, specificity
//! - [`vial`]     knowledge containers (facts + rules + provenance)
//! - [`sand`]     activation grains
//! - [`compute`]  deterministic, serializable rule computations
//! - [`engine`]   the flood loop (rayon-parallel rule firing)
//! - [`db_store`] SQLite persistence and dynamic vial loading
//! - [`proof`]    derivation records and proof-tree rendering
//! - [`demo_kb`]  the showcase knowledge base
//! - [`json_vials`] normalized JSON vial files (the synthesis KB format)
//! - [`harvester`] the Semantic Harvester: natural language -> goal + triples

pub mod compute;
pub mod db_store;
pub mod demo_kb;
pub mod engine;
pub mod facts;
pub mod harvester;
pub mod json_vials;
pub mod proof;
pub mod sand;
pub mod vial;
