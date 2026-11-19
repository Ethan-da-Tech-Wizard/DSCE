// ==============================================================================
// Program: main.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: Command-line interface driver for the DSCE Rust engine.
//          Parses CLI parameters, manages database seeding and queries,
//          evaluates triple query goals, loads JSON vial synthesis directories,
//          and drives Semantic Harvester software code generation.
// ==============================================================================

//! DSCE command-line interface.
//!
//! ```text
//! # Run the in-memory demo showcase
//! cargo run
//!
//! # Ask one triple against the in-memory demo KB (use ?vars for unknowns)
//! cargo run -- socrates is_mortal ?answer
//!
//! # Query a SQLite database, loading only the vials the flood reaches
//! cargo run -- --db dsce.sqlite "modesto" "located_in" "?where"
//!
//! # Seed a SQLite database with the demo knowledge base
//! cargo run -- --db dsce.sqlite --seed
//!
//! # Load a JSON vial directory and ask a raw triple against it
//! cargo run -- --vials vials_synthesis my_app needs "?what"
//!
//! # Assemble software from a natural-language request (Semantic Harvester)
//! cargo run -- --synthesize "Make a multiplayer grid game with a scoring system"
//! ```

// Exit codes used throughout: 0 success, 1 runtime failure (bad database / vial directory), 2 usage error.
// File role: the `dsce` command-line binary (Cargo.toml `[[bin]] dsce`). It is a thin driver: it parses
// arguments, picks an engine source (SQLite store, JSON vial directory or built-in demo), and prints results.
// All reasoning lives in the library crate (`dsce::engine`); the GUI equivalent is gui_main.rs.
//
// Modes, in the order `main` checks them: `--seed` (write demo vials to a database), `--synthesize` (natural
// language to program), then a plain triple query. Output goes to stdout; diagnostics go to stderr.
//
// Testing: the library has the unit and integration tests (`cargo test`); this binary is exercised by hand
// with the commands shown in the module docs above.
use std::process::ExitCode;

// The binary uses only the public library API (`dsce::...`), the same surface an embedding program would use.
use dsce::db_store::SqliteVialStore;
use dsce::demo_kb::{build_engine, demo_vials};
use dsce::engine::Engine;
use dsce::facts::{Pattern, Term};
use dsce::harvester::harvest_offline;
use dsce::json_vials::engine_from_dir;
use dsce::proof::fact_str;

/// Interpret one command-line token as a triple term.
///
/// Order matters: "true"/"false" become booleans, then int is tried, then
/// float, and anything left stays a string (including ?variables — the
/// engine recognizes those by their leading '?', not here).
// Same bool -> int -> float -> string order as db_store::term_from_db, so a term typed on the command line
// compares equal to the same term read from a database. Example: `42` becomes Int(42), `?who` stays a string.
fn parse_term(token: &str) -> Term {
    match token.to_ascii_lowercase().as_str() {
        "true" => return Term::Bool(true),
        "false" => return Term::Bool(false),
        _ => {}
    }
    if let Ok(i) = token.parse::<i64>() {
        return Term::Int(i);
    }
    if let Ok(f) = token.parse::<f64>() {
        return Term::float(f);
    }
    Term::str(token)
}

/// Print the accepted command lines to stderr and return exit code 2 (usage error).
// Exit code 2 is the conventional "bad command line" status, distinct from 1 (a runtime failure such as a missing file).
fn usage() -> ExitCode {
    eprintln!("Run the DSCE demo, query a knowledge base, or assemble software:");
    eprintln!("  dsce  [subject predicate object]");
    eprintln!("  dsce --db <db_path> subject predicate object");
    eprintln!("  dsce --db <db_path> --seed        (write the demo KB into the database)");
    eprintln!("  dsce --vials <dir> subject predicate object");
    eprintln!("  dsce --synthesize \"<request>\" [--vials <dir>]");
    ExitCode::from(2)
}

