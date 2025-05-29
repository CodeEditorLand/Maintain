#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

#[allow(dead_code)]
fn main() { Build::Fn(); }

pub mod Build;
