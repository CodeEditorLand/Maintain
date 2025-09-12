#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_upper_case_globals)]

#[allow(dead_code)]
fn main() { Build::Fn(); }

pub mod Build;
