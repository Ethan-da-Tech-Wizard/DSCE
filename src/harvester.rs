//! The Semantic Harvester — the AI/DE bridge between natural language and
//! the Datalog engine.
//!
//! The engine only understands triples; users speak sentences. The
//! harvester translates a creative, high-level request such as
//!
//! ```text
//!     "Make a multiplayer grid game with a scoring system"
//! ```
//!
//! into the two artifacts the engine needs:
//!
//! 1. A DATALOG QUERY GOAL — the formal pattern that asks for a program:
//!    `("multiplayer_grid_game", "code", "?code")`.
//! 2. DYNAMIC VOCABULARY TRIPLES — one-query synonym assertions such as
//!    `("scoring_system", "is_a", "state_machine")` poured into working
//!    memory before reasoning (via [`crate::engine::Engine::ask_with_facts`]),
//!    mapping the user's words onto the generic design-pattern vials.
//!
//! Two harvesting paths:
//!
//! - [`prompt_for`] renders [`PROMPT_TEMPLATE`] for an LLM; feed the model's
//!   strict-JSON reply to [`parse_response`]. This is the full AI/DE
//!   compiler front end — the LLM does creative interpretation ONCE, then
//!   the engine reasons deterministically forever after.
//! - [`harvest_offline`] is a deterministic keyword harvester covering the
//!   built-in vocabulary. No network, no model, same output every time —
//!   this is what the CLI uses, and what tests pin down.

use serde_json::Value;

use crate::db_store::triple_from_json;
use crate::facts::{Fact, Pattern, Term};

/// The prompt template for LLM-backed harvesting. Replace `{{REQUEST}}`
/// with the user's request (or use [`prompt_for`]) and parse the model's
/// reply with [`parse_response`].
pub const PROMPT_TEMPLATE: &str = r#"You are the Semantic Harvester for the DSCE generic software assembler.
Translate the user's software request into a Datalog goal and vocabulary triples.

The knowledge base understands this generic vocabulary:
  capabilities (via "needs"):    graphics, grid_layout, networking, persistence
  feature classes (via "is_a"):  state_machine, application
  predicates:                    needs, has_feature, is_a

Rules:
- Invent one snake_case application name derived from the request.
- Always assert [app, "is_a", "application"].
- Map every requested capability to a [app, "needs", capability] triple.
- Map stateful behaviors (scores, turns, phases, modes) to a named feature:
  [app, "has_feature", feature] plus [feature, "is_a", "state_machine"].
- Use ONLY the vocabulary above; do not invent new predicates or capabilities.

Reply with STRICT JSON, no prose, in exactly this shape:
{"goal": ["<app>", "code", "?code"],
 "triples": [["<app>", "is_a", "application"], ...]}

USER REQUEST: {{REQUEST}}
"#;

/// Render the harvesting prompt for one request.
pub fn prompt_for(request: &str) -> String {
    PROMPT_TEMPLATE.replace("{{REQUEST}}", request)
}

/// What a harvest produces: the app slug, the query goal, and the
/// vocabulary triples to pour before reasoning.
#[derive(Debug, Clone)]
pub struct Harvest {
    pub app: String,
    pub goal: Pattern,
    pub triples: Vec<Fact>,
}

/// Parse a strict-JSON harvester reply (LLM output) into a [`Harvest`].
pub fn parse_response(json: &str) -> Result<Harvest, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| format!("bad harvest JSON: {e}"))?;
    let goal = triple_from_json(
        value.get("goal").ok_or_else(|| "harvest JSON missing \"goal\"".to_string())?,
    )
    .map_err(|e| format!("bad goal: {e}"))?;
    let triples = value
        .get("triples")
        .and_then(Value::as_array)
        .ok_or_else(|| "harvest JSON missing \"triples\" array".to_string())?
        .iter()
        .map(|t| triple_from_json(t).map_err(|e| format!("bad triple: {e}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Harvest {
        app: goal.0.to_string(),
        goal,
        triples,
    })
}

/// Words that carry no naming or capability signal.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "of", "for", "to", "in", "on", "that", "this", "some", "any",
    "with", "using", "make", "build", "create", "write", "made", "me", "my", "please", "i", "want",
    "app", "application", "program", "software", "simple",
];

fn matches_any(tokens: &[String], keywords: &[&str]) -> bool {
    tokens.iter().any(|t| keywords.contains(&t.as_str()))
}

