// ==============================================================================
// Program: json_vials.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: JSON vial serialization and loader subsystem for DSCE.
//          Parses declarative JSON knowledge definitions (patterns, libraries),
//          deserializes typed terms, validates DSL compute expressions, and
//          constructs in-memory Vial and Rule models from directory structures.
// ==============================================================================

//! Normalized JSON vials — knowledge as data files.
//!
//! The synthesis knowledge base (`vials_synthesis/`) stores vials as plain
//! JSON documents so that design patterns and API libraries can be written,
//! reviewed, and diffed without touching Rust code:
//!
//! ```json
//! {
//!   "id": "grid_layout",
//!   "concept": "Generic grid layout pattern",
//!   "facts": [["grid_layout", "flow", "row_major"]],
//!   "rules": [{
//!     "name": "grid-flow-direction",
//!     "premises": [["?app", "needs", "grid_layout"], ["grid_layout", "flow", "?flow"]],
//!     "conclusion": ["?app", "layout_flow", "?flow"],
//!     "compute": "?code = ?a + ?b"        // optional expression-DSL hook
//!   }],
//!   "neighbors": [], "evidence": ["..."], "confidence": 1.0
//! }
//! ```
//!
//! Terms keep their JSON types (strings, integers, floats, booleans), and
//! `compute` strings are parsed by [`crate::compute`] at load time — a bad
//! expression fails the load loudly instead of silently dropping logic.

use std::fs;
use std::path::{Path, PathBuf};

// serde derives the JSON-to-struct mapping for the two *Doc types below.
use serde::Deserialize;
use serde_json::Value;

use crate::compute::Compute;
use crate::db_store::triple_from_json;
use crate::engine::Engine;
use crate::facts::{Fact, Pattern};
use crate::vial::{Rule, Vial};

/// Serde default for a missing "confidence": fully trusted (1.0).
fn default_confidence() -> f64 {
    1.0
}

#[derive(Deserialize)]
/// On-disk shape of a rule. Premises/conclusion stay as raw JSON `Value`s here and are converted into typed terms afterwards, so errors can name the rule.
struct RuleDoc {
    name: String,
    #[serde(default)]
    premises: Vec<Value>,
    conclusion: Value,
    #[serde(default)]
    compute: Option<String>,
    #[serde(default = "default_confidence")]
    confidence: f64,
}

#[derive(Deserialize)]
/// On-disk shape of a vial. Only `id` is required; every other field defaults to empty/1.0, which keeps hand-written vial files short.
struct VialDoc {
    id: String,
    #[serde(default)]
    concept: String,
    #[serde(default)]
    facts: Vec<Value>,
    #[serde(default)]
    rules: Vec<RuleDoc>,
    #[serde(default)]
    neighbors: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default = "default_confidence")]
    confidence: f64,
}

// Decode one JSON triple, prefixing any error with `context` (e.g. which rule of which vial).
fn triple(value: &Value, context: &str) -> Result<Pattern, String> {
    triple_from_json(value).map_err(|e| format!("{context}: {e}"))
}

/// Parse one vial from JSON text.
pub fn vial_from_json(text: &str) -> Result<Vial, String> {
    let doc: VialDoc = serde_json::from_str(text).map_err(|e| format!("bad vial JSON: {e}"))?;
    let facts: Vec<Fact> = doc
        .facts
        .iter()
        .map(|f| triple(f, &format!("fact in vial {:?}", doc.id)))
        .collect::<Result<_, _>>()?;
    let mut rules: Vec<Rule> = Vec::new();
    for rule_doc in doc.rules {
        let context = format!("rule {:?} in vial {:?}", rule_doc.name, doc.id);
        let premises = rule_doc
            .premises
            .iter()
            .map(|p| triple(p, &context))
            .collect::<Result<Vec<_>, _>>()?;
        let conclusion = triple(&rule_doc.conclusion, &context)?;
        // Parse the compute DSL now so a malformed expression fails the load instead of being silently ignored later.
        let compute = match rule_doc.compute {
            None => None,
            Some(src) => Some(Compute::expr(&src).map_err(|e| format!("{context}: bad compute: {e}"))?),
        };
        rules.push(Rule {
            name: rule_doc.name,
            premises,
            conclusion,
            compute,
            confidence: rule_doc.confidence,
        });
    }
    Ok(Vial {
        id: doc.id,
        concept: doc.concept,
        facts,
        rules,
        neighbors: doc.neighbors,
        evidence: doc.evidence,
        confidence: doc.confidence,
    })
}

