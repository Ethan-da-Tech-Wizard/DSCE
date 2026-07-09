"""The DSCE flood engine.

Reasoning proceeds in discrete ticks:

  1. A query is parsed into a goal pattern. Its constants become the first
     sand grains.
  2. Each tick, grains wake up every dormant vial indexed under their term,
     and firing vials wake their declared neighbors.
  3. Activated vials pour their axiom facts into shared working memory and
     fire their rules against everything derived so far. Each new fact
     emits fresh grains carrying its terms.
  4. The flood stops at fixpoint (no new facts, no new vials) or when the
     tick budget runs out. Answers are all working-memory facts matching
     the goal, each with a full proof tree.

Everything is iterated in sorted order and no randomness is used, so the
same knowledge base and query always produce the identical result — the
"deterministic" in DSCE.

DETAILED FLOOD DYNAMICS
=======================
The "sand" analogy is literal spreading activation. A grain is not data; it
is a marker that "someone is thinking about term T". Vials are indexed by
every term they mention. When term T is active, every vial indexed under T
wakes up.

Neighbor links allow topological propagation: if vial A wakes up, it waken
its neighbors in the same tick. This lets a query about a Socrates (in vial
philosophers) flow automatically to biology rules, even if Socrates was
never mentioned in the biology vial.

Axioms are poured once per vial activation. Rules are fired continuously on
every tick. When a rule concludes a new fact, that fact's terms (subject
and object) are cast into the next tick's sand.

DETERMINISTIC STABILITY
=======================
To guarantee byte-identical execution, every collection is iterated in sorted
order (see facts.sort_key) at every decision point, and no database or hash
randomness is used anywhere. Python dicts preserve insertion order, so the
working memory and active-vial set are themselves reproducible. Same
knowledge base + same query = byte-identical result, every run.

KNOWN COMPLEXITY LIMIT (important!)
===================================
`_match` (premise matching) is a NAIVE JOIN: for a rule with P premises it
tries every combination of working-memory facts, which is O(|WM|^P) in the
worst case. It also re-derives every previously derived fact on every tick
(re-derivations are detected and discarded, but the matching work is still
spent). This is perfectly fine for a prototype knowledge base of dozens of
facts, and hopeless for one with millions. The planned fixes — indexed
joins (Rete-style networks) and semi-naive evaluation (only join against
facts that are NEW since the last tick) — are milestone M3 in
docs/MILESTONES.md and requirement SRD-P2 in docs/SRD.md.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from dsce.facts import Bindings, Fact, Pattern, constants, sort_key, substitute, unify, is_more_specific
from dsce.proof import Derivation, Proof, fact_str
from dsce.sand import Grain
from dsce.vial import Rule, Vial


@dataclass(frozen=True)
class Answer:
    """One fact that satisfied the goal, plus everything needed to trust it.

    Attributes:
        fact:     the ground triple that matched the goal,
                  e.g. ("socrates", "is_mortal", True).
        bindings: what each goal variable ended up equal to,
                  e.g. {"?answer": True}.
        proof:    the full derivation tree — walk it (or .render() it) to
                  see exactly which axioms and rules produced this fact.
    """

    fact: Fact
    bindings: dict
    proof: Proof

    @property
    def confidence(self) -> float:
        """Confidence of the root derivation (already includes the whole chain)."""
        return self.proof.confidence


@dataclass
class Result:
    """Everything the engine can tell you about one query.

    Beyond the answers themselves, this captures the "flood telemetry" —
    how long the flood ran, which vials it woke, which stayed dormant —
    because sparse activation is a core claim of the architecture and
    should be observable, not taken on faith.
    """

    goal: Pattern          # the pattern that was asked
    answers: list          # list of Answer, sorted deterministically
    ticks: int             # how many flood rounds ran before settling
    activated: tuple       # vial ids that received sand, in activation order
    dormant: tuple         # vial ids the flood never reached (sorted)
    grains: int            # total sand grains emitted over the whole flood
    facts_derived: int     # size of working memory when the flood settled
    conflicts: tuple = ()  # functional predicate conflict violations

    def summary(self) -> str:
        """Render the whole result as human-readable text (used by the CLI)."""
        lines = [
            f"goal: {fact_str(self.goal)}",
            f"flood: {self.ticks} tick(s), {self.grains} grain(s) of sand, "
            f"{len(self.activated)}/{len(self.activated) + len(self.dormant)} vials activated, "
            f"{self.facts_derived} fact(s) in working memory",
            f"activated vials: {', '.join(self.activated) if self.activated else '(none)'}",
            f"dormant vials:   {', '.join(self.dormant) if self.dormant else '(none)'}",
        ]
        
        if self.conflicts:
            lines.append("\n!!! CONFLICT WARNING !!!")
            for new_f, old_f in self.conflicts:
                lines.append(f"Conflict detected for functional predicate '{new_f[1]}' on subject '{new_f[0]}':")
                lines.append(f"  - {fact_str(new_f)}")
                lines.append(f"  - {fact_str(old_f)}")
            lines.append("")
            
        if not self.answers:
            lines.append("no proof found.")
            
        for i, answer in enumerate(self.answers, 1):
            lines.append(f"answer {i} (confidence {answer.confidence:.3f}):")
            lines.append(answer.proof.render())
            
            # Check if this answer contains more specific context than any other answer
            specificity_notes = []
            for j, other in enumerate(self.answers, 1):
                if i != j:
                    if is_more_specific(answer.fact[0], other.fact[0], answer.proof.derivations):
                        specificity_notes.append(
                            f"Answer {i} ('{answer.fact[0]}') contains more detailed/specific context than Answer {j} ('{other.fact[0]}')."
                        )
            if specificity_notes:
                for note in specificity_notes:
                    lines.append(f"  [Note] {note}")
                    
        return "\n".join(lines)


class Engine:
    """Holds the vial network and runs floods against it.

    Typical use:

        engine = Engine()
        engine.add_vial(some_vial)
        engine.add_vial(another_vial)
        result = engine.ask(("socrates", "is_mortal", "?answer"))
        print(result.summary())
    """

    def __init__(self, max_ticks: int = 50, store=None):
        # Safety valve: a rule like "n -> n+1" would otherwise derive new
        # facts forever. After max_ticks rounds the flood is cut off even
        # if it hasn't reached fixpoint. 50 is generous for the demo KB
        # (its floods settle in 2 ticks).
        self.max_ticks = max_ticks
        # All knowledge, keyed by vial id. Insertion order is irrelevant
        # because every consumer iterates it sorted.
        self.vials: dict = {}  # id -> Vial
        # term -> tuple of vial ids that mention that term. This is how a
        # sand grain finds which vials to wake. Built lazily by _index()
        # and invalidated whenever a vial is added.
        self._term_index: dict = {}
        self.store = store
        self.predicates: dict = {}  # predicate name -> properties

    def register_predicate(self, name: str, functional: bool = False) -> None:
        self.predicates[name] = {"functional": functional}

    def _check_conflict(self, fact: Fact, wm: dict) -> Optional[tuple]:
        subj, pred, obj = fact
        if pred in self.predicates and self.predicates[pred].get("functional", False):
            for existing_fact in wm:
                e_subj, e_pred, e_obj = existing_fact
                if e_subj == subj and e_pred == pred and e_obj != obj:
                    return (fact, existing_fact)
        return None

    def add_vial(self, vial: Vial) -> None:
        """Register a vial. Ids must be unique — silently replacing
        knowledge would make results depend on registration order."""
        if vial.id in self.vials:
            raise ValueError(f"duplicate vial id {vial.id!r}")
        self.vials[vial.id] = vial
        self._term_index = {}  # stale now; rebuilt lazily on next ask()

    def _index(self) -> dict:
        """Build (or return the cached) term -> vial-ids index.

        Vial ids are visited in sorted order so that, for any term, the
        tuple of vials it maps to is always in the same order — one of the
        several places determinism is enforced.
        """
        if not self._term_index:
            index: dict = {}
            for vial_id in sorted(self.vials):
                for term in self.vials[vial_id].terms():
                    index.setdefault(term, []).append(vial_id)
            self._term_index = {term: tuple(ids) for term, ids in index.items()}
        return self._term_index

    def ask(self, goal: Pattern) -> Result:
        """Run one complete flood for one goal pattern. The main algorithm.

        Data structures used throughout (all insertion-ordered dicts, so
        iteration order is reproducible):

          wm ("working memory"): Fact -> Derivation. Every fact known so
              far in THIS query, mapped to the record of how it got there
              (axiom from a vial, or rule firing with premises). This
              doubles as the proof store: proofs are just walks over wm.
          active: vial id -> None, used as an ordered set of woken vials.
          grains: the sand currently in flight — grains created last tick,
              waiting to wake vials this tick.
        """
        index = self._index()
        wm: dict = {}
        active: dict = {}

        # --- SEEDING ---------------------------------------------------
        # The goal's constants become the first sand. For the goal
        # ("socrates", "is_mortal", "?answer") that is two grains:
        # "socrates" and "is_mortal". Variables ("?answer") carry no
        # information, so they emit nothing. Note that the goal's
        # PREDICATE does seed (unlike derived facts below — see the
        # comment there): the user's own words are always worth following.
        grains: list = [Grain(term, "query", 0) for term in constants(goal)]
        total_grains = len(grains)
        ticks = 0
        conflicts = []

        def get_vials_for_term(term):
            v_ids = list(index.get(term, ()))
            if self.store is not None:
                store_ids = self.store.get_vial_ids_for_term(term)
                for v_id in store_ids:
                    if v_id not in self.vials:
                        self.vials[v_id] = self.store.load_vial(v_id)
                    if v_id not in v_ids:
                        v_ids.append(v_id)
            return tuple(v_ids)

        # --- THE FLOOD LOOP ---------------------------------------------
        for tick in range(1, self.max_ticks + 1):

            # STEP 1: sand wakes dormant vials. Every grain looks up its
            # term in the index and activates any vial found there that
            # isn't active yet.
            newly_active = []
            for grain in grains:
                for vial_id in get_vials_for_term(grain.term):
                    if vial_id not in active:
                        active[vial_id] = None
                        newly_active.append(vial_id)

            # STEP 2: waking is contagious along declared neighbor links.
            # Note this loop iterates a list that grows while being
            # iterated (`newly_active.append` inside), so activation
            # chases neighbor chains transitively within a single tick:
            # A wakes B, B's neighbors wake too, and so on.
            for vial_id in list(newly_active):
                for neighbor in self.vials[vial_id].neighbors:
                    if self.store is not None and neighbor not in self.vials:
                        self.vials[neighbor] = self.store.load_vial(neighbor)
                    if neighbor in self.vials and neighbor not in active:
                        active[neighbor] = None
                        newly_active.append(neighbor)

            # STEP 3: newly woken vials pour their axiom facts into
            # working memory. Facts are added in sorted order (another
            # determinism point) and each remembers which vial it came
            # from, that vial's evidence sources, and its confidence.
            new_facts = []
            for vial_id in newly_active:
                vial = self.vials[vial_id]
                for fact in sorted(vial.facts, key=sort_key):
                    if fact not in wm:
                        conflict = self._check_conflict(fact, wm)
                        if conflict:
                            conflicts.append(conflict)
                        wm[fact] = Derivation(
                            fact=fact,
                            vial_id=vial_id,
                            confidence=vial.confidence,
                            evidence=vial.evidence,
                        )
                        new_facts.append(fact)

            # STEP 4: every ACTIVE vial (not just the new ones) fires its
            # rules against everything in working memory. _match returns
            # every way the rule's premises can be satisfied; _conclude
            # instantiates the conclusion for each. A fact that is already
            # known is skipped — first derivation wins, which keeps proofs
            # stable and floods finite.
            #
            # PERFORMANCE NOTE: this is the naive O(|WM|^P) join described
            # in the module docstring, and it re-matches old facts every
            # tick. Fine at prototype scale; milestone M3 replaces it.
            for vial_id in active:
                vial = self.vials[vial_id]
                for rule in vial.rules:
                    for bindings in self._match(rule.premises, wm):
                        fact, derivation = self._conclude(rule, vial, bindings, wm)
                        if fact is not None and fact not in wm:
                            conflict = self._check_conflict(fact, wm)
                            if conflict:
                                conflicts.append(conflict)
                            wm[fact] = derivation
                            new_facts.append(fact)

            # STEP 5: new facts become new sand for the NEXT tick. Only
            # the subject and object emit grains — the predicate position
            # is deliberately skipped. Predicates are relations, not
            # entities, and generic ones like "is_a" appear in nearly
            # every vial; letting them carry sand was observed (during
            # development, caught by test_sparse_activation) to wake the
            # entire network and destroy sparsity.
            grains = []
            for fact in new_facts:
                subject, _, obj = fact
                grains.append(Grain(subject, wm[fact].vial_id, tick))
                grains.append(Grain(obj, wm[fact].vial_id, tick))
            total_grains += len(grains)
            ticks = tick

            # STEP 6: fixpoint check. If this tick derived no new facts
            # AND woke no new vials, the next tick would be identical —
            # the sand has settled, stop flooding.
            if not new_facts and not newly_active:
                break

        # --- ANSWER EXTRACTION -------------------------------------------
        # Scan settled working memory (sorted, for reproducible answer
        # order) for every fact that unifies with the goal. Each match
        # yields an Answer wrapping the variable bindings and a Proof
        # rooted at that fact.
        answers = []
        for fact in sorted(wm, key=sort_key):
            bindings = unify(goal, fact, {})
            if bindings is not None:
                answers.append(Answer(fact=fact, bindings=bindings, proof=Proof(fact, wm)))
        dormant = tuple(v for v in sorted(self.vials) if v not in active)
        return Result(
            goal=goal,
            answers=answers,
            ticks=ticks,
            activated=tuple(active),
            dormant=dormant,
            grains=total_grains,
            facts_derived=len(wm),
            conflicts=tuple(conflicts),
        )

    def _match(self, premises: tuple, wm: dict):
        """Find every set of variable bindings that satisfies ALL premises.

        Works like a database join, built premise by premise:

          - Start with one empty candidate binding: [{}].
          - For premise 1, try to unify it with every fact in working
            memory; each success produces an extended candidate.
          - For premise 2, extend each surviving candidate against every
            fact again — bindings made by premise 1 constrain premise 2,
            because unify() rejects a fact that contradicts them.
          - ... and so on. What survives all premises is returned.

        Example: premises (("?x", "is_a", "human"),) against a wm holding
        (socrates is_a human) and (plato is_a human) returns
        [{"?x": "plato"}, {"?x": "socrates"}] (in sorted-fact order).

        COMPLEXITY: with P premises and |WM| facts this can inspect up to
        |WM|^P combinations — the naive join flagged in the module
        docstring. Facts are pre-sorted once so the output order (and
        therefore everything downstream) is deterministic.
        """
        results = [{}]
        facts = sorted(wm, key=sort_key)
        for premise in premises:
            extended = []
            for bindings in results:
                for fact in facts:
                    unified = unify(premise, fact, bindings)
                    if unified is not None:
                        extended.append(unified)
            results = extended
            if not results:
                break  # some premise is unsatisfiable; no point continuing
        return results

    def _conclude(self, rule: Rule, vial: Vial, bindings: Bindings, wm: dict):
        """Turn one successful premise match into a concrete derived fact.

        Three stages:
          1. If the rule has a `compute` hook, run it to derive EXTRA
             bindings deterministically from the matched ones (e.g.
             {"?a": 12 * 30} for the rectangle-area rule). This is what
             lets conclusions contain values no premise supplied.
          2. Substitute all bindings into the conclusion pattern to get a
             ground fact. If a conclusion variable is still unbound the
             rule is malformed for this match; we skip it (return None)
             rather than crash the whole flood.
          3. Build the Derivation record: which rule, which vial, which
             ground premises were consumed (these become the proof tree's
             children), and the combined confidence:

                 rule.confidence x vial.confidence x min(premise confidences)

             i.e. a conclusion is never MORE trusted than its shakiest
             premise, further discounted by how much the rule and its home
             vial are trusted.
        """
        if rule.compute is not None:
            bindings = {**bindings, **rule.compute(bindings)}
        try:
            fact = substitute(rule.conclusion, bindings)
        except ValueError:
            return None, None
        premises = tuple(substitute(p, bindings) for p in rule.premises)
        premise_confidence = min(wm[p].confidence for p in premises) if premises else 1.0
        return fact, Derivation(
            fact=fact,
            vial_id=vial.id,
            confidence=rule.confidence * vial.confidence * premise_confidence,
            rule_name=rule.name,
            premises=premises,
        )
