// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use image::GenericImageView;
use serde::Deserialize;

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    asset::{SidAnimationAsset, SidAnimationDefAsset, SidPackedAsset, SidSpriteSheetAsset},
    sid::{self, sid_texture_format},
    sid_error, sid_warning,
};

#[derive(Debug)]
pub enum SidAssetSerializationError {
    UnsupportedFormat(String),
    IO(String),
}

pub static DEFAULT_RESOURCES_PATH: &str = "./resources";

pub type SidAssetSerializationResult<T> = Result<T, SidAssetSerializationError>;

pub trait SerializeSidAsset {
    fn write_resource<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        in_path: P1,
        out_folder: P2,
    ) -> SidAssetSerializationResult<()>;
}

trait SidResourceWrite: Write {
    fn write_packed(&mut self, data: &[u8]) -> SidAssetSerializationResult<()> {
        match self.write_all(data) {
            Ok(img) => img,
            Err(err) => return Err(SidAssetSerializationError::IO(err.to_string())),
        };

        Ok(())
    }
}

impl SidResourceWrite for File {}

impl SerializeSidAsset for SidSpriteSheetAsset {
    fn write_resource<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        in_path: P1,
        out_folder: P2,
    ) -> SidAssetSerializationResult<()> {
        let id = sid::generate_sprite_sheet_id(&self.name);
        let path = out_folder.as_ref().join(id.to_string());

        let mut texture_path = match in_path.as_ref().parent() {
            Some(texture_path) => PathBuf::from(texture_path),
            None => {
                return Err(SidAssetSerializationError::IO(format!(
                    "Unable to retrieve texture path"
                )));
            }
        };

        texture_path.push(&self.image_path);

        let texture = match image::open(texture_path) {
            Ok(texture) => texture,
            Err(error) => return Err(SidAssetSerializationError::IO(error.to_string())),
        };

        let (width, height) = texture.dimensions();
        let channel_count = texture.color().channel_count();

        let format = match texture.color() {
            image::ColorType::Rgb8 => sid_texture_format::SID_TEXTURE_FORMAT_RGB8,
            image::ColorType::Rgba8 => sid_texture_format::SID_TEXTURE_FORMAT_RGBA8,
            format => {
                return Err(SidAssetSerializationError::UnsupportedFormat(format!(
                    "Unknown or unsupported format: {:?}",
                    format
                )))
            }
        };

        let raw_format = format as i32;

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(error) => return Err(SidAssetSerializationError::IO(error.to_string())),
        };

        file.write_packed(&id.to_le_bytes())?;
        file.write_packed(&width.to_le_bytes())?;
        file.write_packed(&height.to_le_bytes())?;
        file.write_packed(&channel_count.to_le_bytes())?;
        file.write_packed(&raw_format.to_le_bytes())?;

        let texture_size =
            (width as sid::UIndex) * (height as sid::UIndex) * channel_count as sid::UIndex;

        file.write_packed(&texture_size.to_le_bytes())?;
        file.write_packed(&texture.as_bytes())?;

        Ok(())
    }
}

impl SerializeSidAsset for SidAnimationDefAsset {
    fn write_resource<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        _: P1,
        out_folder: P2,
    ) -> SidAssetSerializationResult<()> {
        let id = sid::generate_animation_def_id(&self.name);
        let sheet_id = sid::generate_sprite_sheet_id(&self.sheet_name);
        let path = out_folder.as_ref().join(id.to_string());

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => return Err(SidAssetSerializationError::IO(err.to_string())),
        };

        file.write_packed(&id.to_le_bytes())?;
        file.write_packed(&sheet_id.to_le_bytes())?;
        file.write_packed(&self.frame_count.to_le_bytes())?;

        for frame in &self.frames {
            file.write_packed(&frame.pos.x.to_le_bytes())?;
            file.write_packed(&frame.pos.y.to_le_bytes())?;

            file.write_packed(&frame.dims.width.to_le_bytes())?;
            file.write_packed(&frame.dims.height.to_le_bytes())?;

            file.write_packed(&frame.duration.to_le_bytes())?;
        }

        Ok(())
    }
}

