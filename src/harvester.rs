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
  capabilities (via "needs"):    graphics, grid_layout, networking, persistence,
                                 random_generation, dictionary_model, cpp_graphics,
                                 tensor_computation, gpu_acceleration, vector_search,
                                 web_ui, desktop_web, containerization, orchestration
  languages (via "target_language"): lang_python, lang_rust, lang_c, lang_cpp,
                                 lang_csharp, lang_go, lang_sql, lang_assembly,
                                 lang_javascript
  feature classes (via "is_a"):  state_machine, application
  predicates:                    needs, has_feature, is_a, target_language

Rules:
- Invent one snake_case application name derived from the request.
- Always assert [app, "is_a", "application"].
- Map every requested capability to a [app, "needs", capability] triple.
- Map every explicitly requested programming language to a
  [app, "target_language", lang_*] triple; assert lang_python when the
  request names no language.
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

/// Explicit programming-language targets, checked in a fixed order.
/// Each detected language becomes one `[app, "target_language", lang_*]`
/// triple; the `lang_` prefix keeps common words ("go", "c") from
/// colliding with unrelated entity names in the knowledge base.
const LANGUAGE_KEYWORDS: &[(&str, &[&str])] = &[
    ("lang_assembly", &["assembly", "asm", "nasm", "x86"]),
    ("lang_c", &["c"]),
    ("lang_cpp", &["c++", "cpp", "cplusplus"]),
    ("lang_csharp", &["c#", "csharp", "dotnet"]),
    ("lang_go", &["go", "golang"]),
    ("lang_javascript", &["javascript", "js", "node", "nodejs"]),
    ("lang_python", &["python", "python3", "py"]),
    ("lang_rust", &["rust"]),
    ("lang_sql", &["sql"]),
];

/// Deterministic, dependency-free harvesting: keyword-match the request
/// against the built-in vocabulary. Same request, same harvest, every time.
pub fn harvest_offline(request: &str) -> Harvest {
    let tokens: Vec<String> = request
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
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
    if matches_any(&tokens, &["c++", "cpp", "cplusplus", "qt"]) {
        push(needs("cpp_graphics"), &mut triples);
    }
    if matches_any(&tokens, &["dictionary", "dict", "lexicon", "search", "vocabulary"]) {
        push(needs("dictionary_model"), &mut triples);
    }
    if matches_any(&tokens, &["random", "rand", "generator"]) {
        push(needs("random_generation"), &mut triples);
    }
    if matches_any(&tokens, &["grid", "board", "tile", "tiles", "cells", "checkerboard"]) {
        push(needs("grid_layout"), &mut triples);
        push(needs("graphics"), &mut triples);
    }
    if matches_any(&tokens, &["game", "games", "arcade", "draw", "display", "graphics", "render", "screen", "animation", "gui", "tkinter", "pygame"]) {
        push(needs("graphics"), &mut triples);
    }
    if matches_any(&tokens, &["gui", "window", "visual"]) {
        push(needs("gui_graphics"), &mut triples);
    }
    if matches_any(&tokens, &["web", "server", "api", "http"]) {
        push(needs("web_server"), &mut triples);
    }
    if matches_any(&tokens, &["database", "db", "sql"]) {
        push(needs("db_connector"), &mut triples);
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

    // Framework and API capabilities: each keyword group maps to one
    // abstract capability that a framework vial "provides".
    if matches_any(&tokens, &["pytorch", "torch", "tensor", "tensors"]) {
        push(needs("tensor_computation"), &mut triples);
    }
    if matches_any(&tokens, &["cuda", "gpu"]) {
        push(needs("gpu_acceleration"), &mut triples);
    }
    if matches_any(&tokens, &["faiss", "embedding", "embeddings"]) {
        push(needs("vector_search"), &mut triples);
    }
    if matches_any(&tokens, &["react", "jsx"]) {
        push(needs("web_ui"), &mut triples);
    }
    if matches_any(&tokens, &["electron"]) {
        push(needs("desktop_web"), &mut triples);
    }
    if matches_any(&tokens, &["docker", "dockerfile", "container", "containers", "containerize", "containerized"]) {
        push(needs("containerization"), &mut triples);
    }
    if matches_any(&tokens, &["kubernetes", "k8s", "orchestrate", "orchestration"]) {
        push(needs("orchestration"), &mut triples);
    }

    // Target language: every explicitly named language is asserted, in the
    // fixed LANGUAGE_KEYWORDS order. When the request names none, Python is
    // the default target — the assembly patterns key on this triple, so an
    // unqualified request keeps producing the classic Python program.
    let mut language_found = false;
    for (lang, keywords) in LANGUAGE_KEYWORDS {
        if matches_any(&tokens, keywords) {
            push((a.clone(), Term::str("target_language"), Term::str(*lang)), &mut triples);
            language_found = true;
        }
    }
    if !language_found {
        push(
            (a.clone(), Term::str("target_language"), Term::str("lang_python")),
            &mut triples,
        );
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
    fn explicit_languages_are_harvested() {
        let cases = [
            ("Make a random number generator in Rust", "lang_rust"),
            ("Make a random number generator in C", "lang_c"),
            ("Make a random number generator in C++", "lang_cpp"),
            ("Make a random number generator in C#", "lang_csharp"),
            ("Make a random number generator in Go", "lang_go"),
            ("Write a random number generator in SQL", "lang_sql"),
            ("Make a random number generator in assembly", "lang_assembly"),
            ("Make a random number generator in JavaScript", "lang_javascript"),
            ("Make a random number generator in Python", "lang_python"),
        ];
        for (request, lang) in cases {
            let h = harvest_offline(request);
            let fact = (t(&h.app), t("target_language"), t(lang));
            assert!(h.triples.contains(&fact), "{request:?} did not harvest {lang}");
        }
    }

    #[test]
    fn python_is_the_default_target_language() {
        let h = harvest_offline("Make a random number generator");
        assert!(h
            .triples
            .contains(&(t(&h.app), t("target_language"), t("lang_python"))));
    }

    #[test]
    fn explicit_language_replaces_the_default() {
        let h = harvest_offline("Make a random number generator in Rust");
        assert!(!h
            .triples
            .contains(&(t(&h.app), t("target_language"), t("lang_python"))));
    }

    #[test]
    fn framework_keywords_map_to_capabilities() {
        let cases = [
            ("Train a PyTorch model", "tensor_computation"),
            ("Write a CUDA kernel", "gpu_acceleration"),
            ("Build a FAISS index", "vector_search"),
            ("Make a React page", "web_ui"),
            ("Make an Electron app", "desktop_web"),
            ("Containerize this with Docker", "containerization"),
            ("Deploy it on Kubernetes", "orchestration"),
        ];
        for (request, capability) in cases {
            let h = harvest_offline(request);
            let fact = (t(&h.app), t("needs"), t(capability));
            assert!(h.triples.contains(&fact), "{request:?} did not harvest {capability}");
        }
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
