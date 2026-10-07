//! Authoring manifest from the same boxes as collision and cable visibility.
use aether_core::terrain::{self, Surface, TerrainBox};
use serde_json::{Value, json};
fn pieces(boxes: Vec<TerrainBox>, origin: aether_core::glam::Vec3) -> Value {
    json!(boxes.iter().map(|p|json!({"center":(p.center-origin).to_array(),"size":p.size.to_array(),"surface":match p.surface {Surface::Rock=>"rock",Surface::Grass=>"grass",Surface::Trunk=>"trunk",Surface::Foliage=>"foliage",Surface::Crystal=>"crystal"}})).collect::<Vec<_>>())
}
fn main() {
    let mut manifest = serde_json::Map::new();
    for (index, island) in terrain::ISLANDS.iter().enumerate() {
        manifest.insert(
            format!("island-{index}"),
            pieces(
                terrain::island_pieces(island.center, island.radius, index as u32),
                island.center,
            ),
        );
    }
    manifest.insert(
        "sanctuary".into(),
        pieces(terrain::sanctuary_pieces(), terrain::SANCTUARY),
    );
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
}
