#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    {
        let document = web_sys::window().unwrap().document().unwrap();
        let games = document.get_element_by_id("games").unwrap();
        games.set_inner_html(
            r#"<a class="game" href="/game/shipwright/">
                <span class="game-title">Aether Shipwright</span>
                <span>Build a skyship voxel by voxel</span>
                <strong>Launch →</strong>
            </a>
            <a class="game" href="/game/sandbox/">
                <span class="game-title">Sandbox</span>
                <span>First-person movement on the grid ground</span>
                <strong>Launch →</strong>
            </a>"#,
        );
    }
}
