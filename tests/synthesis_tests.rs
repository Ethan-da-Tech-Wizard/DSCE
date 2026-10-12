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
            "academic_concepts",
            "academic_rules",
            "c_std_library",
            "container_app",
            "cpp_gui_app",
            "cpp_std_library",
            "csharp_dotnet_library",
            "cuda_toolkit",
            "ddia_concepts",
            "ddia_rules",
            "dependency_resolution",
            "docker_container",
            "electron_framework",
            "english_dictionary",
            "express_framework",
            "faiss_vector_search",
            "fastapi_framework",
            "finite_state_machine",
            "framework_app",
            "go_std_library",
            "grid_layout",
            "gui_random_app",
            "kubernetes_orchestration",
            "mvc",
            "node_js_runtime",
            "polyglot_academic_app",
            "polyglot_calc_app",
            "polyglot_cli_app",
            "polyglot_db_app",
            "polyglot_ddia_app",
            "polyglot_game_app",
            "polyglot_gui_app",
            "polyglot_hybrid_app",
            "polyglot_multilang_app",
            "polyglot_multilib_app",
            "polyglot_ui_app",
            "polyglot_web_app",
            "pygame_graphics",
            "python_extended",
            "python_gui_libraries",
            "pytorch_framework",
            "qt_cpp_graphics",
            "random_generator",
            "react_framework",
            "rust_std_library",
            "sql_sqlite_library",
            "sqlite_databases",
            "tkinter_graphics",
            "vocabulary_mapping",
            "websockets_networking",
            "x86_64_assembly",
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
    assert!(result.answers.len() >= 1, "expected at least one assembled program");
    // Find the Pygame answer among the candidates
    let mut pygame_answer = None;
    for ans in &result.answers {
        if let Some(Term::Str(c)) = ans.bindings.get("?code") {
            if c.contains("import pygame") {
                pygame_answer = Some(ans.clone());
                break;
            }
        }
    }
    let answer = pygame_answer.expect("Should have compiled a pygame version of the flagship request");
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
    let proof = result.proof(&answer);
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
    assert!(code.starts_with("# assembled by the DSCE polyglot assembler (Python)"));
    assert!(code.contains("import random"));
    assert!(code.contains("print(random.randint(1, 100))"));
}

#[test]
fn cpp_gui_app_synthesis() {
    let result = synthesize("Make a random number generator in C++ and show me the GUI");
    assert!(result.answers.len() >= 1);
    let mut cpp_found = false;
    for answer in &result.answers {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            if code.starts_with("// assembled by the DSCE polyglot assembler (C++)")
                && code.contains("#include <QApplication>")
                && code.contains("QApplication app(argc, argv);")
            {
                cpp_found = true;
            }
        }
    }
    assert!(cpp_found, "C++ GUI program was not assembled by the engine");
}

/// All assembled program texts from one request.
fn assembled_codes(request: &str) -> Vec<String> {
    synthesize(request)
        .answers
        .iter()
        .filter_map(|a| match a.bindings.get("?code") {
            Some(Term::Str(code)) => Some(code.clone()),
            _ => None,
        })
        .collect()
}

/// The single code answer whose banner (first line) contains `marker`.
fn code_with_banner(request: &str, marker: &str) -> String {
    let codes = assembled_codes(request);
    let hits: Vec<&String> = codes
        .iter()
        .filter(|c| c.lines().next().unwrap_or("").contains(marker))
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "expected exactly one {marker:?} program for {request:?}, banners: {:?}",
        codes.iter().map(|c| c.lines().next().unwrap_or("")).collect::<Vec<_>>()
    );
    hits[0].clone()
}

#[test]
fn rust_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in Rust", "(Rust)");
    assert!(code.contains("use std::time::{SystemTime, UNIX_EPOCH};"));
    assert!(code.contains("fn main()"));
    assert!(code.contains("println!"));
}

