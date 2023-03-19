use std::collections::HashMap;

use anyhow::Error;
use tiled::{Chunk, Loader};

pub struct GameMap {
  pub map:          tiled::Map,
  main_layer_index: usize,
}

impl GameMap {
  pub fn from_resources(
    resources: &HashMap<String, Vec<u8>>,
    map_name: &str,
  ) -> Result<Self, Error> {
    let mut loader = Loader::new();

    // Preload all tilesets.
    for (name, data) in resources {
      //crate::log(&format!("Inspecting resource: {}", name));
      if name.ends_with(".tsx") {
        //crate::log(&format!(">> Loading tileset: {}", name));
        let ts = loader.populate_tsx_cache_from(&data[..], name)?;
        //println!("Tileset: {}", name);
      }
    }

    // Load the map.
    let map = loader.load_tmx_map_from(&resources[map_name][..], map_name)?;

    // Select the one layer whose name is "Main".
    let main_layer_index =
      map.layers().position(|layer| layer.name == "Main").expect("No layer named 'Main'");

    Ok(Self {
      map,
      main_layer_index,
    })
  }

  pub fn get_main_layer(&self) -> tiled::Layer {
    self.map.get_layer(self.main_layer_index).unwrap()
  }
}
