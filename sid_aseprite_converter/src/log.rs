// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! sid_debug {
    ($($arg:tt)*) => (println!("[DEBUG] {}", format_args!($($arg)*)));
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! sid_info {
    ($($arg:tt)*) => (println!("[INFO] {}", format_args!($($arg)*)));
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! sid_warning {
    ($($arg:tt)*) => (println!("[WARNING] {}", format_args!($($arg)*)));
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! sid_debug {
    ($($arg:tt)*) => {{}};
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! sid_info {
    ($($arg:tt)*) => {{}};
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! sid_warning {
    ($($arg:tt)*) => {{}};
}

#[macro_export]
macro_rules! sid_error {
    ($($arg:tt)*) => (eprintln!("[ERROR] {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! sid_fatal_error {
    ($($arg:tt)*) => (panic!("[FATAL ERROR] {}", format_args!($($arg)*)));
}