#[test]
fn c_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in C", "(C)");
    assert!(code.contains("#include <stdlib.h>"));
    assert!(code.contains("srand((unsigned) time(NULL));"));
    assert!(code.contains("rand() % 100 + 1"));
}

#[test]
fn cpp_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in C++", "polyglot assembler (C++)");
    assert!(code.contains("#include <random>"));
    assert!(code.contains("std::mt19937"));
    assert!(code.contains("std::uniform_int_distribution<> dist(1, 100);"));
}

#[test]
fn csharp_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in C#", "(C#)");
    assert!(code.contains("using System;"));
    assert!(code.contains("new Random()"));
    assert!(code.contains("Console.WriteLine(random.Next(1, 101));"));
}

#[test]
fn go_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in Go", "(Go)");
    assert!(code.contains("package main"));
    assert!(code.contains("\"math/rand\""));
    assert!(code.contains("fmt.Println(rand.Intn(100) + 1)"));
}

#[test]
fn sql_random_synthesis() {
    let codes = assembled_codes("Write a random number generator in SQL");
    let mut sql_found = false;
    for code in &codes {
        if code.starts_with("-- assembled by the DSCE polyglot assembler (SQL)")
            && code.contains("SELECT (abs(random()) % 100) + 1")
        {
            sql_found = true;
        }
    }
    assert!(sql_found, "SQL random program was not assembled by the engine");
}

#[test]
fn assembly_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in assembly", "x86-64 NASM");
    assert!(code.contains("global _start"));
    assert!(code.contains("rdtsc"));
    assert!(code.contains("syscall"));
}

#[test]
fn javascript_random_cli_synthesis() {
    let code = code_with_banner("Make a random number generator in JavaScript", "Node.js");
    assert!(code.contains("console.log(Math.floor(Math.random() * 100) + 1);"));
}

#[test]
fn explicit_language_suppresses_python_answer() {
    // An explicit Rust request must not also assemble the Python CLI
    // program — the Python assembler is guarded on target_language.
    for code in assembled_codes("Make a random number generator in Rust") {
        assert!(
            !code.starts_with("# assembled by the DSCE polyglot assembler (Python)"),
            "Python program leaked into a Rust request"
        );
    }
}

#[test]
fn python_stays_the_default_language() {
    // No language named: the classic Python path still answers, alone.
    let result = synthesize("Make a random number generator");
    assert_eq!(result.answers.len(), 1);
    // Explicitly named Python routes to the same single program.
    let code = code_with_banner("Make a random number generator in Python", "polyglot assembler (Python)");
    assert!(code.contains("import random"));
}

#[test]
fn dockerfile_synthesis() {
    let code = code_with_banner("Containerize the service with Docker", "(Dockerfile)");
    assert!(code.contains("FROM python:3.12-slim"));
    assert!(code.contains("CMD [\"python\", \"main.py\"]"));
}

#[test]
fn kubernetes_manifest_synthesis() {
    let code = code_with_banner("Deploy the service with Kubernetes", "(Kubernetes)");
    assert!(code.contains("apiVersion: apps/v1"));
    assert!(code.contains("kind: Deployment"));
    assert!(code.contains("containerPort: 8080"));
}

#[test]
fn pytorch_framework_synthesis_with_install_step() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let harvest = harvest_offline("Make a PyTorch tensor multiplication script");
    let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
    let app = t(&harvest.app);
    let code = result
        .answers
        .iter()
        .find_map(|a| match a.bindings.get("?code") {
            Some(Term::Str(c)) if c.contains("import torch") => Some(c.clone()),
            _ => None,
        })
        .expect("PyTorch script was not assembled");
    assert!(code.contains("torch.matmul"));
    // Dependency reasoning: install channel and Python runtime dependency.
    assert!(result
        .derivations
        .contains_key(&(app.clone(), t("install_step"), t("pip install torch"))));
    assert!(result
        .derivations
        .contains_key(&(app, t("requires_dependency"), t("python_runtime"))));
}

