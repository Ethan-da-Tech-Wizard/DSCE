// ==============================================================================
// Program: build.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: Cargo build script for DSCE. Compiles Slint UI definitions
//          (src/ui/appwindow.slint) during build time.
// ==============================================================================

fn main() {
    // Compile Slint graphical user interface definition file
    slint_build::compile("src/ui/appwindow.slint").unwrap();
}
