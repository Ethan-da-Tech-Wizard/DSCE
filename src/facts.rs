// ==============================================================================
// Program: facts.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: Core semantic vocabulary, triple terms, and unification algorithms in Rust.
//          Defines Term, Fact, Pattern, and Bindings data structures. Implements
//          pattern unification, variable substitutions, constant term extraction,
//          and lexical specificity comparisons.
// ==============================================================================

//! Facts, patterns, and unification — the vocabulary of the whole engine.
//!
//! EVERYTHING the DSCE knows or asks is expressed as a triple:
//!
//! ```text
//!     (subject, predicate, object)
//! ```
//!
//! - A FACT is a "ground" triple: all three positions are concrete values.
//! - A PATTERN is a triple that may contain VARIABLES — strings starting
//!   with `?` — standing for "any value, to be determined".
//!
//! Queries are patterns. Rule premises and conclusions are patterns.
//! Matching a pattern against a fact is called UNIFICATION, and the record
//! of what each variable turned out to equal is a [`Bindings`] map.
//!
//! Unification is deliberately FLAT — a term is never itself a triple —
//! matching standard RDF triple-store designs.

// File role: the shared data model. Every other module (engine, vial, compute, harvester, db_store) builds on
// `Term`, `Fact`, `Pattern` and `Bindings` from here, and calls `unify`/`substitute` during rule matching.
//
// Design decisions: terms are a closed enum of four types (no nested terms), which keeps unification a simple
// position-by-position comparison; ordered collections (BTreeMap) and `OrderedFloat` are used so that iteration
// order, and therefore every result the engine produces, is identical from run to run.
//
// Tested at the bottom of this file (unification, substitution, lexical and taxonomic specificity).
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use ordered_float::OrderedFloat;

/// One position of a triple.
///
/// `OrderedFloat` gives floats a total order and a hash, so `Term` can
/// derive full `Eq + Ord + Hash` — which is what lets facts live in sorted
/// maps and guarantees deterministic iteration everywhere.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// Variant meanings: Str is text or a `?variable`; Int/Float are numbers; Bool is true/false. Derived Ord sorts
// variants in declaration order (Str < Int < Float < Bool), which is arbitrary but fixed.
pub enum Term {
    Str(String),
    Int(i64),
    Float(OrderedFloat<f64>),
    Bool(bool),
}

impl Term {
    /// Convenience constructor for string terms.
    // Takes anything convertible to String so call sites can pass &str or String without clones.
    pub fn str(s: impl Into<String>) -> Term {
        Term::Str(s.into())
    }

    /// Convenience constructor for float terms.
    // The only way to build a Float term; it wraps the value so equality and hashing treat NaN consistently.
    pub fn float(f: f64) -> Term {
        Term::Float(OrderedFloat(f))
    }

    /// A variable is any string term starting with `?` (e.g. `?x`).
    /// Everything else — other strings, ints, floats, bools — is a constant.
    // Only strings can be variables; the number 3 or the string "x" are constants.
    pub fn is_variable(&self) -> bool {
        matches!(self, Term::Str(s) if s.starts_with('?'))
    }

    /// The variable name if this term is a variable, else `None`.
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Term::Str(s) if s.starts_with('?') => Some(s.as_str()),
            _ => None,
        }
    }
}

// Display is also the storage form (db_store::term_to_db), so changing it would change what is written to databases.
impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Str(s) => write!(f, "{s}"),
            Term::Int(i) => write!(f, "{i}"),
            // {:?} keeps "625.0" distinguishable from the integer "625".
            Term::Float(x) => write!(f, "{:?}", x.0),
            Term::Bool(b) => write!(f, "{b}"),
        }
    }
}

// From impls let tests and callers write `"x".into()` for terms instead of calling the constructors explicitly.
impl From<&str> for Term {
    fn from(s: &str) -> Term {
        Term::Str(s.to_string())
    }
}

impl From<String> for Term {
    fn from(s: String) -> Term {
        Term::Str(s)
    }
}

impl From<i64> for Term {
    fn from(i: i64) -> Term {
        Term::Int(i)
    }
}

impl From<f64> for Term {
    fn from(x: f64) -> Term {
        Term::Float(OrderedFloat(x))
    }
}

impl From<bool> for Term {
    fn from(b: bool) -> Term {
        Term::Bool(b)
    }
}