#[test]
fn electron_dependencies_resolve_transitively() {
    // Electron depends on Node.js and Chromium; Node.js depends on V8 and
    // libuv. The dependency closure must surface all four.
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let harvest = harvest_offline("Make an Electron desktop app");
    let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
    let app = t(&harvest.app);
    for dep in ["node_js", "chromium", "v8_engine", "libuv"] {
        assert!(
            result
                .derivations
                .contains_key(&(app.clone(), t("requires_dependency"), t(dep))),
            "missing transitive dependency {dep}"
        );
    }
    let codes: Vec<String> = result
        .answers
        .iter()
        .filter_map(|a| match a.bindings.get("?code") {
            Some(Term::Str(c)) => Some(c.clone()),
            _ => None,
        })
        .collect();
    assert!(
        codes.iter().any(|c| c.contains("BrowserWindow")),
        "Electron main-process script was not assembled"
    );
}

#[test]
fn faiss_vector_search_synthesis() {
    let code = code_with_banner("Build a FAISS vector search program", "(FAISS)");
    assert!(code.contains("faiss.IndexFlatL2"));
    assert!(code.contains("index.search"));
}

#[test]
fn cpp_dictionary_synthesis() {
    let result = synthesize("Make a dictionary with a search function in C++");
    assert!(result.answers.len() >= 1);
    let mut dict_found = false;
    for answer in &result.answers {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            if code.starts_with("// assembled by the DSCE C++ GUI dictionary assembler") {
                assert!(code.contains("#include <map>"));
                assert!(code.contains("#include <QLineEdit>"));
                assert!(code.contains("std::map<std::string, std::string> dict"));
                assert!(code.contains("QObject::connect"));
                dict_found = true;
            }
        }
    }
    assert!(dict_found, "C++ GUI dictionary program was not assembled by the engine");
}

#[test]
fn polyglot_gui_app_synthesis_python() {
    let code = code_with_banner("Make a GUI in Python", "Python");
    assert!(code.contains("import tkinter as tk"));
    assert!(code.contains("root.mainloop()"));
}

#[test]
fn polyglot_gui_app_synthesis_cpp() {
    let code = code_with_banner("Make a GUI in C++", "polyglot assembler (C++)");
    assert!(code.contains("#include <QApplication>"));
    assert!(code.contains("QApplication app"));
}

#[test]
fn polyglot_web_app_synthesis_rust() {
    let code = code_with_banner("Make a web server in Rust", "Rust");
    assert!(code.contains("use std::net::TcpListener;"));
    assert!(code.contains("TcpListener::bind"));
}

#[test]
fn polyglot_web_app_synthesis_go() {
    let code = code_with_banner("Make a web server in Go", "Go");
    assert!(code.contains("net/http"));
    assert!(code.contains("ListenAndServe"));
}

#[test]
fn polyglot_db_app_synthesis_python() {
    let code = code_with_banner("Make a database app in Python", "Python");
    assert!(code.contains("import sqlite3"));
    assert!(code.contains("sqlite3.connect"));
}

#[test]
fn polyglot_db_app_synthesis_cpp() {
    let code = code_with_banner("Make a database app in C++", "C++");
    assert!(code.contains("#include <sqlite3.h>"));
    assert!(code.contains("sqlite3_open"));
}

#[test]
fn polyglot_db_app_synthesis_sql() {
    let code = code_with_banner("Make a database app in SQL", "SQL");
    assert!(code.contains("CREATE TABLE users"));
    assert!(code.contains("INSERT INTO users"));
}

#[test]
fn fastapi_framework_synthesis() {
    let code = code_with_banner("Make a fastapi web server", "FastAPI");
    assert!(code.contains("from fastapi import FastAPI"));
    assert!(code.contains("FastAPI()"));
}

#[test]
fn express_framework_synthesis() {
    let code = code_with_banner("Make an express web server", "Express");
    assert!(code.contains("require('express')"));
    assert!(code.contains("app.listen"));
}

