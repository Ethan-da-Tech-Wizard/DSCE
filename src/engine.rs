//! The DSCE flood engine.
//!
//! Reasoning proceeds in discrete ticks:
//!
//! 1. A query is parsed into a goal pattern. Its constants become the
//!    first sand grains (the predicate seeds only when it is the sole
//!    constant — see SEEDING below).
//! 2. Each tick, grains wake every dormant vial indexed under their term,
//!    and waking vials wake their declared neighbors. With a database
//!    store attached, matching vials are loaded from SQLite on demand —
//!    dormant knowledge never even enters memory.
//! 3. Activated vials pour their axiom facts into shared working memory
//!    and fire their rules against everything derived so far. Rule
//!    matching across all active vials runs IN PARALLEL via rayon: each
//!    (vial, rule) pair matches against an immutable snapshot of working
//!    memory, and the results are merged single-threaded in a stable
//!    order, so parallelism never costs determinism.
//! 4. Each new fact emits fresh grains carrying its subject and object.
//!    The flood stops at fixpoint (no new facts, no new vials) or when
//!    the tick budget runs out. Answers are all working-memory facts
//!    matching the goal, each with a full proof tree.
//!
//! Every collection iterated at a decision point is sorted (`BTreeMap` /
//! `BTreeSet` / stable vectors) and no hash-order or randomness leaks in,
//! so the same knowledge base and query always produce the identical
//! result — the "deterministic" in DSCE.
//!
//! KNOWN COMPLEXITY LIMIT: premise matching is a naive join — for a rule
//! with P premises it tries every combination of working-memory facts,
//! O(|WM|^P) worst case, re-run every tick. Fine for prototype-scale
//! knowledge bases; indexed joins and semi-naive evaluation are the
//! planned fixes (see docs/MILESTONES.md, milestone M3).

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;

use crate::db_store::SqliteVialStore;
use crate::facts::{is_more_specific, substitute, unify, Bindings, Fact, Pattern, Term};
use crate::proof::{fact_str, render_proof, Derivation, Derivations};
use crate::sand::Grain;
use crate::vial::{Rule, Vial};

/// Properties the engine knows about a predicate, independent of any vial.
#[derive(Debug, Clone, Copy)]
pub struct PredicateInfo {
    /// A functional predicate maps each subject to AT MOST ONE object
    /// (e.g. `located_in`, `height`). Two facts sharing subject and
    /// predicate but disagreeing on the object are a CONFLICT.
    pub functional: bool,
    /// An ANNOTATION predicate (`emits_sand == false`) describes knowledge
    /// without spreading activation: facts under it never emit grains.
    /// Used for API-documentation predicates like `param`/`returns`, whose
    /// objects ("None", "size", ...) are shared vocabulary across many
    /// unrelated vials — letting them carry sand builds accidental bridges
    /// between libraries and destroys sparsity.
    pub emits_sand: bool,
}

impl Default for PredicateInfo {
    fn default() -> PredicateInfo {
        PredicateInfo {
            functional: false,
            emits_sand: true,
        }
    }
}

/// One fact that satisfied the goal, plus everything needed to trust it.
#[derive(Debug, Clone)]
pub struct Answer {
    /// The ground triple that matched the goal.
    pub fact: Fact,
    /// What each goal variable ended up equal to.
    pub bindings: Bindings,
    /// Confidence of the root derivation (already includes the whole chain).
    pub confidence: f64,
}

