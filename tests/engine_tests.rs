//! Integration tests mirroring the Python prototype's test suite
//! (tests/test_engine.py and tests/test_new_features.py).

use dsce::compute::Compute;
use dsce::db_store::SqliteVialStore;
use dsce::demo_kb::{build_engine, demo_vials};
use dsce::engine::Engine;
use dsce::facts::Term;
use dsce::vial::{Rule, Vial};

fn t(s: &str) -> Term {
    Term::str(s)
}

// --- flood behavior over the demo KB -----------------------------------

#[test]
fn multi_vial_inference_chain() {
    let mut engine = build_engine();
    let result = engine.ask(&(t("socrates"), t("is_mortal"), t("?answer")));
    assert_eq!(result.answers.len(), 1);
    let answer = &result.answers[0];
    assert_eq!(answer.bindings.get("?answer"), Some(&Term::Bool(true)));
    let proof = result.proof(answer);
    assert!(proof.contains("mammals-are-mortal"));
    assert!(proof.contains("humans-are-mammals"));
    assert!(proof.contains("axiom in vial 'philosophers'"));
}

#[test]
fn deterministic_computation() {
    let mut engine = build_engine();
    let result = engine.ask(&(t("courtyard"), t("area"), t("?a")));
    assert_eq!(result.answers.len(), 1);
    assert_eq!(result.answers[0].bindings.get("?a"), Some(&Term::Int(360)));
}

#[test]
fn derived_facts_feed_further_rules() {
    // plaza is a square; area requires deriving rectangle-ness, width and
    // height first, then computing 25 * 25.
    let mut engine = build_engine();
    let result = engine.ask(&(t("plaza"), t("area"), t("?a")));
    assert_eq!(result.answers.len(), 1);
    assert_eq!(result.answers[0].bindings.get("?a"), Some(&Term::Int(625)));
}

#[test]
fn sparse_activation() {
    // A geometry query must not wake the philosophy or biology vials.
    let mut engine = build_engine();
    let result = engine.ask(&(t("courtyard"), t("area"), t("?a")));
    assert!(result.dormant.contains(&"philosophers".to_string()));
    assert!(result.dormant.contains(&"biology".to_string()));
}

#[test]
fn determinism() {
    let goal = (t("?who"), t("is_a"), t("mammal"));
    let first = build_engine().ask(&goal).summary();
    let second = build_engine().ask(&goal).summary();
    assert_eq!(first, second);
}

#[test]
fn confidence_propagates() {
    let mut engine = build_engine();
    let result = engine.ask(&(t("socrates"), t("is_mortal"), t("?answer")));
    // 0.999 (rule) * 0.99 (philosophers vial axiom) along the chain.
    assert!((result.answers[0].confidence - 0.999 * 0.99).abs() < 1e-6);
}

#[test]
fn no_proof_found() {
    let mut engine = build_engine();
    let result = engine.ask(&(t("zeus"), t("is_mortal"), t("?answer")));
    assert!(result.answers.is_empty());
}

#[test]
fn variable_subject_enumerates() {
    let mut engine = build_engine();
    let result = engine.ask(&(t("?who"), t("is_a"), t("mammal")));
    let mut who: Vec<&Term> = result
        .answers
        .iter()
        .filter_map(|a| a.bindings.get("?who"))
        .collect();
    who.sort();
    assert_eq!(who, vec![&t("plato"), &t("socrates")]);
}

// --- engine basics -------------------------------------------------------

#[test]
fn duplicate_vial_id_rejected() {
    let mut engine = Engine::new();
    engine.add_vial(Vial::new("a", "a")).unwrap();
    assert!(engine.add_vial(Vial::new("a", "a again")).is_err());
}

#[test]
fn tick_budget_halts_flood() {
    let mut engine = Engine::new();
    engine.max_ticks = 3;
    engine
        .add_vial(
            Vial::new("counter", "unbounded counting")
                .with_facts(vec![(t("n"), t("value"), Term::Int(0))])
                .with_rules(vec![Rule::new(
                    "successor",
                    vec![(t("n"), t("value"), t("?v"))],
                    (t("n"), t("value"), t("?next")),
                )
                .with_compute(Compute::expr("?next = ?v + 1").unwrap())]),
        )
        .unwrap();
    let result = engine.ask(&(t("n"), t("value"), t("?v")));
    assert_eq!(result.ticks, 3); // halted by budget, not by fixpoint
}

// --- functional predicate conflicts --------------------------------------

#[test]
fn functional_predicate_conflict() {
    let mut engine = Engine::new();
    engine.register_predicate("height", true);

    engine
        .add_vial(
            Vial::new("source_a", "Source A survey")
                .with_facts(vec![(t("tower of pizza in modesto ca"), t("height"), Term::Int(44))])
                .with_evidence(vec!["Survey A"]),
        )
        .unwrap();
    engine
        .add_vial(
            Vial::new("source_b", "Source B survey")
                .with_facts(vec![(t("tower of pizza in modesto ca"), t("height"), Term::Int(45))])
                .with_evidence(vec!["Survey B"]),
        )
        .unwrap();

    let result = engine.ask(&(t("tower of pizza in modesto ca"), t("height"), t("?h")));
    assert_eq!(result.answers.len(), 2);
    assert_eq!(result.conflicts.len(), 1);

    let (fact1, fact2) = &result.conflicts[0];
    assert_eq!(fact1.0, t("tower of pizza in modesto ca"));
    assert_eq!(fact1.1, t("height"));
    assert!(fact1.2 == Term::Int(44) || fact1.2 == Term::Int(45));
    assert!(fact2.2 == Term::Int(44) || fact2.2 == Term::Int(45));
    assert_ne!(fact1.2, fact2.2);

    let summary = result.summary();
    assert!(summary.contains("!!! CONFLICT WARNING !!!"));
    assert!(summary.contains("Conflict detected for functional predicate 'height'"));
    assert!(summary.contains("tower of pizza in modesto ca height 44"));
    assert!(summary.contains("tower of pizza in modesto ca height 45"));
}

