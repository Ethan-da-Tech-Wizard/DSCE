//! A small demonstration knowledge base.
//!
//! Four vials spanning two unrelated domains, chosen so that queries
//! visibly activate only the vials they need and leave the rest dormant:
//!
//! DOMAIN 1 — mortality (a classic syllogism, split across two vials):
//! - `philosophers`  axioms: socrates/plato are human   (facts only)
//! - `biology`       rules: human -> mammal -> mortal   (rules only)
//!
//! Splitting facts from rules forces the flood to CHAIN across vials:
//! neither vial alone can prove "socrates is mortal".
//!
//! DOMAIN 2 — geometry (demonstrates computed conclusions):
//! - `geometry`      rules: square -> rectangle, area = width x height
//! - `measurements`  axioms: the courtyard's and plaza's surveyed sizes
//!
//! The plaza is described only as a square with side 25, so its area proof
//! must first DERIVE that it is a rectangle with width/height 25, then
//! COMPUTE 25 * 25 — a three-rule chain ending in arithmetic.
//!
//! Neighbor links encode "if you're thinking about my content you'll want
//! this too". `philosophers -> biology` and `measurements -> geometry`
//! guarantee the rule vials wake even when no goal constant mentions them.
//! `biology -> philosophers` covers the reverse direction: this engine
//! deliberately does not seed the goal predicate when another constant
//! exists (to prevent predicate flooding), so a category query like
//! `(?who is_a mammal)` arrives carrying only "mammal" — it reaches
//! biology, whose neighbor link pulls in the individuals to reason about.
//!
//! Confidence values are sub-1.0 where the source is fallible (historical
//! texts: 0.99, a physical site survey: 0.97) so confidence propagation is
//! visible in proofs.

use crate::compute::Compute;
use crate::engine::Engine;
use crate::facts::Term;
use crate::vial::{Rule, Vial};

fn t(s: &str) -> Term {
    Term::str(s)
}

/// The four demo vials, ready to add to an engine or save to a store.
pub fn demo_vials() -> Vec<Vial> {
    vec![
        Vial::new("philosophers", "Classical philosophers")
            .with_facts(vec![
                (t("socrates"), t("is_a"), t("human")),
                (t("plato"), t("is_a"), t("human")),
                (t("plato"), t("student_of"), t("socrates")),
            ])
            .with_neighbors(vec!["biology"])
            .with_evidence(vec!["Plato, Apology", "Diogenes Laertius, Lives"])
            .with_confidence(0.99),
        Vial::new("biology", "Basic biology")
            .with_rules(vec![
                Rule::new(
                    "humans-are-mammals",
                    vec![(t("?x"), t("is_a"), t("human"))],
                    (t("?x"), t("is_a"), t("mammal")),
                ),
                Rule::new(
                    "mammals-are-mortal",
                    vec![(t("?x"), t("is_a"), t("mammal"))],
                    (t("?x"), t("is_mortal"), Term::Bool(true)),
                )
                .with_confidence(0.999),
            ])
            .with_neighbors(vec!["philosophers"])
            .with_evidence(vec!["Campbell Biology, ch. 32"]),
        Vial::new("geometry", "Plane geometry")
            .with_rules(vec![
                Rule::new(
                    "squares-are-rectangles",
                    vec![(t("?s"), t("is_a"), t("square")), (t("?s"), t("side"), t("?len"))],
                    (t("?s"), t("is_a"), t("rectangle")),
                ),
                Rule::new(
                    "square-sides-give-width",
                    vec![(t("?s"), t("is_a"), t("square")), (t("?s"), t("side"), t("?len"))],
                    (t("?s"), t("width"), t("?len")),
                ),
                Rule::new(
                    "square-sides-give-height",
                    vec![(t("?s"), t("is_a"), t("square")), (t("?s"), t("side"), t("?len"))],
                    (t("?s"), t("height"), t("?len")),
                ),
                Rule::new(
                    "rectangle-area",
                    vec![
                        (t("?r"), t("is_a"), t("rectangle")),
                        (t("?r"), t("width"), t("?w")),
                        (t("?r"), t("height"), t("?h")),
                    ],
                    (t("?r"), t("area"), t("?a")),
                )
                .with_compute(Compute::expr("?a = ?w * ?h").expect("valid demo expression")),
            ])
            .with_evidence(vec!["Euclid, Elements, Book I"]),
        Vial::new("measurements", "Surveyed measurements")
            .with_facts(vec![
                (t("courtyard"), t("is_a"), t("rectangle")),
                (t("courtyard"), t("width"), Term::Int(12)),
                (t("courtyard"), t("height"), Term::Int(30)),
                (t("plaza"), t("is_a"), t("square")),
                (t("plaza"), t("side"), Term::Int(25)),
            ])
            .with_neighbors(vec!["geometry"])
            .with_evidence(vec!["site survey 2026-03"])
            .with_confidence(0.97),
    ]
}

/// An engine pre-loaded with the demo knowledge base.
pub fn build_engine() -> Engine {
    let mut engine = Engine::new();
    for vial in demo_vials() {
        engine.add_vial(vial).expect("demo vial ids are unique");
    }
    // Schema metadata: these predicates map a subject to at most one value.
    engine.register_predicate("located_in", true);
    engine.register_predicate("height", true);
    engine
}
