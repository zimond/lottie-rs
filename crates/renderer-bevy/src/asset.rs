use bevy::reflect::TypeUuid;
use lottie_core::Precomposition;

#[derive(TypeUuid)]
#[uuid = "760e41e4-94c0-44e7-bbc8-f00ea42d2420"]
pub struct PrecompositionAsset {
    data: Precomposition,
}