// --- specificity ----------------------------------------------------------

#[test]
fn specificity_summary_output() {
    let mut engine = Engine::new();
    engine
        .add_vial(Vial::new("survey", "Location survey").with_facts(vec![
            (t("tower of pie in ajo arizona"), t("height"), Term::Int(55)),
            (t("tower of pie"), t("height"), Term::Int(54)),
        ]))
        .unwrap();

    let result = engine.ask(&(t("?what"), t("height"), t("?h")));
    assert_eq!(result.answers.len(), 2);

    let summary = result.summary();
    assert!(
        summary.contains(
            "Answer 2 ('tower of pie in ajo arizona') contains more detailed/specific context than Answer 1 ('tower of pie')"
        ),
        "summary was:\n{summary}"
    );
}

// --- SQLite store ----------------------------------------------------------

#[test]
fn save_and_load_vial() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteVialStore::open(dir.path().join("dsce.sqlite")).unwrap();

    let geometry = demo_vials().into_iter().find(|v| v.id == "geometry").unwrap();
    store.save_vial(&geometry).unwrap();

    let loaded = store.load_vial("geometry").unwrap();
    assert_eq!(loaded.id, "geometry");
    assert_eq!(loaded.concept, geometry.concept);
    assert_eq!(loaded.confidence, geometry.confidence);
    assert_eq!(loaded.neighbors, geometry.neighbors);
    assert_eq!(loaded.evidence, geometry.evidence);
    assert_eq!(loaded.rules.len(), geometry.rules.len());

    // Verify compute round-trip and execution.
    let rect_area = loaded.rules.iter().find(|r| r.name == "rectangle-area").unwrap();
    let compute = rect_area.compute.as_ref().expect("compute survived the round trip");
    let bindings = [("?w".to_string(), Term::Int(10)), ("?h".to_string(), Term::Int(20))]
        .into_iter()
        .collect();
    let extra = compute.eval(&bindings).unwrap();
    assert_eq!(extra.get("?a"), Some(&Term::Int(200)));
}

#[test]
fn database_backed_engine_ask() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteVialStore::open(dir.path().join("dsce.sqlite")).unwrap();
    for vial in demo_vials() {
        store.save_vial(&vial).unwrap();
    }

    // A database-backed engine initially has NO vials in memory.
    let mut engine = Engine::with_store(store);
    assert_eq!(engine.vials.len(), 0);

    // Socrates question: should activate (and load) philosophers + biology.
    let result = engine.ask(&(t("socrates"), t("is_mortal"), t("?answer")));
    assert_eq!(result.answers.len(), 1);
    assert_eq!(result.answers[0].bindings.get("?answer"), Some(&Term::Bool(true)));

    assert!(engine.vials.contains_key("philosophers"));
    assert!(engine.vials.contains_key("biology"));
    assert!(!engine.vials.contains_key("geometry"));
    assert!(!engine.vials.contains_key("measurements"));

    // Geometry question: now geometry + measurements get loaded too.
    let result2 = engine.ask(&(t("courtyard"), t("area"), t("?a")));
    assert_eq!(result2.answers.len(), 1);
    assert_eq!(result2.answers[0].bindings.get("?a"), Some(&Term::Int(360)));
    assert!(engine.vials.contains_key("geometry"));
    assert!(engine.vials.contains_key("measurements"));
}

#[test]
fn native_compute_cannot_be_saved() {
    let store = SqliteVialStore::open_in_memory().unwrap();
    let vial = Vial::new("native", "unserializable").with_rules(vec![Rule::new(
        "opaque",
        vec![(t("?x"), t("p"), t("?y"))],
        (t("?x"), t("q"), t("?y")),
    )
    .with_compute(Compute::native(|b| b.clone()))]);
    assert!(store.save_vial(&vial).is_err());
}

#[test]
fn seeding_skips_predicate_when_other_constants_exist() {
    // (socrates is_mortal ?x): "is_mortal" appears in the biology vial, but
    // the subject constant "socrates" must be the seed — flooding through
    // the predicate would be redundant here. Behavior is observable via
    // sparse activation staying intact on the geometry side.
    let mut engine = build_engine();
    let result = engine.ask(&(t("socrates"), t("is_mortal"), t("?x")));
    assert!(result.dormant.contains(&"geometry".to_string()));
    assert!(result.dormant.contains(&"measurements".to_string()));

    // A predicate-only goal still seeds the predicate (sole constant).
    let mut engine2 = build_engine();
    let result2 = engine2.ask(&(t("?s"), t("side"), t("?len")));
    assert_eq!(result2.answers.len(), 1, "predicate-only goal must still find the plaza side");
}
