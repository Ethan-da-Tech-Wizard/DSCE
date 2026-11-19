// ==============================================================================
// Program: harvester.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: Semantic Harvester module translating natural language into DSCE goals.
//          Translates high-level programming prompts into formal Datalog query goals
//          and dynamic vocabulary triples using offline keyword parsing or prompt
//          templates, bridging human intent to deterministic code assembly.
// ==============================================================================

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

// triple_from_json is reused from db_store so LLM replies and database rows share one term decoder.
// File role: first stage of the DSCE pipeline. Natural language goes in; a `Harvest` (goal pattern plus
// vocabulary triples) comes out and is handed to the engine (engine.rs `ask_with_facts`).
//
// Two entry points with the same output type: `harvest_offline` (pure keyword tables, used by the CLI and
// the tests) and `parse_response` (decodes JSON written by an LLM that was prompted with `PROMPT_TEMPLATE`).
//
// Design decision: the keyword tables are plain Rust code, not data files, so the compiler checks them and
// the whole harvester stays deterministic and offline. The cost is that adding a keyword means a rebuild.
//
// Limits: matching is whole-token only (no stemming), so "games" and "game" are listed separately below.
// Tested at the bottom of this file; run with `cargo test harvester`.
use serde_json::Value;

use crate::db_store::triple_from_json;
use crate::facts::{Fact, Pattern, Term};

/// The prompt template for LLM-backed harvesting. Replace `{{REQUEST}}`
/// with the user's request (or use [`prompt_for`]) and parse the model's
/// reply with [`parse_response`].
// The vocabulary listed inside the prompt must match the capabilities the knowledge-base vials
// actually provide; the model is told to use only those words, because anything else would be
// a dead-end triple that no vial can satisfy.
// Raw string (r#""#) so the JSON braces and quotes inside the prompt need no escaping. `{{REQUEST}}` is the only
// placeholder; it is replaced textually by `prompt_for`, not by format!, so the literal braces in the JSON example survive.
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
// Returns the full prompt ready to send to a model. Performs no network I/O; the caller owns the API call.
pub fn prompt_for(request: &str) -> String {
    PROMPT_TEMPLATE.replace("{{REQUEST}}", request)
}

/// What a harvest produces: the app slug, the query goal, and the
/// vocabulary triples to pour before reasoning.
#[derive(Debug, Clone)]
/// (Field meaning: `app` is the snake_case slug, which is also the subject of `goal`.)
// `app` is duplicated inside `goal` (its subject) for convenience, so callers can name output files without destructuring the tuple.
pub struct Harvest {
    pub app: String,
    pub goal: Pattern,
    pub triples: Vec<Fact>,
}

/// Parse a strict-JSON harvester reply (LLM output) into a [`Harvest`].
// Errors: a descriptive String for invalid JSON, a missing "goal" key, a missing "triples" array, or any
// malformed triple (the message names which one). Never panics on bad model output.
pub fn parse_response(json: &str) -> Result<Harvest, String> {
    // Everything from an LLM is untrusted text, so each field is validated and a descriptive error is returned instead of panicking.
    let value: Value = serde_json::from_str(json).map_err(|e| format!("bad harvest JSON: {e}"))?;
    let goal = triple_from_json(
        value.get("goal").ok_or_else(|| "harvest JSON missing \"goal\"".to_string())?,
    )
    .map_err(|e| format!("bad goal: {e}"))?;

    // Each triple goes through the same decoder as database rows, so term typing (string/number/variable) is consistent across sources.
    let triples = value
        .get("triples")
        .and_then(Value::as_array)
        .ok_or_else(|| "harvest JSON missing \"triples\" array".to_string())?
        .iter()
        .map(|t| triple_from_json(t).map_err(|e| format!("bad triple: {e}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Harvest {
        // The app name is read back from the goal's subject so the two can never disagree.
        app: goal.0.to_string(),
        goal,
        triples,
    })
}

/// Words that carry no naming or capability signal.
// Filler words removed before choosing the app name. They are still kept as `contains_token` facts further down,
// so rules can use them even though they never appear in the name.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "of", "for", "to", "in", "on", "that", "this", "some", "any",
    "with", "using", "make", "build", "create", "write", "made", "me", "my", "please", "i", "want",
    "app", "application", "program", "software", "simple",
];

