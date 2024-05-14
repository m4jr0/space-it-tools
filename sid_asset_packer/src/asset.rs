// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::sid::{
    self, SidAnimationFrameCoord, SidAnimationFrameDim, SidAnimationFrameDuration,
    SidAnimationFrameIndex, SidSpriteSheetDim,
};

pub static DEFAULT_ASSETS_PATH: &str = "./assets";

#[derive(Debug)]
pub enum SidAssetError {
    Malformed(String),
    IO(String),
}

impl fmt::Display for SidAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SidAssetError::Malformed(error) => write!(f, "Malformed error: {error}"),
            SidAssetError::IO(error) => write!(f, "I/O error: {error}"),
        }
    }
}

pub type SidAssetResult<T> = Result<T, SidAssetError>;

pub trait SidPackedAsset {
    fn namespace() -> &'static str;
    fn write_to_folder<P: AsRef<Path>>(&self, folder_path: P) -> SidAssetResult<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidSpriteSheetAsset {
    pub name: String,
    pub image_path: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    pub image_from_path: PathBuf,
    pub width: SidSpriteSheetDim,
    pub height: SidSpriteSheetDim,
    pub format: String,
}

impl SidSpriteSheetAsset {
    pub fn new() -> Self {
        let name = String::new();
        let image_path = PathBuf::new();
        let image_from_path = PathBuf::new();
        let width = 0;
        let height = 0;
        let format = String::new();

        Self::with_data(name, image_path, image_from_path, width, height, format)
    }

    pub fn with_data(
        name: String,
        image_path: PathBuf,
        image_from_path: PathBuf,
        width: SidSpriteSheetDim,
        height: SidSpriteSheetDim,
        format: String,
    ) -> Self {
        Self {
            name,
            image_path,
            image_from_path,
            width,
            height,
            format,
        }
    }
}

impl SidPackedAsset for SidSpriteSheetAsset {
    fn namespace() -> &'static str {
        return sid::get_sprite_sheet_namespace();
    }

    fn write_to_folder<P: AsRef<Path>>(&self, folder_path: P) -> SidAssetResult<()> {
        let out_json = serde_json::to_string_pretty(self).map_err(|error| {
            SidAssetError::Malformed(format!(
                "Unable to create sprite sheet asset {:?}: {error}",
                self.image_from_path
            ))
        })?;

        let folder_path = folder_path.as_ref();

        let out_path = if folder_path.is_absolute() {
            folder_path.join(Self::namespace())
        } else {
            let out_path = match std::fs::canonicalize(folder_path) {
                Ok(containing_folder_full_path) => containing_folder_full_path,
                Err(error) => {
                    return Err(SidAssetError::IO(format!(
                        "Unable to retrieve the containing folder full path: {error}"
                    )))
                }
            };

            out_path.join(Self::namespace())
        };

        fs::create_dir_all(&out_path).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to create folder at path {:?}: {}",
                &out_path, error
            ))
        })?;

        let in_sheet_path = self.image_from_path.clone();
        let out_sheet_path = out_path.join(&self.image_path);

        fs::copy(&in_sheet_path, &out_sheet_path).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to copy file from {:?} to {:?}: {}",
                in_sheet_path, out_sheet_path, error
            ))
        })?;

        let out_json_path = out_path.join(format!("{}.json", self.name));

        fs::write(&out_json_path, out_json).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to write JSON to file {:?}: {}",
                out_json_path, error
            ))
        })?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAnimationFramePos {
    pub x: SidAnimationFrameCoord,
    pub y: SidAnimationFrameCoord,
}

impl SidAnimationFramePos {
    pub fn new() -> Self {
        Self::with_coords(0, 0)
    }

    pub fn with_coords(x: SidAnimationFrameCoord, y: SidAnimationFrameCoord) -> Self {
        Self { x, y }
    }
}

impl Clone for SidAnimationFramePos {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAnimationFrameDims {
    pub width: SidAnimationFrameDim,
    pub height: SidAnimationFrameDim,
}

impl SidAnimationFrameDims {
    pub fn new() -> Self {
        Self::with_width_and_height(0, 0)
    }

