use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
pub enum AppState {
    Connecting,
    Downloading,
    Cutting,
    Playing,
}
