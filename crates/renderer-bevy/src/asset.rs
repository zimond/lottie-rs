use std::path::{Path, PathBuf};

use bevy::asset::{AssetIo, AssetIoError, BoxedFuture};
use bevy::prelude::{App, AssetServer, Plugin};
use bevy::reflect::TypeUuid;
use bevy::tasks::IoTaskPool;
use lottie_core::Precomposition;

#[derive(TypeUuid)]
#[uuid = "760e41e4-94c0-44e7-bbc8-f00ea42d2420"]
pub struct PrecompositionAsset {
    data: Precomposition,
}
