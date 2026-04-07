//! Maintain binary entry point
//!
//! This file serves as the main entry point for the Maintain application.
//! It calls the main function from the library.

fn main() {
	// DEPENDENCY: Move the main function from Library.rs here in a future refactor
	// Currently Library.rs contains both lib and binary code
	// For now, delegate to the library's main function
	use Maintain::main as lib_main;
	lib_main();
}
