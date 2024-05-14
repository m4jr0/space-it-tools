// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use sid_aseprite_converter::sid_aseprite;
use sid_asset_packer::asset;

use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let sheets_input_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    let assets_output_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        let mut assets_output_path = PathBuf::from(".");
        assets_output_path.push(asset::DEFAULT_ASSETS_PATH);
        assets_output_path
    };

    sid_aseprite::from_aseprite_sheets_to_sid_assets(sheets_input_path, assets_output_path);
}