/// Everything the engine can tell you about one query.
///
/// Beyond the answers themselves, this captures the "flood telemetry" —
/// how long the flood ran, which vials it woke, which stayed dormant —
/// because sparse activation is a core claim of the architecture and
/// should be observable, not taken on faith. The settled working memory
/// (`derivations`) rides along so proofs can be rendered on demand.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub goal: Pattern,
    pub answers: Vec<Answer>,
    pub ticks: usize,
    /// Vial ids that received sand, in activation order.
    pub activated: Vec<String>,
    /// Vial ids the flood never reached (sorted).
    pub dormant: Vec<String>,
    /// Total sand grains emitted over the whole flood.
    pub grains: usize,
    /// Size of working memory when the flood settled.
    pub facts_derived: usize,
    /// Functional-predicate violations: (new fact, existing fact) pairs.
    pub conflicts: Vec<(Fact, Fact)>,
    /// The settled working memory: fact -> how it was derived.
    pub derivations: Derivations,
}

impl QueryResult {
    /// Render the proof tree for one answer.
    pub fn proof(&self, answer: &Answer) -> String {
        render_proof(&answer.fact, &self.derivations)
    }

    /// Render the whole result as human-readable text (used by the CLI).
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("goal: {}", fact_str(&self.goal)),
            format!(
                "flood: {} tick(s), {} grain(s) of sand, {}/{} vials activated, {} fact(s) in working memory",
                self.ticks,
                self.grains,
                self.activated.len(),
                self.activated.len() + self.dormant.len(),
                self.facts_derived
            ),
            format!(
                "activated vials: {}",
                if self.activated.is_empty() {
                    "(none)".to_string()
                } else {
                    self.activated.join(", ")
                }
            ),
            format!(
                "dormant vials:   {}",
                if self.dormant.is_empty() {
                    "(none)".to_string()
                } else {
                    self.dormant.join(", ")
                }
            ),
        ];

        if !self.conflicts.is_empty() {
            lines.push("\n!!! CONFLICT WARNING !!!".to_string());
            for (new_f, old_f) in &self.conflicts {
                lines.push(format!(
                    "Conflict detected for functional predicate '{}' on subject '{}':",
                    new_f.1, new_f.0
                ));
                lines.push(format!("  - {}", fact_str(new_f)));
                lines.push(format!("  - {}", fact_str(old_f)));
            }
            lines.push(String::new());
        }

        if self.answers.is_empty() {
            lines.push("no proof found.".to_string());
        }

        let wm_facts: Vec<&Fact> = self.derivations.keys().collect();
        for (i, answer) in self.answers.iter().enumerate() {
            lines.push(format!("answer {} (confidence {:.3}):", i + 1, answer.confidence));
            lines.push(self.proof(answer));

            // Flag answers whose subject carries more specific context than
            // another answer's subject (lexically or taxonomically).
            for (j, other) in self.answers.iter().enumerate() {
                if i != j && is_more_specific(&answer.fact.0, &other.fact.0, &wm_facts) {
                    lines.push(format!(
                        "  [Note] Answer {} ('{}') contains more detailed/specific context than Answer {} ('{}').",
                        i + 1,
                        answer.fact.0,
                        j + 1,
                        other.fact.0
                    ));
                }
            }
        }
        lines.join("\n")
    }
}

