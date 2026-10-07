//! Read-only export for art verification and recorded keyboard playthroughs.
fn main() {
    if std::env::args().any(|arg| arg == "--traffic-sweep") {
        let samples: Vec<_> = aether_core::traffic::COURIERS.iter().filter(|c| c.sailing_body().is_some()).flat_map(|c| {
            (0..1000).map(move |step| {
                let courier = aether_core::traffic::Courier { phase: step as f32 / 1000.0, ..*c };
                serde_json::json!({"id":courier.id,"phase":courier.phase,"position":courier.pose(120.0).0,"scale":courier.scale()})
            })
        }).collect();
        println!("{}", serde_json::json!(samples));
        return;
    }
    let islands: Vec<_> = aether_core::world::islands().iter().map(|i| {
        let resources: Vec<_> = aether_core::expedition::resource_nodes(i).map(|n|
            serde_json::json!({"slot":n.slot,"goods":n.goods,"position":n.position,"radius":n.radius})).collect();
        serde_json::json!({"id":i.id,"biome":i.biome,"capital":i.capital,"center":i.center,"dock":i.dock(),"resources":resources})
    }).collect();
    let couriers: Vec<_> = aether_core::traffic::COURIERS
        .iter()
        .map(|c| {
            let (position, rotation) = c.pose(120.0);
            serde_json::json!({"id":c.id,"position":position,"rotation":rotation,"scale":c.scale()})
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"islands":islands,"routes":aether_core::world::routes(),"couriers_at_120_seconds":couriers})
    );
}
