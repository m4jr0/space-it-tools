// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use serde::Deserialize;
use serde_json::Value;

use sid_asset_packer::asset::{
    SidAnimationAsset, SidAnimationDefAsset, SidAnimationFrameAsset, SidAnimationFrameDims,
    SidAnimationFramePos, SidPackedAsset, SidSpriteSheetAsset,
};
use sid_asset_packer::sid;

use std::cmp::min;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use crate::{sid_error, sid_warning};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteSize {
    w: i16,
    h: i16,
}

impl AsepriteSize {
    pub fn new() -> Self {
        Self::with_size(0, 0)
    }

    pub fn with_size(w: i16, h: i16) -> Self {
        Self { w, h }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteRect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

impl AsepriteRect {
    pub fn new() -> Self {
        Self::with_coords_and_size(0, 0, 0, 0)
    }

    pub fn with_coords_and_size(x: i16, y: i16, w: i16, h: i16) -> Self {
        Self { x, y, w, h }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteFrameData {
    frame: AsepriteRect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: AsepriteRect,
    source_size: AsepriteSize,
    duration: i32,
}

impl AsepriteFrameData {
    pub fn new() -> Self {
        let frame = AsepriteRect::new();
        let rotated = false;
        let trimmed = false;
        let sprite_source_size = AsepriteRect::new();
        let source_size = AsepriteSize::new();
        let duration = 0;

        Self {
            frame,
            rotated,
            trimmed,
            sprite_source_size,
            source_size,
            duration,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteFrameSlice {}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteFrameLayer {
    name: String,
    opacity: u8,
    blend_mode: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteFrameTag {
    name: String,
    from: u8,
    to: u8,
    direction: String,
    color: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteMeta {
    app: String,
    version: String,
    image: PathBuf,
    format: String,
    size: AsepriteSize,
    scale: String,
    frame_tags: Vec<AsepriteFrameTag>,
    layers: Vec<AsepriteFrameLayer>,
    slices: Vec<AsepriteFrameSlice>,
}

impl AsepriteMeta {
    pub fn new() -> Self {
        let app = String::new();
        let version = String::new();
        let image = PathBuf::new();
        let format = String::new();
        let size = AsepriteSize::new();
        let scale = String::new();
        let frame_tags = vec![];
        let layers = vec![];
        let slices = vec![];

        Self {
            app,
            version,
            image,
            format,
            size,
            scale,
            frame_tags,
            layers,
            slices,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AsepriteFrameTuple {
    name: String,
    data: AsepriteFrameData,
}

impl AsepriteFrameTuple {
    pub fn new(name: String) -> Self {
        let data = AsepriteFrameData::new();
        Self { name, data }
    }
}

#[derive(Debug)]
pub enum AsepriteSheetError {
    Malformed(String),
    IO(String),
}

impl fmt::Display for AsepriteSheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsepriteSheetError::Malformed(error) => write!(f, "Malformed error: {error}"),
            AsepriteSheetError::IO(error) => write!(f, "I/O error: {error}"),
        }
    }
}

pub type AsepriteSheetResult<T> = Result<T, AsepriteSheetError>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct AsepriteSheet {
    frames: Vec<AsepriteFrameTuple>,
    meta: AsepriteMeta,
}

impl AsepriteSheet {
    pub fn new() -> Self {
        let frames = vec![];
        let meta = AsepriteMeta::new();
        Self { frames, meta }
    }

    pub fn from_json<P: AsRef<Path>>(path: P) -> AsepriteSheetResult<Self> {
        let contents = fs::read_to_string(&path).map_err(|_| {
            AsepriteSheetError::IO(format!("Failed to read file {:?}", &path.as_ref()))
        })?;

        let aseprite_json: Value = serde_json::from_str(&contents).map_err(|_| {
            AsepriteSheetError::Malformed(format!("Failed to parse JSON file {:?}", &path.as_ref()))
        })?;

        let map = match aseprite_json {
            Value::Object(map) => map,
            _ => {
                return Err(AsepriteSheetError::Malformed(format!(
                    "Invalid JSON structure in file {:?}",
                    &path.as_ref()
                )))
            }
        };

        let mut descr = Self::new();

        for (key, value) in map {
            match key.as_str() {
                "frames" => {
                    if let Value::Object(obj) = value {
                        for (key, value) in obj {
                            let mut tuple = AsepriteFrameTuple::new(key);
                            tuple.data = serde_json::from_value(value).map_err(|_| {
                                AsepriteSheetError::Malformed(
                                    "Failed to deserialize frame data".to_string(),
                                )
                            })?;
                            descr.frames.push(tuple);
                        }
                    }
                }
                "meta" => {
                    let meta: AsepriteMeta = serde_json::from_value(value).map_err(|_| {
                        AsepriteSheetError::Malformed("Failed to deserialize meta data".to_string())
                    })?;
                    descr.meta = meta;
                }
                _ => {}
            }
        }

        Ok(descr)
    }
}

pub trait FromAsepriteSheet<T> {
    fn from_aseprite_sheet<P: AsRef<Path>>(
        containing_folder: P,
        sheet: &AsepriteSheet,
    ) -> Option<T>;
}

impl FromAsepriteSheet<SidSpriteSheetAsset> for SidSpriteSheetAsset {
    fn from_aseprite_sheet<P: AsRef<Path>>(
        containing_folder: P,
        sheet: &AsepriteSheet,
    ) -> Option<SidSpriteSheetAsset> {
        let meta = &sheet.meta;
        let width = meta.size.w;
        let height = meta.size.h;

        if width <= 0 || height <= 0 {
            sid_error!(
                "Invalid size ({width}x{height}) for Aseprite sheet {:?}",
                meta.image
            );

            return None;
        }

        let width = width as sid::SidSpriteSheetDim;
        let height = height as sid::SidSpriteSheetDim;

        let image_name = meta.image.clone();

        let path = if meta.image.is_absolute() {
            meta.image.clone()
        } else {
            let path = match std::fs::canonicalize(containing_folder.as_ref()) {
                Ok(containing_folder_full_path) => containing_folder_full_path,
                Err(error) => {
                    sid_error!("Unable to retrieve the containing folder full path: {error}");
                    return None;
                }
            };

            path.join(&meta.image)
        };

        let name = match path.file_stem() {
            Some(name) => name,
            None => {
                sid_error!("Invalid name for Aseprite sheet {:?}", path);
                return None;
            }
        };

        let name = match name.to_os_string().into_string() {
            Ok(name) => name,
            Err(_) => {
                sid_error!("Invalid name for Aseprite sheet {:?}", path);
                return None;
            }
        };

        let format = meta.format.clone();
        let sheet = SidSpriteSheetAsset::with_data(name, image_name, path, width, height, format);

        Some(sheet)
    }
}

pub trait FromAsepriteFrameTuplesAndSidSpriteSheet<T> {
    fn from_aseprite_frame_tuples_and_sid_sprite_sheet(
        aseprite_tuples: &[AsepriteFrameTuple],
        sheet: &SidSpriteSheetAsset,
    ) -> Option<T>;
}

impl FromAsepriteFrameTuplesAndSidSpriteSheet<SidAnimationDefAsset> for SidAnimationDefAsset {
    fn from_aseprite_frame_tuples_and_sid_sprite_sheet(
        aseprite_tuples: &[AsepriteFrameTuple],
        sheet: &SidSpriteSheetAsset,
    ) -> Option<SidAnimationDefAsset> {
        let frame_count = min(
            aseprite_tuples.len(),
            SidAnimationDefAsset::max_frame_count() as usize,
        ) as u16;
        let mut frames = Vec::<SidAnimationFrameAsset>::with_capacity(frame_count as usize);

        for tuple in aseprite_tuples {
            let data = &tuple.data;
            let frame = &data.frame;

            let x: u16 = match frame.x.try_into() {
                Ok(x) => x,
                Err(error) => {
                    sid_error!(
                        "Error while processing a frame pos (x) with {:?}: {error}",
                        sheet.image_from_path
                    );
                    return None;
                }
            };

            let y: u16 = match frame.y.try_into() {
                Ok(y) => y,
                Err(error) => {
                    sid_error!(
                        "Error while processing a frame pos (y) with {:?}: {error}",
                        sheet.image_from_path
                    );
                    return None;
                }
            };

            let pos = SidAnimationFramePos::with_coords(x, y);

            let width: u16 = match frame.w.try_into() {
                Ok(width) => width,
                Err(error) => {
                    sid_error!(
                        "Error while processing a frame dimensions (width) with {:?}: {error}",
                        sheet.image_from_path
                    );
                    return None;
                }
            };

            let height: u16 = match frame.h.try_into() {
                Ok(height) => height,
                Err(error) => {
                    sid_error!(
                        "Error while processing a frame dimensions (height) with {:?}: {error}",
                        sheet.image_from_path
                    );
                    return None;
                }
            };

            let dim = SidAnimationFrameDims::with_width_and_height(width, height);

            let duration: u16 = match data.duration.try_into() {
                Ok(duration) => duration,
                Err(error) => {
                    sid_error!(
                        "Error while processing a frame duration with {:?}: {error}",
                        sheet.image_from_path
                    );
                    return None;
                }
            };

            let frame_asset = SidAnimationFrameAsset::with_data(pos, dim, duration);
            frames.push(frame_asset);
        }

        let sheet_name = sheet.name.clone();
        let name = sheet_name.clone();
        let animation_def = SidAnimationDefAsset::with_data(frame_count, frames, name, sheet_name);

        Some(animation_def)
    }
}

pub fn from_aseprite_frame_name_to_animation_name(frame_name: &str) -> Option<&str> {
    let mut start_index = None;
    let mut end_index = None;

    for (i, c) in frame_name.char_indices() {
        if c == '(' {
            start_index = Some(i + 1);
        } else if c == ')' {
            end_index = Some(i);
            break;
        }
    }

    if let (Some(start), Some(end)) = (start_index, end_index) {
        Some(&frame_name[start..end].trim())
    } else {
        None
    }
}

pub fn from_aseprite_sheet_to_sid_animations<P: AsRef<Path>>(
    sheet: &AsepriteSheet,
    def: &SidAnimationDefAsset,
    assets_output_path: P,
) {
    if sheet.frames.len() == 0 {
        sid_warning!("No animation provided from sheet {:?}", sheet.meta.image);
        return;
    }

    let mut offset = 0;
    let mut last_anim_name = "";

    for i in 0..sheet.frames.len() {
        if i > u16::MAX as usize {
            sid_error!(
                "Too many animations added from sheet {:?}. Aborting.",
                sheet.meta.image
            );

            return;
        }

        let tuple = &sheet.frames[i];

        let anim_name = match from_aseprite_frame_name_to_animation_name(&tuple.name) {
            Some(anim_name) => anim_name,
            None => {
                sid_error!(
                    "Malformed animation name from sheet {:?}. Aborting.",
                    sheet.meta.image
                );

                return;
            }
        };

        if i == 0 {
            last_anim_name = anim_name;
            continue;
        }

        if anim_name.eq(last_anim_name) {
            continue;
        }

        let mut final_anim_name = String::with_capacity(last_anim_name.len() + def.name.len() + 1);

        final_anim_name.push_str(&def.name[..def.name.len()]);
        final_anim_name.push('_');
        final_anim_name.push_str(last_anim_name);

        let i = i as u16;
        let sid_asset = SidAnimationAsset::from_def(def, final_anim_name, offset, i - offset);
        last_anim_name = anim_name;
        offset = i;

        if let Err(error) = sid_asset.write_to_folder(&assets_output_path) {
            sid_error!("{error}");
        }
    }

    let mut final_anim_name = String::with_capacity(last_anim_name.len() + def.name.len() + 1);

    final_anim_name.push_str(&def.name[..def.name.len()]);
    final_anim_name.push('_');
    final_anim_name.push_str(last_anim_name);

    let sid_asset = SidAnimationAsset::from_def(
        def,
        final_anim_name.to_string(),
        offset,
        sheet.frames.len() as u16 - offset,
    );

    if let Err(error) = sid_asset.write_to_folder(&assets_output_path) {
        sid_error!("{error}");
    }
}

pub fn from_aseprite_sheets_to_sid_assets<P: AsRef<Path>>(
    sheets_input_path: P,
    assets_output_path: P,
) {
    let entries = fs::read_dir(&sheets_input_path)
        .unwrap_or_else(|error| panic!("Failed to read directory: {error}"));

    for entry in entries {
        if let Err(error) = entry {
            sid_error!("Error iterating over directory entry: {error}");
            continue;
        }

        let path = match entry {
            Ok(entry) => entry.path(),
            Err(err) => {
                sid_error!("Error while processing an entry: {err}");
                continue;
            }
        };

        if let Some(extension) = path.extension() {
            if extension != "json" {
                continue;
            }

            let sheet = match AsepriteSheet::from_json(&path) {
                Ok(sheet) => sheet,
                Err(error) => {
                    sid_error!("{error}");
                    continue;
                }
            };

            let sid_asset =
                match SidSpriteSheetAsset::from_aseprite_sheet(&sheets_input_path, &sheet) {
                    Some(sid_asset) => sid_asset,
                    None => continue,
                };

            if let Err(error) = sid_asset.write_to_folder(&assets_output_path) {
                sid_error!("{error}");
            }

            let sid_asset =
                match SidAnimationDefAsset::from_aseprite_frame_tuples_and_sid_sprite_sheet(
                    &sheet.frames,
                    &sid_asset,
                ) {
                    Some(sid_asset) => sid_asset,
                    None => continue,
                };

            if let Err(error) = sid_asset.write_to_folder(&assets_output_path) {
                sid_error!("{error}");
            }

            from_aseprite_sheet_to_sid_animations(&sheet, &sid_asset, &assets_output_path);
        }
    }
}