/// The Semantic Harvester flow: request -> goal + vocabulary triples ->
/// flood -> assembled program(s).
// Called by: main when `--synthesize` is given. The vial directory must contain the pattern and library vials the harvested vocabulary refers to.
fn synthesize(request: &str, vials_dir: &str) -> ExitCode {
    // Loading first means a bad vial directory is reported before any harvesting output is printed.
    let mut engine = match engine_from_dir(vials_dir) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Error loading vials from {vials_dir:?}: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Print what was harvested before reasoning so the user can see how their words were interpreted.
    let harvest = harvest_offline(request);
    println!("request: {request:?}");
    println!("harvested goal: {}", fact_str(&harvest.goal));
    println!("harvested vocabulary:");
    for triple in &harvest.triples {
        println!("  {}", fact_str(triple));
    }
    println!();

    // ask_with_facts pours the harvested vocabulary into working memory for this query only, so one request never affects the next.
    let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
    println!("{}", result.summary());

    // Dependency reasoning: everything the bound frameworks depend on
    // (transitively) plus their documented install channels. Derived by
    // the dependency_resolution pattern vial; BTreeMap iteration keeps
    // the listing deterministic.
    let app_term = Term::str(harvest.app.clone());
    // Closure: all derived facts of the form (app, predicate, X), returning the X values. `derivations` is keyed by fact in a BTreeMap, so order is stable.
    let objects_of = |predicate: &str| -> Vec<String> {
        result
            .derivations
            .keys()
            .filter(|f| f.0 == app_term && f.1 == Term::str(predicate))
            .map(|f| f.2.to_string())
            .collect()
    };

    // These two listings are derived facts produced by the dependency_resolution pattern vial, not computed here.
    let dependencies = objects_of("requires_dependency");
    if !dependencies.is_empty() {
        println!("\nresolved dependencies: {}", dependencies.join(", "));
    }
    let install_steps = objects_of("install_step");
    if !install_steps.is_empty() {
        println!("install steps:");
        for step in install_steps {
            println!("  $ {step}");
        }
    }

    // Every answer whose `?code` binding is text is a complete program; several answers appear when more than one template matched.
    for (i, answer) in result.answers.iter().enumerate() {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            println!("\n--- assembled program #{} (confidence {:.3}) ---", i + 1, answer.confidence);
            println!("{code}");
        }
    }
    // No answer means the goal could not be fully proven (not an error), so this is reported on stderr but still exits 0.
    if result.answers.is_empty() {
        eprintln!("\nThe request did not harvest enough capabilities to assemble a full program.");
        eprintln!("Partial derivations are listed in the flood summary above.");
    }
    ExitCode::SUCCESS
}

