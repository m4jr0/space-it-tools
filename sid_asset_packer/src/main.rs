// Copyright 2024 m4jr0. All Rights Reserved.
// Use of this source code is governed by the MIT
// license that can be found in the LICENSE file.

use sid_asset_packer::{
    asset,
    resource::{self, from_assets_to_resources},
};

use std::{env, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();

    let assets_input_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        let mut assets_input_path = PathBuf::from(".");
        assets_input_path.push(asset::DEFAULT_ASSETS_PATH);
        assets_input_path
    };

    let resources_output_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        let mut resources_output_path = PathBuf::from(".");
        resources_output_path.push(resource::DEFAULT_RESOURCES_PATH);
        resources_output_path
    };

    from_assets_to_resources(assets_input_path, resources_output_path);
}
