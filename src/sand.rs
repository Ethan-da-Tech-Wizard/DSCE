//! Sand: the activation particles of the DSCE.
//!
//! Sand is NOT data — it is activation, the thing that decides which
//! knowledge participates in answering a given question. Where a
//! transformer multiplies every weight for every token, the DSCE lets
//! activation FLOW: grains spread outward from the question through the
//! vial network, and only vials a grain actually reaches wake up and do
//! work.
//!
//! Each grain carries exactly one TERM — a constant such as "socrates" or
//! "rectangle" — plus provenance (who emitted it, and when). Grains come
//! from two places:
//!
//! 1. SEEDING: constants of the goal pattern itself (origin "query",
//!    tick 0). The predicate position is skipped whenever the subject or
//!    object supplies a constant: predicates are relations, not entities,
//!    and generic ones like `is_a` appear in nearly every vial — letting
//!    them carry sand floods the entire network and destroys sparsity.
//!    Only when the predicate is the goal's SOLE constant does it seed.
//! 2. DERIVATION: every new fact emits grains for its SUBJECT and OBJECT
//!    (origin = the vial whose rule/axiom produced the fact); the
//!    predicate never emits.
//!
//! A grain does one job: its term is looked up in the engine's
//! term-to-vials index, and any dormant vial found there is woken. Grains
//! live for exactly one tick; facts persist, sand does not.

use std::fmt;

use crate::facts::Term;

/// One activation particle. A record of something that happened — never
/// mutated after creation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Grain {
    /// The single constant this grain carries.
    pub term: Term,
    /// `"query"`, or the id of the vial whose derivation emitted it.
    pub origin: String,
    /// Flood round on which this grain was created (0 = seeding).
    pub tick: usize,
}

impl Grain {
    pub fn new(term: Term, origin: impl Into<String>, tick: usize) -> Grain {
        Grain {
            term,
            origin: origin.into(),
            tick,
        }
    }
}

impl fmt::Display for Grain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "grain({:?} from {} @ tick {})", self.term, self.origin, self.tick)
    }
}