    pub fn with_width_and_height(
        width: SidAnimationFrameDim,
        height: SidAnimationFrameDim,
    ) -> Self {
        Self { width, height }
    }
}

impl Clone for SidAnimationFrameDims {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAnimationFrameAsset {
    pub pos: SidAnimationFramePos,
    pub dims: SidAnimationFrameDims,
    pub duration: SidAnimationFrameDuration,
}

impl SidAnimationFrameAsset {
    pub fn new() -> Self {
        Self::with_data(SidAnimationFramePos::new(), SidAnimationFrameDims::new(), 0)
    }

    pub fn with_data(
        pos: SidAnimationFramePos,
        dim: SidAnimationFrameDims,
        duration: SidAnimationFrameDuration,
    ) -> Self {
        Self {
            pos,
            dims: dim,
            duration,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAnimationDefAsset {
    pub frame_count: SidAnimationFrameIndex,
    pub frames: Vec<SidAnimationFrameAsset>,
    pub name: String,
    pub sheet_name: String,
}

impl SidAnimationDefAsset {
    pub fn max_frame_count() -> SidAnimationFrameIndex {
        sid::get_max_animation_frame_count()
    }

    pub fn new() -> Self {
        let frame_count = 0;
        let frames = vec![];
        let name = String::new();
        let sheet_name = String::new();

        Self::with_data(frame_count, frames, name, sheet_name)
    }

    pub fn with_data(
        frame_count: SidAnimationFrameIndex,
        frames: Vec<SidAnimationFrameAsset>,
        name: String,
        sheet_name: String,
    ) -> Self {
        Self {
            frame_count,
            frames,
            name,
            sheet_name,
        }
    }
}

impl SidPackedAsset for SidAnimationDefAsset {
    fn namespace() -> &'static str {
        return sid::get_animation_def_namespace();
    }

    fn write_to_folder<P: AsRef<Path>>(&self, folder_path: P) -> SidAssetResult<()> {
        let out_json = serde_json::to_string_pretty(self).map_err(|error| {
            SidAssetError::Malformed(format!(
                "Unable to create animation definition asset {:?}: {error}",
                self.name
            ))
        })?;

        let out_path = folder_path.as_ref().join(Self::namespace());

        fs::create_dir_all(&out_path).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to create folder at path {:?}: {}",
                &out_path, error
            ))
        })?;

        let out_path = out_path.join(format!("{}.json", self.name));

        fs::write(&out_path, out_json).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to write JSON to file {:?}: {}",
                out_path, error
            ))
        })?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAnimationAsset {
    pub offset: SidAnimationFrameIndex,
    pub length: SidAnimationFrameIndex,
    pub name: String,
    pub def_name: String,
}

impl SidAnimationAsset {
    pub fn new() -> Self {
        let offset = 0;
        let length = 0;
        let name = String::new();
        let def_name = String::new();

        Self::with_data(offset, length, name, def_name)
    }

    pub fn with_data(
        offset: SidAnimationFrameIndex,
        length: SidAnimationFrameIndex,
        name: String,
        def_name: String,
    ) -> Self {
        Self {
            offset,
            length,
            name,
            def_name,
        }
    }

    pub fn from_def(
        def: &SidAnimationDefAsset,
        name: String,
        offset: SidAnimationFrameIndex,
        length: SidAnimationFrameIndex,
    ) -> Self {
        let def_name = def.name.clone();
        SidAnimationAsset::with_data(offset, length, name, def_name)
    }
}

impl SidPackedAsset for SidAnimationAsset {
    fn namespace() -> &'static str {
        return sid::get_animation_namespace();
    }

    fn write_to_folder<P: AsRef<Path>>(&self, folder_path: P) -> SidAssetResult<()> {
        let out_json = serde_json::to_string_pretty(self).map_err(|error| {
            SidAssetError::Malformed(format!(
                "Unable to create animation definition asset {:?}: {error}",
                self.name
            ))
        })?;

        let out_path = folder_path.as_ref().join(Self::namespace());

        fs::create_dir_all(&out_path).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to create folder at path {:?}: {}",
                &out_path, error
            ))
        })?;

        let out_path = out_path.join(format!("{}.json", self.name));

        fs::write(&out_path, out_json).map_err(|error| {
            SidAssetError::IO(format!(
                "Unable to write JSON to file {:?}: {}",
                out_path, error
            ))
        })?;

        Ok(())
    }
}