// Hand-rolled argument parsing (no CLI crate): flags with values, --seed, and up to three positional goal terms.
// Control flow: parse flags -> (seed | synthesize | build engine -> answer goals). Each branch returns its own exit code.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Collected before acting so flags can appear in any order (for example `--seed --db x` or `--db x --seed`).
    let mut db_path: Option<String> = None;
    let mut vials_dir: Option<String> = None;
    let mut synth_request: Option<String> = None;
    let mut seed = false;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        // Fetch the argument after a flag, printing an error when it is missing.
        // Reports the missing value itself, so each flag arm only has to turn `None` into exit code 2.
        let take_value = |i: usize, flag: &str| -> Option<String> {
            args.get(i + 1)
                .cloned()
                .or_else(|| {
                    eprintln!("Error: {flag} requires a value.");
                    None
                })
        };
        match args[i].as_str() {
            "--db" => match take_value(i, "--db") {
                Some(v) => {
                    db_path = Some(v);
                    i += 2;
                }
                None => return ExitCode::from(2),
            },
            "--vials" => match take_value(i, "--vials") {
                Some(v) => {
                    vials_dir = Some(v);
                    i += 2;
                }
                None => return ExitCode::from(2),
            },
            "--synthesize" => match take_value(i, "--synthesize") {
                Some(v) => {
                    synth_request = Some(v);
                    i += 2;
                }
                None => return ExitCode::from(2),
            },
            "--seed" => {
                seed = true;
                i += 1;
            }

            // Anything else that looks like a flag is rejected so typos do not become goal terms.
            other if other.starts_with("--") => {
                eprintln!("Error: unknown flag {other}");
                return usage();
            }

            // Everything remaining is a positional goal term, in order: subject, predicate, object.
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }

    // Seed mode: write the in-memory demo vials into the database and exit without querying.
    if seed {
        let Some(path) = &db_path else {
            eprintln!("Error: --seed requires --db <db_path>.");
            return ExitCode::from(2);
        };
        let store = match SqliteVialStore::open(path) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Error opening database {path:?}: {e}");
                return ExitCode::FAILURE;
            }
        };

        // Seeding is a full overwrite per vial (`save_vial` replaces by id), so re-running it is safe and idempotent.
        for vial in demo_vials() {
            if let Err(e) = store.save_vial(&vial) {
                eprintln!("Error saving vial {:?}: {e}", vial.id);
                return ExitCode::FAILURE;
            }
        }
        println!("Seeded demo knowledge base into {path}");
        return ExitCode::SUCCESS;
    }

    // Natural-language mode; the vial directory defaults to ./vials_synthesis.
    if let Some(request) = synth_request {
        let dir = vials_dir.as_deref().unwrap_or("vials_synthesis");
        return synthesize(&request, dir);
    }

    // Engine source, in priority order: SQLite database (vials loaded lazily), JSON vial directory, then the built-in demo knowledge base.
    let mut engine = if let Some(path) = &db_path {
        match SqliteVialStore::open(path) {
            // `with_store` makes the engine load vials lazily from SQLite: only vials whose terms receive sand are read.
            Ok(store) => {
                let mut engine = Engine::with_store(store);
                // Schema metadata for functional predicates: each subject
                // has at most one value for these.
                // `true` marks the predicate functional, so conflicting values for one subject are flagged as contradictions.
                engine.register_predicate("located_in", true);
                engine.register_predicate("height", true);
                engine
            }
            Err(e) => {
                eprintln!("Error opening database {path:?}: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(dir) = &vials_dir {
        match engine_from_dir(dir) {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("Error loading vials from {dir:?}: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        // No source flag: use the in-memory demo knowledge base (the Socrates / courtyard examples).
        build_engine()
    };

    // Exactly three positional terms form one goal; none runs the demo queries (only allowed for the built-in KB); any other count is a usage error.
    let goals: Vec<Pattern> = match positional.len() {
        3 => vec![(
            parse_term(&positional[0]),
            parse_term(&positional[1]),
            parse_term(&positional[2]),
        )],
        0 => {
            if db_path.is_some() || vials_dir.is_some() {
                eprintln!("Error: When querying a database or vial directory, you must specify a triple goal, e.g.:");
                eprintln!("  dsce --db dsce.sqlite modesto located_in ?where");
                return ExitCode::from(2);
            }
            vec![
                (Term::str("socrates"), Term::str("is_mortal"), Term::str("?answer")),
                (Term::str("courtyard"), Term::str("area"), Term::str("?a")),
                (Term::str("plaza"), Term::str("area"), Term::str("?a")),
                (Term::str("?who"), Term::str("is_a"), Term::str("mammal")),
            ]
        }
        _ => return usage(),
    };

    // `ask` runs one flood per goal; the engine is reused, so vials loaded for an earlier goal stay cached.
    for (i, goal) in goals.iter().enumerate() {
        if i > 0 {
            // Visual separator between goals.
            println!("\n{}\n", "=".repeat(72));
        }
        println!("{}", engine.ask(goal).summary());
    }
    ExitCode::SUCCESS
}
