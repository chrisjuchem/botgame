use bevy::prelude::*;

pub struct FontPlugin;
impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_font);
        app.add_systems(PostUpdate, update_font_size);
    }
}

#[derive(Resource)]
pub struct DefaultFont(pub Handle<Font>);
impl DefaultFont {
    pub fn handle(&self) -> Handle<Font> {
        self.0.clone()
    }
}

pub fn load_font(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle = asset_server.load("NotoSansMonoWithEmoji.ttf");
    commands.insert_resource(DefaultFont(handle));
}

#[derive(Component, Deref)]
pub struct DynamicFontSize(pub f32);

pub fn update_font_size(
    windows: Query<&Window>,
    mut texts: Query<(&mut Text, &mut Style, &DynamicFontSize)>,
    mut prev_height: Local<f32>,
) {
    let Ok(window) = windows.get_single() else { return };
    let h = window.height();
    if *prev_height != h {
        *prev_height = h;
        for (mut text, mut style, size) in &mut texts {
            scale_text(&mut *text, &mut *style, size, h);
        }
    }
}

pub fn scale_text(text: &mut Text, style: &mut Style, size: &DynamicFontSize, window_h: f32) {
    for section in &mut text.sections {
        section.style.font_size = **size * window_h / 720.;
    }
    style.top = Val::Px(**size / -12.) // updated as Vh changes;
}

#[derive(Clone)]
pub struct CustomText {
    pub value: String,
    pub color: Color,
    pub font_size: f32,
    pub alignment: TextAlignment,
}
impl Default for CustomText {
    fn default() -> Self {
        Self {
            value: String::new(),
            color: Color::BLACK,
            font_size: 36.,
            alignment: TextAlignment::Left,
        }
    }
}

impl CustomText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            value: text.into(),
            color: Color::BLACK,
            font_size: 36.,
            alignment: TextAlignment::Left,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.value = text.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn centered(mut self) -> Self {
        self.alignment = TextAlignment::Center;
        self
    }
}
