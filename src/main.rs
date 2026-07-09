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

use std::process::ExitCode;

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
fn synthesize(request: &str, vials_dir: &str) -> ExitCode {
    let mut engine = match engine_from_dir(vials_dir) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Error loading vials from {vials_dir:?}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let harvest = harvest_offline(request);
    println!("request: {request:?}");
    println!("harvested goal: {}", fact_str(&harvest.goal));
    println!("harvested vocabulary:");
    for triple in &harvest.triples {
        println!("  {}", fact_str(triple));
    }
    println!();

    let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
    println!("{}", result.summary());

    // Dependency reasoning: everything the bound frameworks depend on
    // (transitively) plus their documented install channels. Derived by
    // the dependency_resolution pattern vial; BTreeMap iteration keeps
    // the listing deterministic.
    let app_term = Term::str(harvest.app.clone());
    let objects_of = |predicate: &str| -> Vec<String> {
        result
            .derivations
            .keys()
            .filter(|f| f.0 == app_term && f.1 == Term::str(predicate))
            .map(|f| f.2.to_string())
            .collect()
    };
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

    for (i, answer) in result.answers.iter().enumerate() {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            println!("\n--- assembled program #{} (confidence {:.3}) ---", i + 1, answer.confidence);
            println!("{code}");
        }
    }
    if result.answers.is_empty() {
        eprintln!("\nThe request did not harvest enough capabilities to assemble a full program.");
        eprintln!("Partial derivations are listed in the flood summary above.");
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut db_path: Option<String> = None;
    let mut vials_dir: Option<String> = None;
    let mut synth_request: Option<String> = None;
    let mut seed = false;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
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
            other if other.starts_with("--") => {
                eprintln!("Error: unknown flag {other}");
                return usage();
            }
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }

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
        for vial in demo_vials() {
            if let Err(e) = store.save_vial(&vial) {
                eprintln!("Error saving vial {:?}: {e}", vial.id);
                return ExitCode::FAILURE;
            }
        }
        println!("Seeded demo knowledge base into {path}");
        return ExitCode::SUCCESS;
    }

    if let Some(request) = synth_request {
        let dir = vials_dir.as_deref().unwrap_or("vials_synthesis");
        return synthesize(&request, dir);
    }

    let mut engine = if let Some(path) = &db_path {
        match SqliteVialStore::open(path) {
            Ok(store) => {
                let mut engine = Engine::with_store(store);
                // Schema metadata for functional predicates: each subject
                // has at most one value for these.
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
        build_engine()
    };

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

    for (i, goal) in goals.iter().enumerate() {
        if i > 0 {
            println!("\n{}\n", "=".repeat(72));
        }
        println!("{}", engine.ask(goal).summary());
    }
    ExitCode::SUCCESS
}
