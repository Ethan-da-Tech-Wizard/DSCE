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
//! ```

use std::process::ExitCode;

use dsce::db_store::SqliteVialStore;
use dsce::demo_kb::{build_engine, demo_vials};
use dsce::engine::Engine;
use dsce::facts::{Pattern, Term};

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
    eprintln!("Run the DSCE demo or query a database:");
    eprintln!("  dsce  [subject predicate object]");
    eprintln!("  dsce --db <db_path> subject predicate object");
    eprintln!("  dsce --db <db_path> --seed        (write the demo KB into the database)");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    let mut db_path: Option<String> = None;
    if args.first().map(String::as_str) == Some("--db") {
        if args.len() < 2 {
            eprintln!("Error: --db flag requires a database path parameter.");
            return ExitCode::from(2);
        }
        db_path = Some(args[1].clone());
        args.drain(..2);
    }

    if args.first().map(String::as_str) == Some("--seed") {
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

    let mut engine = match &db_path {
        Some(path) => match SqliteVialStore::open(path) {
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
        },
        None => build_engine(),
    };

    let goals: Vec<Pattern> = match args.len() {
        3 => vec![(parse_term(&args[0]), parse_term(&args[1]), parse_term(&args[2]))],
        0 => {
            if db_path.is_some() {
                eprintln!("Error: When querying a database, you must specify a triple goal query, e.g.:");
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