#[test]
fn polyglot_calc_app_synthesis_python() {
    let code = code_with_banner("Make a calculator in Python", "Python");
    assert!(code.contains("def add(a, b):"));
    assert!(code.contains("Calculator Demo:"));
}

#[test]
fn polyglot_calc_app_synthesis_go() {
    let code = code_with_banner("Make a calculator in Go", "Go");
    assert!(code.contains("package main"));
    assert!(code.contains("func add(a, b float64)"));
}

#[test]
fn polyglot_calc_app_synthesis_assembly() {
    let code = code_with_banner("Make a calculator in assembly", "NASM");
    assert!(code.contains("global _start"));
    assert!(code.contains("add eax, 5"));
}

#[test]
fn polyglot_game_app_synthesis_python() {
    let code = code_with_banner("Make a game in Python", "Python");
    assert!(code.contains("IN-DEPTH INSTRUCTIONS FOR PYTHON TKINTER"));
    assert!(code.contains("canvas.create_oval"));
}

#[test]
fn polyglot_game_app_synthesis_go() {
    let code = code_with_banner("Make a game in Go", "Go");
    assert!(code.contains("IN-DEPTH INSTRUCTIONS FOR RETRO 8-BIT GO CHASE"));
    assert!(code.contains("playerPos"));
    assert!(code.contains("lionPos"));
}

#[test]
fn polyglot_game_app_synthesis_assembly() {
    let code = code_with_banner("Make a game in assembly", "NASM");
    assert!(code.contains("IN-DEPTH INSTRUCTIONS FOR ASSEMBLY GAME"));
    assert!(code.contains("mov eax, 42"));
}

#[test]
fn polyglot_multilang_tool_synthesis_go_js() {
    let code = code_with_banner("Make a multi-language tool in Go", "Go");
    // Contains Go backend web server
    assert!(code.contains("package main"));
    assert!(code.contains("net/http"));
    // Contains JS frontend logic
    assert!(code.contains("assembled by the DSCE polyglot assembler (JavaScript / Node.js)"));
    assert!(code.contains("console.log"));
}

#[test]
fn ddia_architectural_inference_and_code_synthesis() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_data_app");
    
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("high_write_throughput")),
        (app.clone(), t("needs"), t("prevent_write_skew")),
        (app.clone(), t("needs"), t("high_availability_consensus")),
        (app.clone(), t("needs"), t("even_write_distribution")),
        (app.clone(), t("target_language"), t("lang_python")),
    ];
    
    // 1. Logical recommendation assertions
    let goal = (app.clone(), t("recommended_storage"), t("?storage"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    assert_eq!(result.answers[0].bindings.get("?storage").unwrap(), &t("lsm_tree"));

    let goal_iso = (app.clone(), t("recommended_isolation"), t("?iso"));
    let result_iso = engine.ask_with_facts(&goal_iso, &extra_facts);
    assert_eq!(result_iso.answers.len(), 1);
    assert_eq!(result_iso.answers[0].bindings.get("?iso").unwrap(), &t("serializability"));

    // 2. Code synthesis assertions
    let goal_code = (app.clone(), t("code"), t("?code"));
    let result_code = engine.ask_with_facts(&goal_code, &extra_facts);
    
    assert!(result_code.answers.len() >= 3);
    
    let mut lsm_found = false;
    let mut raft_found = false;
    let mut hash_found = false;
    
    for answer in &result_code.answers {
        if let Some(Term::Str(code)) = answer.bindings.get("?code") {
            if code.contains("class LSMTree:") && code.contains("wal_path") && code.contains("__tombstone__") {
                lsm_found = true;
            }
            if code.contains("class RaftReplica:") && code.contains("request_vote") && code.contains("votes") {
                raft_found = true;
            }
            if code.contains("class HashRouter:") && code.contains("md5") {
                hash_found = true;
            }
        }
    }
    
    assert!(lsm_found, "LSM-Tree storage code with WAL/Tombstones should be synthesized");
    assert!(raft_found, "Raft consensus replica code should be synthesized");
    assert!(hash_found, "Hash partitioning code with md5 routing should be synthesized");
}

#[test]
fn polyglot_ui_synthesis_python() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_ui_app");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("complete_gui")),
        (app.clone(), t("target_language"), t("lang_python")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("tkinter"));
    assert!(code.contains("BG_COLOR"));
    assert!(code.contains("grid_frame"));
    assert!(code.contains("mainloop"));
}

