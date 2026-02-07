/*=============================================================================*/
/* File Path: Element/Maintain/Source/Library.rs                             */
/*=============================================================================*/
/* Module: Library                                                            */
/*                                                                            */
/* Brief Description: Entry point for the Build Orchestrator binary.          */
/*                                                                            */
/* RESPONSIBILITIES:                                                          */
/* ================                                                          */
/*                                                                            */
/* Primary:                                                                   */
/* - Serve as the entry point for the build orchestrator                       */
/* - Initialize the Build module for the binary                                */
/*                                                                            */
/* Secondary:                                                                 */
/* - None                                                                     */
/*                                                                            */
/* ARCHITECTURAL ROLE:                                                        */
/* ===================                                                        */
/*                                                                            */
/* Position:                                                                  */
/* - Entry point layer                                                         */
/* - Binary initialization                                                     */
/*                                                                            */
/* Dependencies (What this module requires):                                  */
/* - External crates: None                                                    */
/* - Internal modules: Build                                                  */
/* - Traits implemented: None                                                 */
/*                                                                            */
/* Dependents (What depends on this module):                                  */
/* - Cargo.toml (binary definition)                                           */
/* - Build system entry point                                                 */
/*                                                                            */
/*=============================================================================*/
/* IMPLEMENTATION                                                             */
/*=============================================================================*/

// Disable Windows console for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Allow PascalCase naming for function names
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

/*=============================================================================*/
/* MAIN ENTRY POINT                                                            */
/*=============================================================================*/

/// The primary entry point for the Build Orchestrator binary.
///
/// This function serves as the bridge between the Cargo binary definition
/// and the Build module's orchestration logic. It simply calls the main
/// `Fn` function from the Build module, which handles:
///
/// - Logger initialization
/// - Argument parsing
/// - Build orchestration
/// - Error handling and exit
///
/// The function is marked as `#[allow(dead_code)]` because when this file
/// is used as a library module, the main function may not be called directly.
/// However, when compiled as a binary, this main function is the entry point.
#[allow(dead_code)]
fn main() {
	Build::Fn();
}

/*=============================================================================*/
/* MODULE DECLARATIONS                                                        */
/*=============================================================================*/

/// Build Orchestrator Module.
///
/// This module contains all the build orchestration logic, including:
///
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments, manifests, and file guards
/// - **Error**: Comprehensive error types for build operations
/// - **Function**: Build orchestration functions and utilities
///
/// See the Build module documentation for detailed information about the
/// build system's capabilities and usage.
pub mod Build;
