use bevy::{
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};

use super::spawn::sequencer::NUM_SYNTH_NOTES;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HandleMap<ImageKey>>();
    app.init_resource::<HandleMap<ImageKey>>();

    app.register_type::<HandleMap<SfxKey>>();
    app.init_resource::<HandleMap<SfxKey>>();

    app.register_type::<HandleMap<SoundtrackKey>>();
    app.init_resource::<HandleMap<SoundtrackKey>>();

    app.register_type::<HandleMap<FontKey>>();
    app.init_resource::<HandleMap<FontKey>>();
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum ImageKey {
    Player,
    Box,
    Spikes,
}

impl AssetKey for ImageKey {
    type Asset = Image;
}

impl FromWorld for HandleMap<ImageKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                ImageKey::Player,
                asset_server.load_with_settings(
                    "images/bb_atlas.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::Box,
                asset_server.load_with_settings(
                    "images/box.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
            (
                ImageKey::Spikes,
                asset_server.load_with_settings(
                    "images/spikes.png",
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::nearest();
                    },
                ),
            ),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SfxKey {
    ButtonHover,
    ButtonPress,
    Kick,
    Snare,
    HiHat,
    Synth(usize),
}

impl AssetKey for SfxKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SfxKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map: HandleMap<SfxKey> = [
            (
                SfxKey::ButtonHover,
                asset_server.load("audio/sfx/button_hover.ogg"),
            ),
            (
                SfxKey::ButtonPress,
                asset_server.load("audio/sfx/button_press.ogg"),
            ),
            (SfxKey::Kick, asset_server.load("audio/sfx/kick1.ogg")),
            (SfxKey::Snare, asset_server.load("audio/sfx/snare1.ogg")),
            (SfxKey::HiHat, asset_server.load("audio/sfx/hihat1.ogg")),
            (SfxKey::Synth(0), asset_server.load("audio/sfx/synth1.ogg")),
        ]
        .into();

        for i in 0..NUM_SYNTH_NOTES {
            map.insert(
                SfxKey::Synth(i),
                asset_server.load(format!("audio/sfx/synth{i}.ogg")),
            );
        }

        map
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SoundtrackKey {
    Title,
}

impl AssetKey for SoundtrackKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SoundtrackKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [(
            SoundtrackKey::Title,
            asset_server.load("audio/sfx/kick1.ogg"), //TODO
        )]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum FontKey {
    Title,
    General,
}

impl AssetKey for FontKey {
    type Asset = Font;
}

impl FromWorld for HandleMap<FontKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                FontKey::Title,
                asset_server.load("fonts/JosefinSans-Bold.ttf"),
            ),
            (
                FontKey::General,
                asset_server.load("fonts/Dosis-Regular.ttf"),
            ),
        ]
        .into()
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);

impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<K: AssetKey + Eq + std::hash::Hash> HandleMap<K> {
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.values()
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }

    /// Gets a handle to the asset with the provided key
    pub fn get(&self, key: K) -> Handle<K::Asset> {
        self[&key].clone_weak()
    }
}
