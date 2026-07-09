slint::include_modules!();

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use slint::{Model, SharedString, VecModel};

use dsce::facts::Term;
use dsce::harvester::harvest_offline;
use dsce::json_vials::engine_from_dir;

fn main() -> Result<(), slint::PlatformError> {
    // 1. Load Vials from the synthesis directory
    let vials_dir = "vials_synthesis";
    let mut engine = match engine_from_dir(vials_dir) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Error loading vials from {vials_dir:?}: {e}");
            std::process::exit(1);
        }
    };

    // 2. Initialize Slint Window
    let window = AppWindow::new()?;
    
    // 3. Populate Vials List in UI
    let ui_vials = Rc::new(VecModel::default());
    let all_vials = &engine.vials;
    for (vial_id, vial) in all_vials {
        ui_vials.push(VialInfo {
            id: SharedString::from(vial_id),
            concept: SharedString::from(&vial.concept),
            facts_count: vial.facts.len() as i32,
            rules_count: vial.rules.len() as i32,
            activated: false,
        });
    }
    window.set_vials(ui_vials.clone().into());
    window.set_active_vials_count(SharedString::from(all_vials.len().to_string()));

    // 4. Synthesize Callback Handler
    let window_weak = window.as_weak();
    window.on_synthesize(move |prompt| {
        let window = window_weak.unwrap();
        window.set_engine_status(SharedString::from("SYNTHESIZING..."));

        let req = prompt.trim();
        if req.is_empty() {
            return;
        }

        // Run harvester
        let harvest = harvest_offline(req);
        
        // Execute Datalog flood reasoning
        let result = engine.ask_with_facts(&harvest.goal, &harvest.triples);
        
        // Determine which vials were activated
        let active_vial_ids = &result.activated;
        
        // Update vial active states in UI model
        let current_vials = window.get_vials();
        for i in 0..current_vials.row_count() {
            if let Some(mut vial) = current_vials.row_data(i) {
                let id_str = vial.id.as_str();
                vial.activated = active_vial_ids.iter().any(|v| v == id_str);
                current_vials.set_row_data(i, vial);
            }
        }

        // Extract compiled code blocks
        let mut assembled_code = String::new();
        for (i, answer) in result.answers.iter().enumerate() {
            if let Some(Term::Str(code)) = answer.bindings.get("?code") {
                if i > 0 {
                    assembled_code.push_str("\n\n");
                }
                assembled_code.push_str(code);
            }
        }
        if assembled_code.is_empty() {
            assembled_code = "No code was generated for this query request.\nVerify vocabulary mapping requirements.".to_string();
        }
        window.set_assembled_code(SharedString::from(assembled_code));

        // Format and render Datalog Proof Tree
        let ui_proof = Rc::new(VecModel::default());
        let lines = result.summary().split('\n').map(|s| s.to_string()).collect::<Vec<String>>();
        
        let mut in_answers = false;
        for line in lines {
            if line.starts_with("answer ") {
                in_answers = true;
                ui_proof.push(ProofLine {
                    text: SharedString::from(&line),
                    depth: 0,
                    is_rule: true,
                });
                continue;
            }
            if line.starts_with("--- assembled program") {
                in_answers = false;
                continue;
            }

            if in_answers && !line.trim().is_empty() {
                // Determine depth based on branch characters
                let branch_chars = line.chars().take_while(|c| *c == '├' || *c == '─' || *c == '│' || *c == '└' || c.is_whitespace()).count();
                let text = line[branch_chars..].trim().to_string();
                let is_rule = text.contains("[by rule");
                
                ui_proof.push(ProofLine {
                    text: SharedString::from(text),
                    depth: (branch_chars / 3) as i32,
                    is_rule,
                });
            }
        }
        window.set_proof_tree(ui_proof.into());
        window.set_engine_status(SharedString::from("READY"));
    });

    // 5. Run Sandbox Callback Handler
    let window_weak = window.as_weak();
    window.on_run_sandbox(move |code, lang| {
        let window = window_weak.unwrap();
        window.set_console_output(SharedString::from("Spawning sandbox runner...\n"));

        let code_str = code.trim();
        if code_str.is_empty() {
            window.set_console_output(SharedString::from("Error: No code to execute."));
            return;
        }

        // Auto-detect language extension
        let lang_lower = lang.to_lowercase();
        let (file_ext, cmd_name, cmd_args) = if lang_lower.contains("python") {
            ("py", "python3", vec![])
        } else if lang_lower.contains("go") {
            ("go", "go", vec!["run"])
        } else if lang_lower.contains("javascript") || lang_lower.contains("js") || lang_lower.contains("node") {
            ("js", "node", vec![])
        } else if lang_lower.contains("rust") {
            ("rs", "rustc", vec![]) // will compile and run manually
        } else if lang_lower.contains("c++") || lang_lower.contains("cpp") {
            ("cpp", "g++", vec![])
        } else if lang_lower.contains("c") {
            ("c", "gcc", vec![])
        } else if lang_lower.contains("sql") {
            ("sql", "sqlite3", vec![":memory:"])
        } else {
            // Default to Python
            ("py", "python3", vec![])
        };

        // Create scratch dir and write temp file
        let scratch_dir = Path::new("scratch");
        if !scratch_dir.exists() {
            fs::create_dir_all(scratch_dir).unwrap();
        }
        let file_path = scratch_dir.join(format!("temp_sandbox.{}", file_ext));
        fs::write(&file_path, code_str).unwrap();

        // Run execution command
        let output = if file_ext == "rs" {
            let binary = scratch_dir.join("temp_rust_bin");
            let build = Command::new("rustc")
                .arg(&file_path)
                .arg("-o")
                .arg(&binary)
                .output();
            match build {
                Ok(out) if out.status.success() => {
                    let run = Command::new(&binary).output();
                    match run {
                        Ok(run_out) => String::from_utf8_lossy(&run_out.stdout).to_string() + &String::from_utf8_lossy(&run_out.stderr),
                        Err(e) => format!("Error running binary: {e}"),
                    }
                }
                Ok(out) => "Compilation failed:\n".to_string() + &String::from_utf8_lossy(&out.stderr),
                Err(e) => format!("Error compiling Rust code: {e}"),
            }
        } else if file_ext == "cpp" || file_ext == "c" {
            let binary = scratch_dir.join(format!("temp_{}_bin", file_ext));
            let compiler = if file_ext == "cpp" { "g++" } else { "gcc" };
            let build = Command::new(compiler)
                .arg(&file_path)
                .arg("-o")
                .arg(&binary)
                .arg("-lsqlite3")
                .output();
            match build {
                Ok(out) if out.status.success() => {
                    let run = Command::new(&binary).output();
                    match run {
                        Ok(run_out) => String::from_utf8_lossy(&run_out.stdout).to_string() + &String::from_utf8_lossy(&run_out.stderr),
                        Err(e) => format!("Error running binary: {e}"),
                    }
                }
                Ok(out) => "Compilation failed:\n".to_string() + &String::from_utf8_lossy(&out.stderr),
                Err(e) => format!("Error compiling C/C++ code: {e}"),
            }
        } else if file_ext == "sql" {
            // Write input redirect manually
            let mut child = Command::new("sqlite3")
                .arg(":memory:")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            
            {
                let stdin = child.stdin.as_mut().expect("Failed to open stdin");
                stdin.write_all(code_str.as_bytes()).expect("Failed to write to stdin");
            }

            let out = child.wait_with_output().unwrap();
            String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)
        } else {
            // Script languages
            let mut args = cmd_args;
            args.push(file_path.to_str().unwrap());
            match Command::new(cmd_name).args(&args).output() {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr),
                Err(e) => format!("Error executing sandbox script: {e}"),
            }
        };

        window.set_console_output(SharedString::from(output));
    });

    window.run()
}
