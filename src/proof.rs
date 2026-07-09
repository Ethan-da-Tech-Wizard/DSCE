//! Proof construction and rendering — how the DSCE shows its work.
//!
//! THE KEY DESIGN DECISION in this file: proofs are not built as a separate
//! step after reasoning. They fall out of bookkeeping the engine does
//! anyway. Every fact that enters working memory is stored alongside a
//! [`Derivation`] record answering "how do I know this?":
//!
//! - AXIOM:   "vial V asserted it directly" (+ V's evidence sources), or
//! - DERIVED: "rule R in vial V concluded it from these premise facts".
//!
//! Because each derived fact's premises are themselves facts in working
//! memory (with their own Derivation records), walking the records
//! backwards from any answer reconstructs the complete reasoning chain,
//! ending at cited axioms. That walk IS the proof tree — the DSCE's
//! replacement for a neural network's opaque hidden states.
//!
//! Rendering example (from `cargo run -- socrates is_mortal ?x`):
//!
//! ```text
//! (socrates is_mortal true)  [by rule 'mammals-are-mortal' in vial 'biology', confidence 0.989]
//! └─ (socrates is_a mammal)  [by rule 'humans-are-mammals' in vial 'biology', confidence 0.990]
//!    └─ (socrates is_a human)  [axiom in vial 'philosophers' (evidence: Plato, Apology, ...), confidence 0.990]
//! ```
//!
//! Read bottom-up: an axiom with its source, each rule that consumed it,
//! and the final answer — every step attributable, every step checkable.

use std::collections::BTreeMap;

use crate::facts::Fact;

/// The "how do I know this?" record attached to every working-memory fact.
#[derive(Debug, Clone)]
pub struct Derivation {
    /// The ground triple this record explains.
    pub fact: Fact,
    /// The vial responsible: home of the axiom, or home of the rule that fired.
    pub vial_id: String,
    /// ALREADY-COMBINED confidence of the whole chain below this fact
    /// (rule x vial x weakest premise), computed at derivation time.
    /// Nothing needs to re-walk the tree to know how trustworthy a fact is.
    pub confidence: f64,
    /// Name of the rule that derived it, or `None` -> axiom.
    pub rule_name: Option<String>,
    /// The ground premise facts the rule consumed. These are the proof
    /// tree's child edges. Empty for axioms.
    pub premises: Vec<Fact>,
    /// Human-readable sources, carried only by axioms (derived facts point
    /// at their premises instead).
    pub evidence: Vec<String>,
}

/// The engine's working memory once a flood has settled: every known fact
/// mapped to its derivation. A `BTreeMap` so iteration is always sorted —
/// deterministic answers, deterministic proofs.
pub type Derivations = BTreeMap<Fact, Derivation>;

/// Render a triple as `(subject predicate object)`.
pub fn fact_str(fact: &Fact) -> String {
    format!("({} {} {})", fact.0, fact.1, fact.2)
}

/// Render the proof tree rooted at `root` as indented text with
/// box-drawing connectors, by recursively walking the derivation records.
pub fn render_proof(root: &Fact, derivations: &Derivations) -> String {
    let mut lines = Vec::new();
    render_node(root, derivations, "", true, true, &mut lines);
    lines.join("\n")
}

/// Recursive worker; one call renders one node.
///
/// Layout bookkeeping (standard tree-drawing technique):
/// - `prefix`  the accumulated indentation inherited from ancestors —
///   either spaces (under a last child) or `│  ` (under a non-last child,
///   so the vertical rail continues down to its remaining siblings).
/// - `is_last` whether this node is its parent's final premise; picks
///   `└─ ` vs `├─ ` as this node's connector.
/// - `is_root` the root gets no connector and adds no indentation.
fn render_node(
    fact: &Fact,
    derivations: &Derivations,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    lines: &mut Vec<String>,
) {
    let Some(d) = derivations.get(fact) else {
        lines.push(format!("{prefix}?? missing derivation for {}", fact_str(fact)));
        return;
    };
    let because = match &d.rule_name {
        None => {
            let mut s = format!("axiom in vial '{}'", d.vial_id);
            if !d.evidence.is_empty() {
                s.push_str(&format!(" (evidence: {})", d.evidence.join(", ")));
            }
            s
        }
        Some(rule) => format!("by rule '{}' in vial '{}'", rule, d.vial_id),
    };
    let connector = if is_root {
        ""
    } else if is_last {
        "└─ "
    } else {
        "├─ "
    };
    lines.push(format!(
        "{prefix}{connector}{}  [{because}, confidence {:.3}]",
        fact_str(fact),
        d.confidence
    ));
    let child_prefix = if is_root {
        prefix.to_string()
    } else if is_last {
        format!("{prefix}   ")
    } else {
        format!("{prefix}│  ")
    };
    for (i, premise) in d.premises.iter().enumerate() {
        render_node(
            premise,
            derivations,
            &child_prefix,
            i == d.premises.len() - 1,
            false,
            lines,
        );
    }
}
