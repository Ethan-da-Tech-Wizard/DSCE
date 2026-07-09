//! Integration tests for the generic software assembler: JSON vials,
//! the Semantic Harvester, and end-to-end program synthesis.

use dsce::engine::QueryResult;
use dsce::facts::Term;
use dsce::harvester::harvest_offline;
use dsce::json_vials::{engine_from_dir, load_vials_dir};

fn t(s: &str) -> Term {
    Term::str(s)
}

fn vials_dir() -> String {
    format!("{}/vials_synthesis", env!("CARGO_MANIFEST_DIR"))
}

fn synthesize(request: &str) -> QueryResult {
    let mut engine = engine_from_dir(vials_dir()).expect("synthesis KB loads");
    let harvest = harvest_offline(request);
    engine.ask_with_facts(&harvest.goal, &harvest.triples)
}

#[test]
fn synthesis_kb_loads_completely() {
    let vials = load_vials_dir(vials_dir()).unwrap();
    let mut ids: Vec<&str> = vials.iter().map(|v| v.id.as_str()).collect();
    ids.sort();
    assert_eq!(
        ids,
        vec![
            "cli_app",
            "cpp_gui_app",
            "finite_state_machine",
            "grid_layout",
            "gui_random_app",
            "mvc",
            "pygame_graphics",
            "qt_cpp_graphics",
            "random_generator",
            "sqlite_databases",
            "tkinter_graphics",
            "websockets_networking",
        ]
    );
}

#[test]
fn library_vials_are_decoupled() {
    // The graphics, database, and networking vials must not mention each
    // other anywhere — they are bound purely by Datalog rules in the
    // pattern vials.
    let vials = load_vials_dir(vials_dir()).unwrap();
    let library_terms = ["pygame", "sqlite3", "websockets"];
    for vial in vials.iter().filter(|v| {
        ["pygame_graphics", "sqlite_databases", "websockets_networking"].contains(&v.id.as_str())
    }) {
        let own = match vial.id.as_str() {
            "pygame_graphics" => "pygame",
            "sqlite_databases" => "sqlite3",
            _ => "websockets",
        };
        assert!(vial.rules.is_empty(), "library vial {} must be pure data", vial.id);
        for term in vial.terms() {
            if let Term::Str(s) = &term {
                for foreign in library_terms.iter().filter(|l| **l != own) {
                    assert!(
                        !s.starts_with(foreign) && !s.contains(&format!("import {foreign}")),
                        "vial {} leaks foreign library term {s:?}",
                        vial.id
                    );
                }
            }
        }
    }
}

#[test]
fn flagship_request_assembles_full_program() {
    let result = synthesize("Make a multiplayer grid game with a scoring system");
    assert_eq!(result.answers.len(), 1, "expected exactly one assembled program");
    let answer = &result.answers[0];
    let Some(Term::Str(code)) = answer.bindings.get("?code") else {
        panic!("?code did not bind to a string");
    };

    // Every requested capability made it into the program.
    assert!(code.starts_with("# assembled by the DSCE generic software assembler"));
    assert!(code.contains("import pygame"), "graphics layer missing");
    assert!(code.contains("import sqlite3"), "persistence layer missing");
    assert!(code.contains("import asyncio, websockets"), "networking layer missing");
    assert!(code.contains("def tick(state):"), "state machine missing");
    assert!(code.contains("def draw_grid(screen, grid"), "grid rendering missing");
    assert!(code.contains("def main():"), "loop glue missing");

    // MVC ordering: model before view before controller before loop.
    let model = code.find("STATES =").unwrap();
    let view = code.find("import pygame").unwrap();
    let controller = code.find("def poll_input").unwrap();
    let main_loop = code.find("def main():").unwrap();
    assert!(model < view && view < controller && controller < main_loop);

    // The proof traces back through the pattern rules to library axioms.
    let proof = result.proof(answer);
    assert!(proof.contains("assemble-application"));
    assert!(proof.contains("bind-capability"));
    assert!(proof.contains("asserted by query"));
    assert!(proof.contains("pygame 2.x documentation"));
}

#[test]
fn synthesis_is_deterministic() {
    let a = synthesize("Make a multiplayer grid game with a scoring system").summary();
    let b = synthesize("Make a multiplayer grid game with a scoring system").summary();
    assert_eq!(a, b);
}

#[test]
fn partial_request_stays_sparse() {
    // A grid-only request must not wake the database, networking, or FSM
    // vials — annotation predicates keep shared API-doc vocabulary from
    // building activation bridges.
    let result = synthesize("Draw a grid of tiles");
    assert!(result.answers.is_empty(), "no full program without model/controller");
    for dormant in ["sqlite_databases", "websockets_networking", "finite_state_machine"] {
        assert!(
            result.dormant.contains(&dormant.to_string()),
            "{dormant} should stay dormant, activated: {:?}",
            result.activated
        );
    }
    // The generic layout reasoning still happened.
    assert!(result
        .derivations
        .contains_key(&(t("draw_grid_tiles"), t("layout_flow"), t("row_major"))));
    assert!(result
        .derivations
        .contains_key(&(t("draw_grid_tiles"), t("uses"), t("pygame"))));
}

#[test]
fn vocabulary_triples_steer_generic_patterns() {
    // The same KB serves a completely different request zero-shot: a
    // persistent turn tracker gets the FSM + sqlite model layers without
    // any app-specific vial existing.
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let harvest = harvest_offline("Track game turns and save them to a database");
    let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
    let app = t(&harvest.app);
    assert!(result
        .derivations
        .keys()
        .any(|f| f.0 == app && f.1 == t("code_model")));
    // No grid was requested, so no grid rule concluded anything for the
    // app (the vial may still wake and pour its own axioms).
    assert!(!result
        .derivations
        .values()
        .any(|d| d.vial_id == "grid_layout" && d.rule_name.is_some()));
}

#[test]
fn random_number_generator_synthesis() {
    let result = synthesize("Make a random number generator");
    assert_eq!(result.answers.len(), 1);
    let answer = &result.answers[0];
    let Some(Term::Str(code)) = answer.bindings.get("?code") else {
        panic!("?code did not bind to a string");
    };
    assert!(code.starts_with("# assembled by the DSCE generic CLI assembler"));
    assert!(code.contains("import random"));
    assert!(code.contains("print(random.randint(1, 100))"));
}

#[test]
fn cpp_gui_app_synthesis() {
    let result = synthesize("Make a random number generator in C++ with a GUI");
    assert!(result.answers.len() >= 1, "expected at least one answer");
    let mut cpp_found = false;
    for answer in &result.answers {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            if code.starts_with("// assembled by the DSCE C++ GUI assembler") {
                assert!(code.contains("#include <QApplication>"));
                assert!(code.contains("QApplication app(argc, argv);"));
                assert!(code.contains("QObject::connect"));
                cpp_found = true;
            }
        }
    }
    assert!(cpp_found, "C++ GUI program was not assembled by the engine");
}