// True when at least one request token equals one of `keywords` (whole-token match, not substring).
// Whole-token comparison keeps "go" from matching inside "gopher" or "algorithm", at the cost of missing plurals.
fn matches_any(tokens: &[String], keywords: &[&str]) -> bool {
    tokens.iter().any(|t| keywords.contains(&t.as_str()))
}

/// Explicit programming-language targets, checked in a fixed order.
/// Each detected language becomes one `[app, "target_language", lang_*]`
/// triple; the `lang_` prefix keeps common words ("go", "c") from
/// colliding with unrelated entity names in the knowledge base.
// Table format: (knowledge-base entity, accepted words). The order is the emission order of the triples, which keeps output deterministic.
// "c" and "go" are real English tokens, so a request like "go to c" matches two languages; see the limitation note on `harvest_offline`.
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
/// (Limitation: matching is by whole word only, so very short keywords such as "c" or "go" can
/// also match ordinary English words in the request.)
// Pipeline: tokenise -> derive the app name -> assert baseline facts -> keyword groups in fixed order ->
// language -> tunable parameters. Every step is a pure function of the request text, so the output can be pinned in tests.
pub fn harvest_offline(request: &str) -> Harvest {
    // Lower-cased first so keyword tables only need lower-case entries.
    let tokens: Vec<String> = request
        .to_lowercase()
        // Split on anything that is not a letter/digit, but keep '+' and '#' so "c++" and "c#" survive as tokens.
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect();

    // The app name: the first three significant words, joined snake_case.
    // Stopwords are dropped so "make a multiplayer grid game" becomes multiplayer_grid_game, not make_a_multiplayer.
    let significant: Vec<&str> = tokens
        .iter()
        .map(String::as_str)
        .filter(|t| !STOPWORDS.contains(t))
        .collect();
    let app: String = if significant.is_empty() {
        "unnamed_app".to_string()
    } else {
        // Three words is a compromise: long enough to be recognisable, short enough to be a usable file/identifier name.
        significant[..significant.len().min(3)].join("_")
    };

    // `a` is the application entity that all later triples are about.
    // All facts below have `a` as subject, so rules can join on the application entity.
    let a = Term::str(app.clone());

    // The first fact marks the entity as an application; vials use `is_a application` to decide they apply.
    let mut triples: Vec<Fact> = vec![(a.clone(), Term::str("is_a"), Term::str("application"))];
    // Every raw token is also asserted (contains_token) so rules can react to words the keyword tables do not cover.
    for token in &tokens {
        triples.push((a.clone(), Term::str("contains_token"), Term::str(token.clone())));
    }
    // Add a fact only if it is not already present, keeping the output free of duplicates while preserving first-seen order.
    let push = |fact: Fact, triples: &mut Vec<Fact>| {
        if !triples.contains(&fact) {
            triples.push(fact);
        }
    };
    // Shorthand constructors for the two kinds of triples emitted most often.
    // `needs(x)` builds [app, "needs", x]. A capability is satisfied when some vial in the knowledge base `provides` it.
    let needs = |cap: &str| (a.clone(), Term::str("needs"), Term::str(cap));

    // A feature is two facts at once: the app has it, and it is a state machine (which pulls in the generic state-machine vial).
    let feature = |name: &str| {
        [
            (a.clone(), Term::str("has_feature"), Term::str(name)),
            (Term::str(name), Term::str("is_a"), Term::str("state_machine")),
        ]
    };

    // Capability keywords. Checked in a fixed order — determinism again.
    // Capability keyword groups. Each `if` is independent (no else-if), so one request can need several capabilities.
    // Order is fixed so the emitted triples are always in the same sequence.
    if matches_any(&tokens, &["c++", "cpp", "cplusplus", "qt"]) {
        push(needs("cpp_graphics"), &mut triples);
    }

    // Dictionary/search words select the dictionary-model template (a word list with lookup).
    if matches_any(&tokens, &["dictionary", "dict", "lexicon", "search", "vocabulary"]) {
        push(needs("dictionary_model"), &mut triples);
    }

    // Random-number requests select the random generation template; combined with a language word it picks that language's variant.
    if matches_any(&tokens, &["random", "rand", "generator"]) {
        push(needs("random_generation"), &mut triples);
    }

    // A grid implies graphics (cells have to be drawn), so both capabilities are asserted from one keyword group.
    if matches_any(&tokens, &["grid", "board", "tile", "tiles", "cells", "checkerboard"]) {
        push(needs("grid_layout"), &mut triples);
        push(needs("graphics"), &mut triples);
    }

    // Broad graphics trigger: any drawing-related word. This is intentionally generous because a false positive only adds an optional capability.
    if matches_any(&tokens, &[
        "game", "games", "arcade", "draw", "display", "graphics", "render", "screen", "animation",
        "gui", "tkinter", "pygame", "color", "colors", "flash", "flashing", "paint", "painting",
        "drawing", "rendering", "shape", "shapes", "circle", "rect", "rectangle", "sprite", "sprites",
        "font", "canvas", "view", "viewport", "visual", "visuals"
    ]) {
        push(needs("graphics"), &mut triples);
    }

    // From here to the economy block: the game-mechanics capabilities that select the richer game vials (combat, towers, waves, bosses, effects, economy).
    if matches_any(&tokens, &["twin-stick", "combat", "shoot", "shooting", "projectile", "weapon"]) {
        push(needs("twin_stick_combat"), &mut triples);
    }

    // Tower defence mechanics: placing turrets on a grid.
    if matches_any(&tokens, &["tower", "defense", "turret", "turrets", "defenses"]) {
        push(needs("tower_defense_grid"), &mut triples);
    }

    // Enemy wave spawning with a point budget per wave.
    if matches_any(&tokens, &["wave", "waves", "spawner", "spawning", "budget"]) {
        push(needs("wave_spawning"), &mut triples);
    }

    // Procedural level generation (connected rooms), as opposed to a fixed map.
    if matches_any(&tokens, &["procedural", "rooms", "connect", "mapping", "map"]) {
        push(needs("procedural_generation"), &mut triples);
    }

    // Boss fights modelled as a finite-state machine with telegraphed attacks.
    if matches_any(&tokens, &["boss", "phase", "telegraph", "bosses"]) {
        push(needs("boss_ai_fsm"), &mut triples);
    }

    // Timed status effects (freeze, burn, shock, poison) that tick down each frame.
    if matches_any(&tokens, &["status", "freeze", "burn", "shock", "poison"]) {
        push(needs("status_effect_ticker"), &mut triples);
    }

    // In-game currency and shop economy.
    if matches_any(&tokens, &["economy", "scrap", "currency", "points"]) {
        push(needs("currency_economy"), &mut triples);
    }
    // Complexity tier: requesting advanced mechanics (bosses, status effects, economy) selects the richer game template.
    // `high_tier` selects the larger game template. The keyword list repeats the boss/status/economy words from the groups above on purpose:
    // asking for any of those features is what upgrades the tier.
    let mut tier = "mid_tier";
    if matches_any(&tokens, &["boss", "phase", "telegraph", "bosses", "status", "freeze", "burn", "shock", "poison", "economy", "scrap", "currency", "points"]) {
        tier = "high_tier";
    }
    push((a.clone(), Term::str("game_tier"), Term::str(tier)), &mut triples);
    // The next few blocks choose one variant each (weapon, movement, tower) with a default, so downstream rules always find exactly one value.
    // Variant selectors: weapon, movement and tower each have a default, so every game request gets exactly one value per selector.
    // Rules downstream can therefore rely on a single `weapon_type`/`move_modifier`/`tower_type` fact existing.
    let mut w_type = "shoot_standard";

    // Weapon variants are mutually exclusive: shotgun wins over beam, beam over the standard shot, which is the else-if order.
    if matches_any(&tokens, &["shotgun"]) {
        w_type = "shoot_shotgun";
    } else if matches_any(&tokens, &["beam", "laser", "railgun"]) {
        w_type = "shoot_beam";
    }
    push((a.clone(), Term::str("weapon_type"), Term::str(w_type)), &mut triples);

    let mut m_type = "move_standard";

    // Only one movement modifier exists today (dash); the default keeps the selector fact always present.
    if matches_any(&tokens, &["dash", "teleport", "blink"]) {
        m_type = "move_dash";
    }
    push((a.clone(), Term::str("move_modifier"), Term::str(m_type)), &mut triples);

    let mut t_type = "tower_standard";

    // Tower variant selector, same pattern as weapons above.
    if matches_any(&tokens, &["tesla", "lightning", "electricity"]) {
        t_type = "tower_tesla";
    }
    push((a.clone(), Term::str("tower_type"), Term::str(t_type)), &mut triples);

    // Plain games get the simple `gui_game` template; multiplayer and grid requests skip it because they have their own templates.
    if matches_any(&tokens, &["game", "games", "arcade", "play"]) {
        if !matches_any(&tokens, &["multiplayer", "grid"]) {
            push(needs("gui_game"), &mut triples);
        }
    }

    // UI vocabulary: widget-style requests need the GUI capability set, distinct from game graphics above.
    if matches_any(&tokens, &[
        "gui", "window", "visual", "widget", "widgets", "button", "buttons", "label", "labels",
        "textbox", "textboxes", "form", "forms", "menu", "menus", "dialog", "dialogs", "frame",
        "frames", "slider", "sliders", "layout", "layouts", "ui", "interface", "ux", "dashboard"
    ]) {
        push(needs("gui_graphics"), &mut triples);
    }

    // A stricter subset: only requests that explicitly ask for a full interface get `complete_gui` (the multi-widget template).
    if matches_any(&tokens, &[
        "ui", "ux", "interface", "dashboard", "widgets", "controls", "panels"
    ]) {
        push(needs("complete_gui"), &mut triples);
    }

    // GUI/game framework keywords -> framework name. Several may match, each emits its own needs_gui_framework triple.
    // Table of (framework name, accepted words). `[..]` converts each array to a slice so entries of different lengths share one type.
    let frameworks = [
        ("pyside6", &["pyside", "pyside6", "qt6"][..]),
        ("pyqt6", &["pyqt", "pyqt6"][..]),
        ("wxpython", &["wxpython", "wxwidgets"][..]),
        ("customtkinter", &["customtkinter", "ctk"][..]),
        ("dearpygui", &["dearpygui", "dpg"][..]),
        ("flet", &["flet", "flutter"][..]),
        ("nicegui", &["nicegui"][..]),
        ("tkinter", &["tkinter"][..]),
        ("ttkbootstrap", &["ttkbootstrap"][..]),
        ("pysimplegui", &["pysimplegui"][..]),
        ("kivy", &["kivy"][..]),
        ("kivymd", &["kivymd"][..]),
        ("toga", &["toga", "beeware"][..]),
        ("textual", &["textual", "tui"][..]),
        ("rich", &["rich"][..]),
        ("prompt_toolkit", &["prompt_toolkit", "prompt-toolkit"][..]),
        ("pygame_ce", &["pygame", "pygame-ce"][..]),
        ("arcade", &["arcade"][..]),
        ("pyglet", &["pyglet"][..]),
        ("panda3d", &["panda3d"][..]),
        ("gtk4", &["gtk4", "pygobject", "gtk"][..]),
        ("qml", &["qml", "qtquick"][..]),
        ("pywebview", &["pywebview", "webview"][..]),
        ("eel", &["eel"][..]),
        ("enaml", &["enaml"][..]),
    ];

    // Not exclusive: a request that names two frameworks gets two `needs_gui_framework` triples, and the vials decide what to do.
    for (name, keywords) in &frameworks {
        if matches_any(&tokens, keywords) {
            push((a.clone(), Term::str("needs_gui_framework"), Term::str(*name)), &mut triples);
        }
    }
    // From here: keyword groups for classic data-structure / design-pattern / library templates, each mapping to one capability.
    // Educational-library keywords (data structures, SICP-style interpreters, kernel patterns). Each maps to exactly one template capability.
    if matches_any(&tokens, &["academic", "ods", "sicp", "linux", "kernel", "pattern", "patterns"]) {
        push(needs("academic_suite"), &mut triples);
    }

    // Randomised binary search tree (treap) template.
    if matches_any(&tokens, &["treap", "randomized"]) {
        push(needs("randomized_bst"), &mut triples);
    }

    // Strictly height-balanced tree template (AVL rotations).
    if matches_any(&tokens, &["avl", "rotation"]) {
        push(needs("strict_height_balance"), &mut triples);
    }

    // SICP-style evaluator: an eval/apply loop.
    if matches_any(&tokens, &["evaluator", "eval", "apply"]) {
        push(needs("eval_apply_loop"), &mut triples);
    }

    // Delayed (lazy) evaluation and streams.
    if matches_any(&tokens, &["lazy", "stream"]) {
        push(needs("delayed_evaluation"), &mut triples);
    }

    // Kernel-style intrusive linked list.
    if matches_any(&tokens, &["intrusive", "list"]) {
        push(needs("intrusive_linked_list"), &mut triples);
    }

    // Lock-free ring buffer (kfifo style).
    if matches_any(&tokens, &["ring", "kfifo", "circular"]) {
        push(needs("lockless_ring_buffer"), &mut triples);
    }

    // Strategy pattern: separate an algorithm from the code that uses it.
    if matches_any(&tokens, &["strategy", "decouple"]) {
        push(needs("decouple_algorithm_from_client"), &mut triples);
    }

    // Observer / publish-subscribe pattern.
    if matches_any(&tokens, &["observer", "pubsub", "pub_sub"]) {
        push(needs("decoupled_pub_sub"), &mut triples);
    }

    // The hybrid template: a bouncing-ball paddle game and a calculator in one window (polyglot_hybrid_app.json).
    if matches_any(&tokens, &["gameboy"]) {
        push(needs("gameboy_hybrid"), &mut triples);
    }

    // Several libraries combined in one pipeline.
    if matches_any(&tokens, &["multilib", "pipeline"]) {
        push(needs("multilib_suite"), &mut triples);
    }

    // Data/ML/web library keywords: each group names the capability a library vial provides, not the library itself.
    if matches_any(&tokens, &["pandas", "dataframe"]) {
        push(needs("data_manipulation"), &mut triples);
    }
    if matches_any(&tokens, &["numpy", "matrix"]) {
        push(needs("numerical_arrays"), &mut triples);
    }
    if matches_any(&tokens, &["sklearn", "regression", "ml"]) {
        push(needs("machine_learning"), &mut triples);
    }

    // "http" intentionally also appears in the web_server group below: HTTP clients and servers are both plausible readings.
    if matches_any(&tokens, &["requests", "http"]) {
        push(needs("http_requests"), &mut triples);
    }
    if matches_any(&tokens, &["web", "server", "api", "http"]) {
        push(needs("web_server"), &mut triples);
    }

    // "sql" also selects the SQL language in LANGUAGE_KEYWORDS; this group adds a database-connector capability on top.
    if matches_any(&tokens, &["database", "db", "sql"]) {
        push(needs("db_connector"), &mut triples);
    }

    // Calculator template.
    if matches_any(&tokens, &["calculator", "calc", "math", "arithmetic", "addition"]) {
        push(needs("math_calculator"), &mut triples);
    }

    // "server" appears in two groups (web_server and networking) so a "server" request gets both; the vials sort out which applies.
    if matches_any(&tokens, &["multiplayer", "online", "network", "networked", "server", "websocket", "websockets"]) {
        push(needs("networking"), &mut triples);
    }
    if matches_any(&tokens, &["save", "saves", "database", "persist", "persistent", "storage", "sqlite"]) {
        push(needs("persistence"), &mut triples);
    }

    // A scoring system is a feature (state machine) plus persistence, because scores are only useful if they survive a restart.
    if matches_any(&tokens, &["score", "scores", "scoring", "points", "leaderboard", "highscore"]) {
        for fact in feature("scoring_system") {
            push(fact, &mut triples);
        }
        // Scores are worth keeping: a scoring system implies persistence.
        push(needs("persistence"), &mut triples);
    }

    // Turn/phase/mode vocabulary means the program is a state machine, modelled as a generic `control_flow` feature.
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
    // Framework/API groups. The harvester maps words to *abstract* capabilities so new frameworks only need a vial, not a harvester change.
    if matches_any(&tokens, &["pytorch", "torch", "tensor", "tensors"]) {
        push(needs("tensor_computation"), &mut triples);
    }

    // GPU acceleration is separate from tensor computation so a request can ask for CUDA without PyTorch.
    if matches_any(&tokens, &["cuda", "gpu"]) {
        push(needs("gpu_acceleration"), &mut triples);
    }

    // Vector similarity search (FAISS-style).
    if matches_any(&tokens, &["faiss", "embedding", "embeddings"]) {
        push(needs("vector_search"), &mut triples);
    }

    // Browser UI framework.
    if matches_any(&tokens, &["react", "jsx"]) {
        push(needs("web_ui"), &mut triples);
    }

    // Desktop app wrapping a web UI.
    if matches_any(&tokens, &["electron"]) {
        push(needs("desktop_web"), &mut triples);
    }

    // Python HTTP API framework template.
    if matches_any(&tokens, &["fastapi"]) {
        push(needs("fastapi_api"), &mut triples);
    }

    // Node.js routing template.
    if matches_any(&tokens, &["express"]) {
        push(needs("express_routing"), &mut triples);
    }

    // Deployment capabilities: a container image first, then orchestration of several containers.
    if matches_any(&tokens, &["docker", "dockerfile", "container", "containers", "containerize", "containerized"]) {
        push(needs("containerization"), &mut triples);
    }

    // Orchestration is separate so "docker only" requests do not pull in Kubernetes manifests.
    if matches_any(&tokens, &["kubernetes", "k8s", "orchestrate", "orchestration"]) {
        push(needs("orchestration"), &mut triples);
    }

    // Target language: every explicitly named language is asserted, in the
    // fixed LANGUAGE_KEYWORDS order. When the request names none, Python is
    // the default target — the assembly patterns key on this triple, so an
    // unqualified request keeps producing the classic Python program.
    // Multi-language requests get a JavaScript frontend plus a backend language (Go when none is named); otherwise a single target_language is used.
    // Special case: "multi", "multilang" or "hybrid" means a JavaScript frontend plus one backend, instead of a single target language.
    let is_multilang = matches_any(&tokens, &["multi", "multilang", "hybrid"]);
    if is_multilang {
        // A multi-language request also needs the generic multilang template, besides the frontend/backend language facts below.
        push(needs("multilang_tool"), &mut triples);

        // Frontend is always JavaScript in this mode, the usual browser language.
        push((a.clone(), Term::str("frontend_language"), Term::str("lang_javascript")), &mut triples);
        let mut backend_found = false;

        // JavaScript is skipped as a backend because it is already asserted as the frontend.
        for (lang, keywords) in LANGUAGE_KEYWORDS {
            if *lang != "lang_javascript" && matches_any(&tokens, keywords) {
                push((a.clone(), Term::str("backend_language"), Term::str(*lang)), &mut triples);
                backend_found = true;
            }
        }

        // Go is the default backend: it compiles to one binary, which keeps the generated project simple.
        if !backend_found {
            push((a.clone(), Term::str("backend_language"), Term::str("lang_go")), &mut triples);
        }
    } else {
        // Default-language rule: assert lang_python only when no language was named, so an explicit choice replaces rather than adds to it.
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
    }

    // Tunable parameters parsed from phrases like "speed 7" or "limit 50", with defaults. They only take effect when the numeric word follows the keyword.
    // Tunable parameters for the demo templates (ball colour, speed, calculator limit). Defaults apply when the request does not set them.
    let mut color = "cyan".to_string();
    let mut speed = "3".to_string();

    // `limit` is the largest result the hybrid calculator accepts; bigger results show "OVER LIMIT!" (polyglot_hybrid_app.json).
    let mut limit = "10".to_string();

    // Index loop (not an iterator) because each keyword looks one token ahead.
    for i in 0..tokens.len() {
        if ["red", "blue", "green", "cyan", "yellow", "orange"].contains(&tokens[i].as_str()) {
            color = tokens[i].clone();
        }

        // "speed 7": the token after the keyword is used only if it is all digits, so "speed up" does not set a nonsense value.
        if tokens[i] == "speed" && i + 1 < tokens.len() {
            if tokens[i+1].chars().all(|c| c.is_numeric()) {
                speed = tokens[i+1].clone();
            }
        }
        if tokens[i] == "limit" && i + 1 < tokens.len() {
            if tokens[i+1].chars().all(|c| c.is_numeric()) {
                limit = tokens[i+1].clone();
            }
        }
    }

    // Always emitted (with the default if needed), so templates that substitute these values never find them missing.
    push((a.clone(), Term::str("ball_color"), Term::str(color)), &mut triples);
    push((a.clone(), Term::str("ball_speed"), Term::str(speed)), &mut triples);
    push((a.clone(), Term::str("calc_limit"), Term::str(limit)), &mut triples);

    Harvest {
        // The goal asks "what is the `code` of this app?"; `?code` is a variable the engine solves by assembling vials.
        goal: (a, Term::str("code"), Term::str("?code")),
        app,
        triples,
    }
}