#[test]
fn polyglot_ui_synthesis_assembly() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_asm_ui");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("complete_gui")),
        (app.clone(), t("target_language"), t("lang_assembly")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("section .data"));
    assert!(code.contains("frame: db"));
    assert!(code.contains("RETRO ASSEMBLY INTERFACE"));
}

#[test]
fn polyglot_ui_synthesis_sql() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_sql_ui");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("complete_gui")),
        (app.clone(), t("target_language"), t("lang_sql")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("SELECT"));
    assert!(code.contains("SQL DATABASE INTERFACE"));
    assert!(code.contains("UNION ALL"));
}

#[test]
fn polyglot_academic_synthesis_python() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_academic_app");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("academic_suite")),
        (app.clone(), t("needs"), t("randomized_bst")),
        (app.clone(), t("needs"), t("eval_apply_loop")),
        (app.clone(), t("needs"), t("intrusive_linked_list")),
        (app.clone(), t("needs"), t("decouple_algorithm_from_client")),
        (app.clone(), t("target_language"), t("lang_python")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("class TreapNode:"), "Treap structures should be synthesized");
    assert!(code.contains("def sicp_eval("), "Metacircular evaluator should be synthesized");
    assert!(code.contains("class ListHead:"), "Linux intrusive doubly-linked list should be synthesized");
    assert!(code.contains("class SortingStrategy:"), "Strategy pattern should be synthesized");
}

#[test]
fn polyglot_academic_synthesis_go() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_academic_app_go");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("academic_suite")),
        (app.clone(), t("needs"), t("randomized_bst")),
        (app.clone(), t("needs"), t("eval_apply_loop")),
        (app.clone(), t("needs"), t("intrusive_linked_list")),
        (app.clone(), t("needs"), t("decouple_algorithm_from_client")),
        (app.clone(), t("target_language"), t("lang_go")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("type TreapNode struct"), "Go TreapNode should be synthesized");
    assert!(code.contains("type EnvFrame struct"), "Go EnvFrame should be synthesized");
    assert!(code.contains("func ListAdd("), "Go ListAdd should be synthesized");
    assert!(code.contains("type SortingStrategy interface"), "Go Strategy interface should be synthesized");
}

#[test]
fn polyglot_academic_synthesis_rust() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_academic_app_rust");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("academic_suite")),
        (app.clone(), t("needs"), t("randomized_bst")),
        (app.clone(), t("needs"), t("eval_apply_loop")),
        (app.clone(), t("needs"), t("intrusive_linked_list")),
        (app.clone(), t("needs"), t("decouple_algorithm_from_client")),
        (app.clone(), t("target_language"), t("lang_rust")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("struct TreapNode"), "Rust TreapNode should be synthesized");
    assert!(code.contains("struct EnvFrame"), "Rust EnvFrame should be synthesized");
    assert!(code.contains("unsafe fn add("), "Rust unsafe add list should be synthesized");
    assert!(code.contains("trait SortingStrategy"), "Rust SortingStrategy trait should be synthesized");
}

