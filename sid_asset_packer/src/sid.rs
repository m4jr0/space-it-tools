// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use std::{ffi::CStr, ffi::CString};

pub type SChar = i8;
pub type SidStringId = u32;

pub type SidSpriteSheetId = SidStringId;
pub type SidSpriteSheetDim = u32;

pub type SidAnimationDefId = SidStringId;
pub type SidAnimationId = SidStringId;
pub type SidAnimationFrameDim = u16;
pub type SidAnimationFrameCoord = u16;
pub type SidAnimationFrameIndex = u16;
pub type SidAnimationFrameDuration = u16;

pub type UIndex = u64;

#[link(name = "sid_lib", kind = "static")]
extern "C" {
    fn sid_get_animation_namespace() -> *const SChar;
    fn sid_get_animation_def_namespace() -> *const SChar;
    fn sid_get_max_animation_frame_count() -> u16;
    fn sid_generate_animation_def_id(name: *const SChar) -> SidAnimationDefId;
    fn sid_generate_animation_id(name: *const SChar) -> SidAnimationId;
    fn sid_get_sprite_sheet_namespace() -> *const SChar;
    fn sid_generate_sprite_sheet_id(name: *const SChar) -> SidSpriteSheetId;
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum sid_texture_format {
    SID_TEXTURE_FORMAT_UNKNOWN,
    SID_TEXTURE_FORMAT_RGB8,
    SID_TEXTURE_FORMAT_RGBA8,
}

macro_rules! sid_namespace {
    ($fn:ident) => {{
        static mut NAMESPACE: Option<&'static str> = None;

        unsafe {
            if let Some(namespace) = NAMESPACE {
                namespace
            } else {
                let namespace = $fn();
                let namespace = CStr::from_ptr(namespace);
                let namespace = namespace.to_str().expect("Invalid UTF-8 in value");
                NAMESPACE = Some(namespace);
                namespace
            }
        }
    }};
}

macro_rules! generate_string_id {
    ($name:expr, $id_generator:ident) => {{
        let name = CString::new($name).expect("Failed to create CString");
        let name: *const SChar = name.as_ptr();
        unsafe { $id_generator(name) }
    }};
}

// Public API below.
pub fn get_animation_namespace() -> &'static str {
    sid_namespace!(sid_get_animation_namespace)
}

pub fn get_animation_def_namespace() -> &'static str {
    sid_namespace!(sid_get_animation_def_namespace)
}

pub fn get_max_animation_frame_count() -> SidAnimationFrameIndex {
    unsafe { sid_get_max_animation_frame_count() }
}

pub fn generate_animation_def_id(name: &str) -> SidAnimationDefId {
    generate_string_id!(name, sid_generate_animation_def_id)
}

pub fn generate_animation_id(name: &str) -> SidAnimationId {
    generate_string_id!(name, sid_generate_animation_id)
}

pub fn get_sprite_sheet_namespace() -> &'static str {
    sid_namespace!(sid_get_sprite_sheet_namespace)
}

pub fn generate_sprite_sheet_id(name: &str) -> SidSpriteSheetId {
    generate_string_id!(name, sid_generate_sprite_sheet_id)
}