/// Holds the vial network and runs floods against it.
///
/// Typical use:
///
/// ```ignore
/// let mut engine = Engine::new();
/// engine.add_vial(some_vial)?;
/// let result = engine.ask(&goal);
/// println!("{}", result.summary());
/// ```
pub struct Engine {
    /// Safety valve: a rule like "n -> n+1" would otherwise derive new
    /// facts forever. After `max_ticks` rounds the flood is cut off even
    /// if it hasn't reached fixpoint.
    pub max_ticks: usize,
    /// All knowledge currently in memory, keyed by vial id. Sorted map so
    /// every enumeration (dormant list, index build) is deterministic.
    pub vials: BTreeMap<String, Vial>,
    /// Optional SQLite backing store; vials matching active terms are
    /// loaded from it on demand during a flood.
    pub store: Option<SqliteVialStore>,
    /// Schema metadata: predicate name -> properties (e.g. functional).
    pub predicates: BTreeMap<String, PredicateInfo>,
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new()
    }
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            max_ticks: 50,
            vials: BTreeMap::new(),
            store: None,
            predicates: BTreeMap::new(),
        }
    }

    pub fn with_store(store: SqliteVialStore) -> Engine {
        Engine {
            store: Some(store),
            ..Engine::new()
        }
    }

    /// Register schema metadata for a predicate (e.g. mark `located_in`
    /// or `height` as functional so conflicting values raise warnings).
    pub fn register_predicate(&mut self, name: impl Into<String>, functional: bool) {
        self.predicates.entry(name.into()).or_default().functional = functional;
    }

    /// Mark a predicate as pure annotation: facts under it are matched by
    /// rules as usual but never emit sand (see [`PredicateInfo::emits_sand`]).
    pub fn register_annotation(&mut self, name: impl Into<String>) {
        self.predicates.entry(name.into()).or_default().emits_sand = false;
    }

    fn predicate_emits_sand(&self, pred: &Term) -> bool {
        match pred {
            Term::Str(name) => self.predicates.get(name).map(|p| p.emits_sand).unwrap_or(true),
            _ => true,
        }
    }

    /// Register a vial. Ids must be unique — silently replacing knowledge
    /// would make results depend on registration order.
    pub fn add_vial(&mut self, vial: Vial) -> Result<(), String> {
        if self.vials.contains_key(&vial.id) {
            return Err(format!("duplicate vial id {:?}", vial.id));
        }
        self.vials.insert(vial.id.clone(), vial);
        Ok(())
    }

    /// Build the term -> vial-ids index over the in-memory vials.
    ///
    /// Vial ids are visited in sorted order (`BTreeMap` iteration) so
    /// that, for any term, the list of vials it maps to is always in the
    /// same order — one of the several places determinism is enforced.
    fn build_index(&self) -> BTreeMap<Term, Vec<String>> {
        let mut index: BTreeMap<Term, Vec<String>> = BTreeMap::new();
        for (vial_id, vial) in &self.vials {
            for term in vial.terms() {
                index.entry(term).or_default().push(vial_id.clone());
            }
        }
        index
    }

    /// All vial ids associated with `term`: in-memory index hits first,
    /// then database index hits. Vials found only in the database are
    /// loaded into memory here — this is the dynamic-load path that keeps
    /// dormant knowledge on disk.
    fn vials_for_term(&mut self, index: &BTreeMap<Term, Vec<String>>, term: &Term) -> Vec<String> {
        let mut ids: Vec<String> = index.get(term).cloned().unwrap_or_default();
        if let Some(store) = &self.store {
            for vial_id in store.get_vial_ids_for_term(term) {
                if !self.vials.contains_key(&vial_id) {
                    if let Ok(vial) = store.load_vial(&vial_id) {
                        self.vials.insert(vial_id.clone(), vial);
                    } else {
                        continue; // indexed but unloadable; skip it
                    }
                }
                if !ids.contains(&vial_id) {
                    ids.push(vial_id);
                }
            }
        }
        ids
    }

    /// Check a candidate fact against working memory for functional
    /// predicate violations: same subject and predicate, different object,
    /// predicate flagged functional -> conflict.
    fn check_conflict(&self, fact: &Fact, wm: &Derivations) -> Option<(Fact, Fact)> {
        let (subj, pred, obj) = fact;
        let functional = match pred {
            Term::Str(name) => self.predicates.get(name).map(|p| p.functional).unwrap_or(false),
            _ => false,
        };
        if !functional {
            return None;
        }
        for existing in wm.keys() {
            if &existing.0 == subj && &existing.1 == pred && &existing.2 != obj {
                return Some((fact.clone(), existing.clone()));
            }
        }
        None
    }

    /// Run one complete flood for one goal pattern. The main algorithm.
    ///
    /// Takes `&mut self` because a database-backed engine loads activated
    /// vials into memory as the flood reaches them.
    pub fn ask(&mut self, goal: &Pattern) -> QueryResult {
        self.ask_with_facts(goal, &[])
    }

    /// Like [`Engine::ask`], but pours `extra_facts` into working memory
    /// BEFORE the flood starts. This is the Semantic Harvester's entry
    /// point: dynamic vocabulary triples (e.g.
    /// `("scoring_system", "is_a", "state_machine")`) asserted for one
    /// query only, mapping the user's words onto the knowledge base's
    /// generic terms. The poured facts also emit seed sand, so they steer
    /// which vials wake.
    pub fn ask_with_facts(&mut self, goal: &Pattern, extra_facts: &[Fact]) -> QueryResult {
        let index = self.build_index();

        // Working memory: every fact known so far in THIS query, mapped to
        // the record of how it got there. Doubles as the proof store.
        let mut wm: Derivations = BTreeMap::new();
        // Woken vials: membership set + activation order.
        let mut active: BTreeSet<String> = BTreeSet::new();
        let mut activation_order: Vec<String> = Vec::new();
        let mut conflicts: Vec<(Fact, Fact)> = Vec::new();

        // --- SEEDING ----------------------------------------------------
        // The goal's constants become the first sand. The predicate
        // position is skipped whenever the subject or object supplies a
        // constant: generic predicates like `is_a` appear in nearly every
        // vial, and letting them carry sand floods the entire network. If
        // the predicate is the goal's ONLY constant, it does seed — it is
        // then the sole lead worth following.
        let mut seeds: Vec<Term> = Vec::new();
        if !goal.0.is_variable() {
            seeds.push(goal.0.clone());
        }
        if !goal.2.is_variable() {
            seeds.push(goal.2.clone());
        }
        if seeds.is_empty() && !goal.1.is_variable() {
            seeds.push(goal.1.clone());
        }
        let mut grains: Vec<Grain> = seeds.into_iter().map(|t| Grain::new(t, "query", 0)).collect();

        // Extra facts asserted by the query itself (harvested vocabulary)
        // enter working memory as query-owned axioms and emit seed sand
        // for their subject and object, exactly like derived facts do.
        for fact in extra_facts {
            if !wm.contains_key(fact) {
                if let Some(conflict) = self.check_conflict(fact, &wm) {
                    conflicts.push(conflict);
                }
                wm.insert(
                    fact.clone(),
                    Derivation {
                        fact: fact.clone(),
                        vial_id: "query".to_string(),
                        confidence: 1.0,
                        rule_name: None,
                        premises: Vec::new(),
                        evidence: vec!["asserted by query".to_string()],
                    },
                );
                grains.push(Grain::new(fact.0.clone(), "query", 0));
                grains.push(Grain::new(fact.2.clone(), "query", 0));
            }
        }
        let mut total_grains = grains.len();
        let mut ticks = 0;

        // --- THE FLOOD LOOP ----------------------------------------------
        for tick in 1..=self.max_ticks {
            // STEP 1: sand wakes dormant vials. Every grain looks up its
            // term in the index (and the database index, loading vials on
            // demand) and activates any vial found there.
            let mut newly_active: Vec<String> = Vec::new();
            for grain in &grains {
                for vial_id in self.vials_for_term(&index, &grain.term) {
                    if active.insert(vial_id.clone()) {
                        activation_order.push(vial_id.clone());
                        newly_active.push(vial_id);
                    }
                }
            }

            // STEP 2: waking is contagious along declared neighbor links.
            // The list grows while being walked, so activation chases
            // neighbor chains transitively within a single tick: A wakes
            // B, B's neighbors wake too, and so on. Neighbors only present
            // in the database are loaded here.
            let mut i = 0;
            while i < newly_active.len() {
                let neighbors = self.vials[&newly_active[i]].neighbors.clone();
                for neighbor in neighbors {
                    if !self.vials.contains_key(&neighbor) {
                        if let Some(store) = &self.store {
                            if let Ok(vial) = store.load_vial(&neighbor) {
                                self.vials.insert(neighbor.clone(), vial);
                            }
                        }
                    }
                    if self.vials.contains_key(&neighbor) && active.insert(neighbor.clone()) {
                        activation_order.push(neighbor.clone());
                        newly_active.push(neighbor);
                    }
                }
                i += 1;
            }

            // STEP 3: newly woken vials POUR their axiom facts into
            // working memory. Facts are added in sorted order (another
            // determinism point) and each remembers which vial it came
            // from, that vial's evidence, and its confidence.
            let mut new_facts: Vec<Fact> = Vec::new();
            for vial_id in &newly_active {
                let vial = &self.vials[vial_id];
                let mut poured: Vec<Fact> = vial.facts.clone();
                poured.sort();
                let derivations: Vec<Derivation> = poured
                    .iter()
                    .map(|fact| Derivation {
                        fact: fact.clone(),
                        vial_id: vial_id.clone(),
                        confidence: vial.confidence,
                        rule_name: None,
                        premises: Vec::new(),
                        evidence: vial.evidence.clone(),
                    })
                    .collect();
                for (fact, derivation) in poured.into_iter().zip(derivations) {
                    if !wm.contains_key(&fact) {
                        if let Some(conflict) = self.check_conflict(&fact, &wm) {
                            conflicts.push(conflict);
                        }
                        wm.insert(fact.clone(), derivation);
                        new_facts.push(fact);
                    }
                }
            }

            // STEP 4: every ACTIVE vial (not just the new ones) fires its
            // rules against everything in working memory — IN PARALLEL.
            // Each (vial, rule) pair works against the same immutable
            // snapshot of wm, so the matches are embarrassingly parallel;
            // rayon's ordered collect keeps the batches in (activation
            // order, rule order), and the single-threaded merge below
            // applies "first derivation wins" in that stable order.
            // Parallelism therefore never costs determinism.
            let firings: Vec<(&Vial, &Rule)> = activation_order
                .iter()
                .filter_map(|vial_id| self.vials.get(vial_id))
                .flat_map(|vial| vial.rules.iter().map(move |rule| (vial, rule)))
                .collect();
            let batches: Vec<Vec<(Fact, Derivation)>> = firings
                .par_iter()
                .map(|(vial, rule)| {
                    match_premises(&rule.premises, &wm)
                        .into_iter()
                        .filter_map(|bindings| conclude(rule, vial, bindings, &wm))
                        .collect()
                })
                .collect();
            for (fact, derivation) in batches.into_iter().flatten() {
                // A fact that is already known is skipped — first
                // derivation wins, which keeps proofs stable and floods
                // finite.
                if !wm.contains_key(&fact) {
                    if let Some(conflict) = self.check_conflict(&fact, &wm) {
                        conflicts.push(conflict);
                    }
                    wm.insert(fact.clone(), derivation);
                    new_facts.push(fact);
                }
            }

            // STEP 5: new facts become new sand for the NEXT tick. Only
            // the subject and object emit grains — predicates are
            // relations, not entities, and letting them carry sand wakes
            // the entire network and destroys sparsity. Facts under
            // ANNOTATION predicates (API documentation like param/returns)
            // emit nothing at all.
            grains = new_facts
                .iter()
                .filter(|fact| self.predicate_emits_sand(&fact.1))
                .flat_map(|fact| {
                    let origin = wm[fact].vial_id.clone();
                    [
                        Grain::new(fact.0.clone(), origin.clone(), tick),
                        Grain::new(fact.2.clone(), origin, tick),
                    ]
                })
                .collect();
            total_grains += grains.len();
            ticks = tick;

            // STEP 6: fixpoint check. If this tick derived no new facts
            // AND woke no new vials, the next tick would be identical —
            // the sand has settled, stop flooding.
            if new_facts.is_empty() && newly_active.is_empty() {
                break;
            }
        }

        // --- ANSWER EXTRACTION --------------------------------------------
        // Scan settled working memory (sorted BTreeMap iteration, so answer
        // order is reproducible) for every fact that unifies with the goal.
        let empty = Bindings::new();
        let answers: Vec<Answer> = wm
            .iter()
            .filter_map(|(fact, derivation)| {
                unify(goal, fact, &empty).map(|bindings| Answer {
                    fact: fact.clone(),
                    bindings,
                    confidence: derivation.confidence,
                })
            })
            .collect();
        let dormant: Vec<String> = self
            .vials
            .keys()
            .filter(|id| !active.contains(*id))
            .cloned()
            .collect();
        QueryResult {
            goal: goal.clone(),
            answers,
            ticks,
            activated: activation_order,
            dormant,
            grains: total_grains,
            facts_derived: wm.len(),
            conflicts,
            derivations: wm,
        }
    }
}