/// A ground triple: no variables in any position.
// Same shape as Pattern; the two names document intent (ground vs. may-contain-variables), not a different type.
pub type Fact = (Term, Term, Term);

/// A triple that may contain variables.
pub type Pattern = (Term, Term, Term);

/// Variable name -> the value it is bound to.
///
/// A `BTreeMap` (not `HashMap`) so iteration order is always sorted —
/// one of the several places determinism is enforced.
pub type Bindings = BTreeMap<String, Term>;

/// Iterate the three positions of a pattern/fact pair in parallel.
// Pairs up subject/predicate/object so `unify` can loop once instead of repeating the same logic three times.
fn positions<'a>(a: &'a (Term, Term, Term), b: &'a (Term, Term, Term)) -> [(&'a Term, &'a Term); 3] {
    [(&a.0, &b.0), (&a.1, &b.1), (&a.2, &b.2)]
}

/// Try to match `pattern` against ground `fact`, extending `bindings`.
///
/// Walks the three positions in parallel. At each position:
/// - pattern has a CONSTANT -> it must equal the fact's value exactly,
///   otherwise the match fails (return `None`).
/// - pattern has a VARIABLE -> if already bound, its bound value must equal
///   the fact's value; if unbound, it becomes bound to the fact's value.
///
/// Returns the EXTENDED bindings on success, `None` on failure. The input
/// `bindings` is never mutated — callers rely on being able to try many
/// facts against the same starting bindings (the premise join does exactly
/// that), so the extension is built on a clone.
// Called by: the engine's premise join (once per candidate fact per premise). Complexity O(1) per call plus the bindings clone.
pub fn unify(pattern: &Pattern, fact: &Fact, bindings: &Bindings) -> Option<Bindings> {
    // Clone first, extend later: a failed partial match must leave the caller's bindings untouched.
    let mut result = bindings.clone();
    for (p, f) in positions(pattern, fact) {
        if let Some(var) = p.as_variable() {
            match result.get(var) {
                // A variable that appears twice in a pattern (e.g. `(?x, knows, ?x)`) must match the same value both times.
                Some(bound) if bound != f => return None, // already bound to something else
                Some(_) => {}
                None => {
                    result.insert(var.to_string(), f.clone());
                }
            }
        } else if p != f {
            return None; // constant mismatch
        }
    }
    Some(result)
}

/// Turn a pattern into a ground fact by filling in every variable.
///
/// Errors if any variable has no binding — the engine treats that as
/// "this rule cannot conclude anything for this match" and skips it rather
/// than producing a half-ground fact.
// Called by: the engine when a rule concludes a new fact. Fails (instead of leaving a `?var`) so non-ground facts never enter working memory.
pub fn substitute(pattern: &Pattern, bindings: &Bindings) -> Result<Fact, String> {
    // Constants pass through unchanged; variables are replaced by their binding.
    let fill = |term: &Term| -> Result<Term, String> {
        if let Some(var) = term.as_variable() {
            bindings
                .get(var)
                .cloned()
                .ok_or_else(|| format!("unbound variable {var} in pattern"))
        } else {
            Ok(term.clone())
        }
    };
    Ok((fill(&pattern.0)?, fill(&pattern.1)?, fill(&pattern.2)?))
}

/// The non-variable terms of a pattern, in position order.
// Used to decide which terms of a pattern can wake vials: variables carry no information, constants do.
pub fn constants(pattern: &Pattern) -> Vec<Term> {
    [&pattern.0, &pattern.1, &pattern.2]
        .into_iter()
        .filter(|t| !t.is_variable())
        .cloned()
        .collect()
}

