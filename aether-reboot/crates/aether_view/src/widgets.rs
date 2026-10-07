use bevy::prelude::*;
pub const INK: Color = Color::srgb(0.045, 0.064, 0.090);
pub const PAPER: Color = Color::srgb(0.93, 0.92, 0.86);
pub const MUTED: Color = Color::srgb(0.58, 0.65, 0.69);
pub const ACCENT: Color = Color::srgb(0.65, 0.49, 1.0);
#[derive(Resource)]
pub struct Typography {
    pub body: Handle<Font>,
    pub display: Handle<Font>,
}
pub fn load_fonts(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Typography {
        body: assets.load("fonts/notosans.ttf"),
        display: assets.load("fonts/cormorantgaramond.ttf"),
    });
}
pub fn apply_fonts(
    fonts: Res<Typography>,
    assets: Res<Assets<Font>>,
    mut text: Query<&mut TextFont>,
) {
    for mut font in &mut text {
        let display = matches!(font.font_size, bevy::text::FontSize::Px(size) if size >= 30.0);
        let handle = if display { &fonts.display } else { &fonts.body };
        if assets.contains(handle.id()) {
            let source = bevy::text::FontSource::Handle(handle.clone());
            if font.font != source {
                font.font = source;
            }
        }
    }
}
pub fn label(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}
pub fn panel() -> Node {
    Node {
        padding: UiRect::all(px(18)),
        flex_direction: FlexDirection::Column,
        row_gap: px(10),
        border_radius: BorderRadius::all(px(8)),
        ..default()
    }
}