#[cfg(test)]
// Tests pin the offline harvester's exact output (determinism) and the JSON/prompt helpers.
// Test scope: exact harvest content for a flagship request, determinism, language defaults, framework keywords and the JSON/prompt helpers.
mod tests {
    use super::*;

    // Shorthand for building a string term.
    fn t(s: &str) -> Term {
        Term::str(s)
    }

    #[test]
    // Pins the sample request from the module docs: name derivation plus the grid/network/persistence/scoring triples. Uses `contains`, so extra triples do not break it.
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
    // The same request must always produce the same triples in the same order.
    fn harvest_is_deterministic() {
        let a = harvest_offline("Make a multiplayer grid game with a scoring system");
        let b = harvest_offline("Make a multiplayer grid game with a scoring system");
        assert_eq!(a.triples, b.triples);
        assert_eq!(a.goal, b.goal);
    }

    #[test]
    // One request per supported language, each of which must produce its own `target_language` triple. This guards the keyword table against typos.
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
    // No language named -> Python. Many assembly rules key on this fact, so losing it would produce empty programs.
    fn python_is_the_default_target_language() {
        let h = harvest_offline("Make a random number generator");
        assert!(h
            .triples
            .contains(&(t(&h.app), t("target_language"), t("lang_python"))));
    }

    #[test]
    // The counterpart of the default test: naming Rust must NOT also add Python.
    fn explicit_language_replaces_the_default() {
        let h = harvest_offline("Make a random number generator in Rust");
        assert!(!h
            .triples
            .contains(&(t(&h.app), t("target_language"), t("lang_python"))));
    }

    #[test]
    // Table-driven: one phrase per framework-class capability, which would catch a renamed or dropped keyword group.
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
    // Simulates a well-formed LLM reply (raw string so the quotes need no escaping) and checks the app name, triple count and goal variable.
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
    // Guards the template substitution and the "STRICT JSON" instruction, which the parser depends on.
    fn prompt_embeds_request() {
        let p = prompt_for("build a chess timer");
        assert!(p.contains("build a chess timer"));
        assert!(p.contains("STRICT JSON"));
    }
}