/// Returns true if `term1` carries more specific context or detail than `term2`.
///
/// Two checks, mirroring the Python prototype:
/// 1. LEXICAL: `term1` contains `term2` as a (case-insensitive) substring,
///    e.g. `"tower of pie in ajo arizona"` contains `"tower of pie"`.
/// 2. TAXONOMIC: a BFS over `is_a` edges in the supplied facts reaches
///    `term2` starting from `term1` (socrates is_a human is_a mammal =>
///    "socrates" is more specific than "mammal").
// Used by conflict resolution when two facts disagree: the more specific statement wins. Cost is a BFS over the given facts.
pub fn is_more_specific(term1: &Term, term2: &Term, facts: &[&Fact]) -> bool {
    // Specificity is defined only for text; numbers and booleans are never "more specific" than anything.
    let (Term::Str(s1), Term::Str(s2)) = (term1, term2) else {
        return false;
    };

    // A term is not more specific than itself, so equal strings are false (this also stops trivial cycles).
    if s1 == s2 {
        return false;
    }

    // 1. Lexical sub-string containment.
    if s1.to_lowercase().contains(&s2.to_lowercase()) {
        return true;
    }

    // 2. Taxonomic specialization along is_a paths.
    let is_a = Term::str("is_a");

    // `visited` is a Vec (linear contains) rather than a set because the `is_a` chains here are a handful of terms long.
    let mut visited: Vec<&Term> = Vec::new();
    let mut queue: VecDeque<&Term> = VecDeque::new();
    queue.push_back(term1);
    while let Some(curr) = queue.pop_front() {
        if visited.contains(&curr) {
            continue;
        }
        visited.push(curr);
        if curr == term2 {
            return true;
        }
        for (subj, pred, obj) in facts.iter() {
            if subj == curr && *pred == is_a {
                queue.push_back(obj);
            }
        }
    }
    false
}

#[cfg(test)]
// The tests use the classic examples from the docs (Socrates is a human, a mammal) so they double as usage examples.
mod tests {
    use super::*;

    // Shorthand for string terms to keep the assertions readable.
    fn t(s: &str) -> Term {
        Term::str(s)
    }

    #[test]
    // A fresh variable must be bound to the fact's value.
    fn unify_binds_variables() {
        let b = unify(
            &(t("?x"), t("is_a"), t("human")),
            &(t("socrates"), t("is_a"), t("human")),
            &Bindings::new(),
        )
        .unwrap();
        assert_eq!(b.get("?x"), Some(&t("socrates")));
    }

    #[test]
    // If `?x` is already Socrates, a fact about Plato must not unify: bindings constrain later premises.
    fn unify_respects_existing_bindings() {
        let mut existing = Bindings::new();
        existing.insert("?x".into(), t("socrates"));
        assert!(unify(
            &(t("?x"), t("is_a"), t("human")),
            &(t("plato"), t("is_a"), t("human")),
            &existing,
        )
        .is_none());
    }

    #[test]
    // Constants must match exactly, so a different object ("god" vs "human") fails.
    fn unify_rejects_constant_mismatch() {
        assert!(unify(
            &(t("socrates"), t("is_a"), t("god")),
            &(t("socrates"), t("is_a"), t("human")),
            &Bindings::new(),
        )
        .is_none());
    }

    #[test]
    // `?a` has no binding, so the result must be an error (the rule is skipped) rather than a fact with a leftover variable.
    fn substitute_errors_on_unbound() {
        let mut b = Bindings::new();
        b.insert("?x".into(), t("courtyard"));
        assert!(substitute(&(t("?x"), t("area"), t("?a")), &b).is_err());
    }

    #[test]
    // Longer string containing the shorter one wins, not the reverse, not for equal strings, and not for non-strings.
    fn lexical_specificity() {
        assert!(is_more_specific(&t("tower of pie in ajo arizona"), &t("tower of pie"), &[]));
        assert!(!is_more_specific(&t("tower of pie"), &t("tower of pie in ajo arizona"), &[]));
        assert!(!is_more_specific(&t("tower of pie"), &t("tower of pie"), &[]));
        assert!(!is_more_specific(&t("tower of pie"), &Term::Int(123), &[]));
    }

    #[test]
    // Follows is_a chains transitively (socrates -> human -> mammal) but never in the reverse direction or to unrelated terms.
    fn taxonomic_specificity() {
        let f1 = (t("socrates"), t("is_a"), t("human"));
        let f2 = (t("human"), t("is_a"), t("mammal"));
        let wm: Vec<&Fact> = vec![&f1, &f2];
        assert!(is_more_specific(&t("socrates"), &t("human"), &wm));
        assert!(is_more_specific(&t("socrates"), &t("mammal"), &wm));
        assert!(is_more_specific(&t("human"), &t("mammal"), &wm));
        assert!(!is_more_specific(&t("mammal"), &t("socrates"), &wm));
        assert!(!is_more_specific(&t("socrates"), &t("geometry"), &wm));
    }
}
