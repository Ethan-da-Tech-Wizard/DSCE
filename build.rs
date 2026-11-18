// Program: DSCE - build.rs
// Programmer: Ethan-da-Tech-Wizard (written with Claude, an AI assistant)
// Last Modified: 2026-11-18
// Purpose: Cargo build script that compiles the Slint UI definition into Rust code.
fn main() {
    slint_build::compile("src/ui/appwindow.slint").unwrap();
}