/// Find every set of variable bindings that satisfies ALL premises.
///
/// Works like a database join, built premise by premise:
/// - Start with one empty candidate binding.
/// - For premise 1, try to unify it with every fact in working memory;
///   each success produces an extended candidate.
/// - For premise 2, extend each surviving candidate against every fact
///   again — bindings made by premise 1 constrain premise 2, because
///   `unify` rejects a fact that contradicts them.
/// - ...and so on. What survives all premises is returned.
///
/// COMPLEXITY: with P premises and |WM| facts this can inspect up to
/// |WM|^P combinations — the naive join flagged in the module docs. The
/// `BTreeMap` iterates facts in sorted order, so the output order (and
/// everything downstream) is deterministic.
fn match_premises(premises: &[Pattern], wm: &Derivations) -> Vec<Bindings> {
    let mut results = vec![Bindings::new()];
    for premise in premises {
        let mut extended = Vec::new();
        for bindings in &results {
            for fact in wm.keys() {
                if let Some(unified) = unify(premise, fact, bindings) {
                    extended.push(unified);
                }
            }
        }
        results = extended;
        if results.is_empty() {
            break; // some premise is unsatisfiable; no point continuing
        }
    }
    results
}

/// Turn one successful premise match into a concrete derived fact.
///
/// Three stages:
/// 1. If the rule has a `compute` hook, run it to derive EXTRA bindings
///    deterministically from the matched ones. A failed computation
///    (unbound input, division by zero) skips this firing.
/// 2. Substitute all bindings into the conclusion pattern to get a ground
///    fact. An unbound conclusion variable means the rule is malformed
///    for this match; skip rather than poison the flood.
/// 3. Build the Derivation record with the combined confidence:
///
///    ```text
///        rule.confidence x vial.confidence x min(premise confidences)
///    ```
///
///    i.e. a conclusion is never MORE trusted than its shakiest premise,
///    further discounted by how much the rule and its home vial are
///    trusted.
fn conclude(rule: &Rule, vial: &Vial, bindings: Bindings, wm: &Derivations) -> Option<(Fact, Derivation)> {
    let bindings = match &rule.compute {
        Some(compute) => {
            let extra = compute.eval(&bindings)?;
            let mut merged = bindings;
            merged.extend(extra);
            merged
        }
        None => bindings,
    };
    let fact = substitute(&rule.conclusion, &bindings).ok()?;
    let premises: Vec<Fact> = rule
        .premises
        .iter()
        .map(|p| substitute(p, &bindings))
        .collect::<Result<_, _>>()
        .ok()?;
    let premise_confidence = premises
        .iter()
        .map(|p| wm.get(p).map(|d| d.confidence).unwrap_or(1.0))
        .fold(f64::INFINITY, f64::min)
        .min(1.0);
    let derivation = Derivation {
        fact: fact.clone(),
        vial_id: vial.id.clone(),
        confidence: rule.confidence * vial.confidence * premise_confidence,
        rule_name: Some(rule.name.clone()),
        premises,
        evidence: Vec::new(),
    };
    Some((fact, derivation))
}