impl SerializeSidAsset for SidAnimationAsset {
    fn write_resource<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        _: P1,
        out_folder: P2,
    ) -> SidAssetSerializationResult<()> {
        let id = sid::generate_animation_id(&self.name);
        let def_id = sid::generate_animation_def_id(&self.def_name);
        let path = out_folder.as_ref().join(id.to_string());

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => return Err(SidAssetSerializationError::IO(err.to_string())),
        };

        file.write_packed(&id.to_le_bytes())?;
        file.write_packed(&def_id.to_le_bytes())?;
        file.write_packed(&self.offset.to_le_bytes())?;
        file.write_packed(&self.length.to_le_bytes())?;

        Ok(())
    }
}

trait SidAssetProcessor: SidPackedAsset
where
    for<'de> Self: Deserialize<'de>,
    Self: Sized + SidPackedAsset + SerializeSidAsset,
{
    fn compatible(entry: &fs::DirEntry) -> bool {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => {
                return false;
            }
        };

        if !file_type.is_file() {
            return false;
        }

        let path = entry.path();

        let extension = match path.extension() {
            Some(extension) => extension,
            None => {
                return false;
            }
        };

        let extension = match extension.to_str() {
            Some(extension) => extension,
            None => {
                return false;
            }
        };

        return Self::extension_compatible(extension);
    }

    fn assets_folder<P: AsRef<Path>>(folder_name: P) -> bool {
        let folder_name = match folder_name.as_ref().to_str() {
            Some(folder_name) => folder_name,
            None => {
                return false;
            }
        };

        Self::namespace() == folder_name
    }

    fn process_asset<P1: AsRef<Path>, P2: AsRef<Path>>(
        asset_input_path: P1,
        resources_output_path: P2,
    ) {
        let file = File::open(&asset_input_path).unwrap();
        let asset: Self = serde_json::from_reader(file).unwrap();
        asset
            .write_resource(asset_input_path, resources_output_path)
            .unwrap();
    }

    fn extension_compatible(extension: &str) -> bool;

    fn process_assets<P1: AsRef<Path>, P2: AsRef<Path>>(
        assets_input_path: P1,
        resources_output_path: P2,
    ) {
        let entries = match fs::read_dir(assets_input_path) {
            Ok(entries) => entries,
            Err(error) => {
                sid_error!("Error retrieving directory entries: {error}");
                return;
            }
        };

        for entry in entries.flatten() {
            if !Self::compatible(&entry) {
                continue;
            }

            Self::process_asset(entry.path(), &resources_output_path);
        }
    }
}

impl SidAssetProcessor for SidSpriteSheetAsset {
    fn extension_compatible(extension: &str) -> bool {
        extension == "json"
    }
}

impl SidAssetProcessor for SidAnimationDefAsset {
    fn extension_compatible(extension: &str) -> bool {
        extension == "json"
    }
}

impl SidAssetProcessor for SidAnimationAsset {
    fn extension_compatible(extension: &str) -> bool {
        extension == "json"
    }
}

pub fn from_assets_to_resources<P: AsRef<Path>>(assets_input_path: P, resources_output_path: P) {
    fs::create_dir_all(&resources_output_path).unwrap_or_else(|error| {
        sid_error!(
            "Unable to create folder at path {:?}: {}",
            assets_input_path.as_ref(),
            error
        );

        return;
    });

    let entries = fs::read_dir(&assets_input_path)
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

        let metadata = match fs::metadata(path.clone()) {
            Ok(metadata) => metadata,
            Err(err) => {
                sid_error!("Error while processing an entry: {err}");
                continue;
            }
        };

        if !metadata.is_dir() {
            sid_warning!("Ignoring entry (a folder is expected): {:?}", path.to_str());
            continue;
        }

        let folder_name = match std::path::Path::new(&path).file_name() {
            Some(folder_name) => folder_name,
            None => {
                sid_error!(
                    "Unknown error while processing an entry: {:?}",
                    path.to_str()
                );
                continue;
            }
        };

        match folder_name {
            folder_name if SidSpriteSheetAsset::assets_folder(folder_name) => {
                SidSpriteSheetAsset::process_assets(path, &resources_output_path);
            }
            folder_name if SidAnimationDefAsset::assets_folder(folder_name) => {
                SidAnimationDefAsset::process_assets(path, &resources_output_path);
            }
            folder_name if SidAnimationAsset::assets_folder(folder_name) => {
                SidAnimationAsset::process_assets(path, &resources_output_path);
            }
            _ => sid_warning!(
                "Ignoring entry (unknown or unsupported namespace): {:?}",
                path
            ),
        }
    }
}
