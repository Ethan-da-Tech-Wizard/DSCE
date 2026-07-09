//! Vials: the knowledge containers of the DSCE.
//!
//! A vial is the unit of knowledge STORAGE, the way a parameter matrix is
//! the unit of storage in a neural network — except a vial is explicit,
//! readable, and citable. Each vial holds a small, coherent body of
//! knowledge about one concept:
//!
//! ```text
//!     facts       ground axioms this vial asserts outright
//!     rules       "if premises then conclusion" inference steps
//!     neighbors   ids of related vials, woken alongside this one
//!     evidence    human-readable sources backing this vial's content
//!     confidence  how much the vial as a whole is trusted (0.0 - 1.0)
//! ```
//!
//! Vials stay DORMANT until sand reaches them; only activated vials
//! contribute facts and fire rules. This is the sparsity claim of the
//! architecture: a geometry question never pays for the philosophy vial.

use std::collections::BTreeSet;

use crate::compute::Compute;
use crate::facts::{constants, Fact, Pattern, Term};

/// An inference rule: if ALL premises hold, the conclusion holds.
///
/// `compute` is the optional deterministic function from bindings to EXTRA
/// bindings — the "computation" in DSCE. It lets a conclusion contain a
/// value no premise supplied (e.g. the rectangle-area rule matches `?w`
/// and `?h` from premises and computes `?a = ?w * ?h`). It MUST be pure
/// and deterministic or the engine's determinism guarantee is broken.
///
/// `confidence` is how much this rule itself is trusted; multiplied into
/// the confidence of every fact it derives.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub premises: Vec<Pattern>,
    pub conclusion: Pattern,
    pub compute: Option<Compute>,
    pub confidence: f64,
}

impl Rule {
    /// A rule with no compute hook and full confidence.
    pub fn new(name: impl Into<String>, premises: Vec<Pattern>, conclusion: Pattern) -> Rule {
        Rule {
            name: name.into(),
            premises,
            conclusion,
            compute: None,
            confidence: 1.0,
        }
    }

    pub fn with_compute(mut self, compute: Compute) -> Rule {
        self.compute = Some(compute);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Rule {
        self.confidence = confidence;
        self
    }
}

/// One knowledge container. See the module docs for the concept.
#[derive(Debug, Clone)]
pub struct Vial {
    /// Unique key, used in indexes, proofs, and neighbor links.
    pub id: String,
    /// Human-readable one-liner of what this vial is about.
    pub concept: String,
    /// Ground axioms poured into working memory when the vial activates.
    pub facts: Vec<Fact>,
    /// Rules fired every tick while the vial is active.
    pub rules: Vec<Rule>,
    /// Ids of related vials, woken (transitively, within the same tick)
    /// whenever this vial wakes. Encodes "if you're thinking about X you
    /// will probably need Y" — e.g. measurements -> geometry.
    pub neighbors: Vec<String>,
    /// Human-readable sources; attached to every axiom this vial
    /// contributes, so they appear in proof traces.
    pub evidence: Vec<String>,
    /// Trust in this vial overall; axioms inherit it, and it discounts
    /// every rule firing from this vial.
    pub confidence: f64,
}

impl Vial {
    pub fn new(id: impl Into<String>, concept: impl Into<String>) -> Vial {
        Vial {
            id: id.into(),
            concept: concept.into(),
            facts: Vec::new(),
            rules: Vec::new(),
            neighbors: Vec::new(),
            evidence: Vec::new(),
            confidence: 1.0,
        }
    }

    pub fn with_facts(mut self, facts: Vec<Fact>) -> Vial {
        self.facts = facts;
        self
    }

    pub fn with_rules(mut self, rules: Vec<Rule>) -> Vial {
        self.rules = rules;
        self
    }

    pub fn with_neighbors(mut self, neighbors: Vec<&str>) -> Vial {
        self.neighbors = neighbors.into_iter().map(String::from).collect();
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<&str>) -> Vial {
        self.evidence = evidence.into_iter().map(String::from).collect();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Vial {
        self.confidence = confidence;
        self
    }

    /// Every constant this vial mentions anywhere — its "address".
    ///
    /// The engine indexes vials under these terms; a sand grain carrying
    /// one of them will find and wake this vial. Collected from every
    /// position of every axiom fact and every CONSTANT position of every
    /// rule premise and conclusion (variables are placeholders, not
    /// addresses). Returned as a `BTreeSet` so iteration is sorted —
    /// determinism again.
    pub fn terms(&self) -> BTreeSet<Term> {
        let mut found = BTreeSet::new();
        for (s, p, o) in &self.facts {
            found.insert(s.clone());
            found.insert(p.clone());
            found.insert(o.clone());
        }
        for rule in &self.rules {
            for premise in &rule.premises {
                found.extend(constants(premise));
            }
            found.extend(constants(&rule.conclusion));
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::Term;

    #[test]
    fn terms_collects_facts_and_rule_constants() {
        let vial = Vial::new("v", "test")
            .with_facts(vec![(Term::str("a"), Term::str("is_a"), Term::str("b"))])
            .with_rules(vec![Rule::new(
                "r",
                vec![(Term::str("?x"), Term::str("is_a"), Term::str("b"))],
                (Term::str("?x"), Term::str("is_a"), Term::str("c")),
            )]);
        let terms = vial.terms();
        for expected in ["a", "is_a", "b", "c"] {
            assert!(terms.contains(&Term::str(expected)), "missing {expected}");
        }
        assert!(!terms.contains(&Term::str("?x")), "variables are not addresses");
    }
}