#[test]
fn polyglot_multilib_synthesis_python() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_multilib_app");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("multilib_suite")),
        (app.clone(), t("needs"), t("data_manipulation")),
        (app.clone(), t("needs"), t("numerical_arrays")),
        (app.clone(), t("needs"), t("machine_learning")),
        (app.clone(), t("needs"), t("http_requests")),
        (app.clone(), t("target_language"), t("lang_python")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("import pandas as pd"), "Pandas imports should be synthesized");
    assert!(code.contains("import numpy as np"), "NumPy imports should be synthesized");
    assert!(code.contains("from sklearn.linear_model import LinearRegression"), "Scikit-Learn imports should be synthesized");
    assert!(code.contains("import requests"), "Requests imports should be synthesized");
    assert!(code.contains("df['Scaled'] = df['Value'] * 1.5"), "Pandas processing logic should be synthesized");
    assert!(code.contains("np.linalg.eigvals"), "NumPy eigenvalue computation should be synthesized");
}

#[test]
fn polyglot_hybrid_synthesis_python() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_hybrid_app");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("needs"), t("gameboy_hybrid")),
        (app.clone(), t("target_language"), t("lang_python")),
        (app.clone(), t("ball_color"), t("cyan")),
        (app.clone(), t("ball_speed"), t("3")),
        (app.clone(), t("calc_limit"), t("10")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("def press_key(key):"), "Calculator code should be synthesized");
    assert!(code.contains("tk.Canvas("), "Game code should be synthesized");
}

#[test]
fn polyglot_modular_game_synthesis() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_complex_game");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("target_language"), t("lang_python")),
        (app.clone(), t("game_tier"), t("mid_tier")),
        (app.clone(), t("needs"), t("twin_stick_combat")),
        (app.clone(), t("needs"), t("tower_defense_grid")),
        (app.clone(), t("needs"), t("wave_spawning")),
        (app.clone(), t("needs"), t("procedural_generation")),
        (app.clone(), t("weapon_type"), t("shoot_standard")),
        (app.clone(), t("move_modifier"), t("move_standard")),
        (app.clone(), t("tower_type"), t("tower_standard")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code.contains("import pygame"), "Pygame import should be synthesized");
    assert!(code.contains("player_pos ="), "Player position init should be synthesized");
    assert!(code.contains("def update_game():"), "update_game loop should be synthesized");
    assert!(code.contains("def draw_game():"), "draw_game loop should be synthesized");
    assert!(code.contains("clock.tick(60)"), "Frame rate limiter should be synthesized");
}

#[test]
fn polyglot_dynamic_game_synthesis() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_shotgun_game");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("target_language"), t("lang_python")),
        (app.clone(), t("game_tier"), t("mid_tier")),
        (app.clone(), t("needs"), t("twin_stick_combat")),
        (app.clone(), t("needs"), t("tower_defense_grid")),
        (app.clone(), t("needs"), t("wave_spawning")),
        (app.clone(), t("needs"), t("procedural_generation")),
        (app.clone(), t("weapon_type"), t("shoot_shotgun")),
        (app.clone(), t("move_modifier"), t("move_dash")),
        (app.clone(), t("tower_type"), t("tower_tesla")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    // Assert shotgun vectors
    assert!(code.contains("for a_offset in [-0.25, 0.0, 0.25]:"), "Shotgun code should be synthesized");
    // Assert dash controls
    assert!(code.contains("if keys[pygame.K_LSHIFT]: mult = 3.0"), "Dash modifier should be synthesized");
    // Assert tesla coils
    assert!(code.contains("pygame.draw.rect(screen, (0, 180, 255),"), "Tesla coil draw should be synthesized");
}