/// Deterministic, dependency-free harvesting: keyword-match the request
/// against the built-in vocabulary. Same request, same harvest, every time.
pub fn harvest_offline(request: &str) -> Harvest {
    let tokens: Vec<String> = request
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect();

    // The app name: the first three significant words, joined snake_case.
    let significant: Vec<&str> = tokens
        .iter()
        .map(String::as_str)
        .filter(|t| !STOPWORDS.contains(t))
        .collect();
    let app: String = if significant.is_empty() {
        "unnamed_app".to_string()
    } else {
        significant[..significant.len().min(3)].join("_")
    };

    let a = Term::str(app.clone());
    let mut triples: Vec<Fact> = vec![(a.clone(), Term::str("is_a"), Term::str("application"))];
    let push = |fact: Fact, triples: &mut Vec<Fact>| {
        if !triples.contains(&fact) {
            triples.push(fact);
        }
    };
    let needs = |cap: &str| (a.clone(), Term::str("needs"), Term::str(cap));
    let feature = |name: &str| {
        [
            (a.clone(), Term::str("has_feature"), Term::str(name)),
            (Term::str(name), Term::str("is_a"), Term::str("state_machine")),
        ]
    };

    // Capability keywords. Checked in a fixed order — determinism again.
    if matches_any(&tokens, &["grid", "board", "tile", "tiles", "cells", "checkerboard"]) {
        push(needs("grid_layout"), &mut triples);
        push(needs("graphics"), &mut triples);
    }
    if matches_any(&tokens, &["game", "games", "arcade", "draw", "display", "graphics", "render", "screen", "animation"]) {
        push(needs("graphics"), &mut triples);
    }
    if matches_any(&tokens, &["multiplayer", "online", "network", "networked", "server", "websocket", "websockets"]) {
        push(needs("networking"), &mut triples);
    }
    if matches_any(&tokens, &["save", "saves", "database", "persist", "persistent", "storage", "sqlite"]) {
        push(needs("persistence"), &mut triples);
    }
    if matches_any(&tokens, &["score", "scores", "scoring", "points", "leaderboard", "highscore"]) {
        for fact in feature("scoring_system") {
            push(fact, &mut triples);
        }
        // Scores are worth keeping: a scoring system implies persistence.
        push(needs("persistence"), &mut triples);
    }
    if matches_any(&tokens, &["turn", "turns", "phase", "phases", "state", "states", "mode", "modes"]) {
        for fact in feature("control_flow") {
            push(fact, &mut triples);
        }
    }
    if matches_any(&tokens, &["game", "games", "arcade"]) {
        // Every game ticks: give it a game-loop state machine.
        for fact in feature("game_loop") {
            push(fact, &mut triples);
        }
    }

    Harvest {
        goal: (a, Term::str("code"), Term::str("?code")),
        app,
        triples,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Term {
        Term::str(s)
    }

    #[test]
    fn flagship_request_harvests_full_vocabulary() {
        let h = harvest_offline("Make a multiplayer grid game with a scoring system");
        assert_eq!(h.app, "multiplayer_grid_game");
        assert_eq!(h.goal, (t("multiplayer_grid_game"), t("code"), t("?code")));
        let expect = [
            (t("multiplayer_grid_game"), t("is_a"), t("application")),
            (t("multiplayer_grid_game"), t("needs"), t("grid_layout")),
            (t("multiplayer_grid_game"), t("needs"), t("graphics")),
            (t("multiplayer_grid_game"), t("needs"), t("networking")),
            (t("multiplayer_grid_game"), t("needs"), t("persistence")),
            (t("multiplayer_grid_game"), t("has_feature"), t("scoring_system")),
            (t("scoring_system"), t("is_a"), t("state_machine")),
        ];
        for fact in &expect {
            assert!(h.triples.contains(fact), "missing {fact:?}");
        }
    }

    #[test]
    fn harvest_is_deterministic() {
        let a = harvest_offline("Make a multiplayer grid game with a scoring system");
        let b = harvest_offline("Make a multiplayer grid game with a scoring system");
        assert_eq!(a.triples, b.triples);
        assert_eq!(a.goal, b.goal);
    }

    #[test]
    fn parse_response_round_trip() {
        let h = parse_response(
            r#"{"goal": ["chess_club_tracker", "code", "?code"],
                "triples": [["chess_club_tracker", "is_a", "application"],
                            ["chess_club_tracker", "needs", "persistence"]]}"#,
        )
        .unwrap();
        assert_eq!(h.app, "chess_club_tracker");
        assert_eq!(h.triples.len(), 2);
        assert_eq!(h.goal.2, t("?code"));
    }

    #[test]
    fn prompt_embeds_request() {
        let p = prompt_for("build a chess timer");
        assert!(p.contains("build a chess timer"));
        assert!(p.contains("STRICT JSON"));
    }
}