/// Load one `.json` vial file.
pub fn load_vial_file(path: impl AsRef<Path>) -> Result<Vial, String> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    vial_from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Load every `.json` vial under `dir`, recursively.
///
/// Paths are visited in sorted order so the returned vial list — and
/// therefore everything the engine derives from it — is deterministic.
pub fn load_vials_dir(dir: impl AsRef<Path>) -> Result<Vec<Vial>, String> {
    let dir = dir.as_ref();
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_json_files(dir, &mut paths)?;
    // Sorting makes load order independent of filesystem iteration order, which varies between systems.
    paths.sort();
    paths.into_iter().map(load_vial_file).collect()
}

/// An engine loaded from a JSON vial directory, with the synthesis
/// knowledge base's schema metadata registered:
///
/// - FUNCTIONAL predicates (`located_in`, `height`): one value per subject;
///   disagreements raise conflict warnings.
/// - ANNOTATION predicates (`function`, `param`, `returns`, `color`, `rgb`,
///   toolchain metadata like `install_command`/`build_command`):
///   API documentation that rules can match but that never emits sand —
///   shared doc vocabulary ("None", "size", shell command strings) must not
///   build activation bridges between unrelated libraries.
pub fn engine_from_dir(dir: impl AsRef<Path>) -> Result<Engine, String> {
    let mut engine = Engine::new();
    for vial in load_vials_dir(dir)? {
        engine.add_vial(vial)?;
    }
    engine.register_predicate("located_in", true);
    engine.register_predicate("height", true);
    // Annotation predicates are documentation-like facts that rules may match but that do not carry activation (see the doc comment above).
    for annotation in [
        "function",
        "param",
        "returns",
        "color",
        "rgb",
        "definition",
        "install_command",
        "install_step",
        "file_extension",
        "build_command",
        "run_command",
        "docs_url",
        "maps_to_need",
    ] {
        engine.register_annotation(annotation);
    }
    Ok(engine)
}

// Recursively gather every `*.json` file under `dir` into `out` (unordered; the caller sorts).
fn collect_json_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read directory {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
// Tests cover successful parsing and the "bad compute fails loudly" guarantee.
mod tests {
    use super::*;
    use crate::facts::Term;

    #[test]
    fn parses_facts_rules_and_compute() {
        let vial = vial_from_json(
            r#"{
                "id": "t",
                "concept": "test",
                "facts": [["a", "width", 3], ["a", "height", 4]],
                "rules": [{
                    "name": "area",
                    "premises": [["?r", "width", "?w"], ["?r", "height", "?h"]],
                    "conclusion": ["?r", "area", "?a"],
                    "compute": "?a = ?w * ?h"
                }]
            }"#,
        )
        .unwrap();
        assert_eq!(vial.facts[0].2, Term::Int(3));
        assert_eq!(vial.rules.len(), 1);
        assert!(vial.rules[0].compute.is_some());
        assert_eq!(vial.confidence, 1.0);
    }

    #[test]
    fn bad_compute_fails_loudly() {
        let err = vial_from_json(
            r#"{"id": "t", "rules": [{"name": "r", "premises": [],
                "conclusion": ["a", "b", "c"], "compute": "?x = )("}]}"#,
        )
        .unwrap_err();
        assert!(err.contains("bad compute"), "got: {err}");
    }
}