#[test]
fn polyglot_python_gui_frameworks_synthesis() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();

    // Test PySide6 App
    let app1 = t("my_pyside_app");
    let extra_facts1 = vec![
        (app1.clone(), t("is_a"), t("application")),
        (app1.clone(), t("target_language"), t("lang_python")),
        (app1.clone(), t("needs_gui_framework"), t("pyside6")),
    ];
    let goal1 = (app1.clone(), t("code"), t("?code"));
    let result1 = engine.ask_with_facts(&goal1, &extra_facts1);
    assert_eq!(result1.answers.len(), 1);
    let code1 = match result1.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code1.contains("PySide6.QtWidgets import QApplication"), "PySide6 imports should be present");

    // Test CustomTkinter App
    let app2 = t("my_ctk_app");
    let extra_facts2 = vec![
        (app2.clone(), t("is_a"), t("application")),
        (app2.clone(), t("target_language"), t("lang_python")),
        (app2.clone(), t("needs_gui_framework"), t("customtkinter")),
    ];
    let goal2 = (app2.clone(), t("code"), t("?code"));
    let result2 = engine.ask_with_facts(&goal2, &extra_facts2);
    assert_eq!(result2.answers.len(), 1);
    let code2 = match result2.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code2.contains("import customtkinter as ctk"), "CustomTkinter imports should be present");

    // Test Textual TUI App
    let app3 = t("my_textual_app");
    let extra_facts3 = vec![
        (app3.clone(), t("is_a"), t("application")),
        (app3.clone(), t("target_language"), t("lang_python")),
        (app3.clone(), t("needs_gui_framework"), t("textual")),
    ];
    let goal3 = (app3.clone(), t("code"), t("?code"));
    let result3 = engine.ask_with_facts(&goal3, &extra_facts3);
    assert_eq!(result3.answers.len(), 1);
    let code3 = match result3.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    assert!(code3.contains("from textual.app import App"), "Textual imports should be present");
}

#[test]
fn polyglot_neon_rift_synthesis() {
    let mut engine = engine_from_dir(vials_dir()).unwrap();
    let app = t("my_neon_rift_defense");
    let extra_facts = vec![
        (app.clone(), t("is_a"), t("application")),
        (app.clone(), t("target_language"), t("lang_python")),
        (app.clone(), t("game_tier"), t("high_tier")),
        (app.clone(), t("needs"), t("twin_stick_combat")),
        (app.clone(), t("needs"), t("tower_defense_grid")),
        (app.clone(), t("needs"), t("wave_spawning")),
        (app.clone(), t("needs"), t("procedural_generation")),
        (app.clone(), t("needs"), t("boss_ai_fsm")),
        (app.clone(), t("needs"), t("status_effect_ticker")),
        (app.clone(), t("needs"), t("currency_economy")),
        (app.clone(), t("weapon_type"), t("shoot_shotgun")),
        (app.clone(), t("move_modifier"), t("move_dash")),
        (app.clone(), t("tower_type"), t("tower_tesla")),
    ];
    let goal = (app.clone(), t("code"), t("?code"));
    let result = engine.ask_with_facts(&goal, &extra_facts);
    assert_eq!(result.answers.len(), 1);
    let code = match result.answers[0].bindings.get("?code").unwrap() {
        Term::Str(s) => s,
        _ => panic!("Expected string code"),
    };
    // Assert 7-system loops
    assert!(code.contains("boss_active = False"), "Boss FSM should be initialized");
    assert!(code.contains("status_slow_multiplier = 0.5"), "Status ticker should be initialized");
    assert!(code.contains("scrap = 100"), "Currency economy should be initialized");
    assert!(code.contains("scrap -= tower_cost"), "Tower purchase costs should be updated");
    assert!(code.contains("boss_telegraph_timer ="), "Boss AI ticks should be updated");
}

#[test]
fn polyglot_declarative_harvesting() {
    let result = synthesize("Make a form in python that displays flashing colors");
    assert!(result.answers.len() >= 1, "expected at least one assembled program");
    
    // Find the Tkinter GUI answer among candidates
    let mut tkinter_code = None;
    for ans in &result.answers {
        if let Some(Term::Str(c)) = ans.bindings.get("?code") {
            if c.contains("import tkinter") {
                tkinter_code = Some(c.clone());
                break;
            }
        }
    }
    
    let code = tkinter_code.expect("Should have compiled a tkinter GUI based on declarative vocabulary mapping");
    assert!(code.contains("import tkinter as tk"));
    assert!(code.contains("root = tk.Tk()"));
}




