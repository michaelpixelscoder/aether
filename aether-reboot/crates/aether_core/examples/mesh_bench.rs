use aether_core::{Block, Body, fixtures, mesh};
use serde_json::json;
use std::time::Instant;
fn measure(name: &str, bodies: Vec<Body>) -> serde_json::Value {
    let mut samples = Vec::new();
    let mut triangles = 0;
    let mut vertices = 0;
    for run in 0..24 {
        let start = Instant::now();
        triangles = 0;
        vertices = 0;
        for body in &bodies {
            for coord in body.grid().chunk_coords() {
                let surfaces = mesh::chunk_mesh(body.grid(), coord);
                for surface in surfaces.values() {
                    triangles += surface.indices.len() / 3;
                    vertices += surface.positions.len();
                }
                std::hint::black_box(surfaces);
            }
        }
        if run >= 3 {
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    samples.sort_by(f64::total_cmp);
    json!({"fixture":name,"cells":bodies.iter().map(|b|b.grid().len()).sum::<usize>(),"bodies":bodies.len(),"chunks":bodies.iter().map(|b|b.grid().chunk_coords().count()).sum::<usize>(),"cpu_mesh_ms":{"p50":samples[10],"p95":samples[19],"max":samples[20]},"triangles":triangles,"vertices":vertices,"samples":21,"warmup":3})
}
fn main() {
    println!("{}",serde_json::to_string_pretty(&json!({"profile":if cfg!(debug_assertions){"dev opt1 deps2"}else{"release"},"gpu_measured":false,"results":[
        measure("1k",vec![fixtures::solid([10,10,10],Block::Wood)]),
        measure("10k",vec![fixtures::solid([20,25,20],Block::Wood)]),
        measure("50k stress / 5 legal bodies",(0..5).map(|_|fixtures::solid([20,25,20],Block::Wood)).collect()),
    ]})).unwrap());
}
