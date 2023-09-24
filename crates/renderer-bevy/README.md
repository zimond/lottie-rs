<div align="center">
  <h1><code>lottie-render-bevy</code></h1>
  <p>
    <strong>A Lottie JSON file renderer using Bevy as the rendering engine</strong>
  </p>
</div>

# lottie-render-bevy
Using [Bevy](https://github.com/bevyengine/bevy) as the engine to render Lottie
files.

Bevy is chosen as a renderer because:

- It is relatively mature, has an active community and clean design
- Allows interacting with entities as a game engine, this enables future interactive
  Lottie file editing apps
- Supports both 2D and 3D
- Supports lyon, which is a good choice for vectorized graphics rendering on GPU

# Known limitation

## Audio looping

Due to https://github.com/RustAudio/rodio/issues/176 , `Bevy` do not support audio seeking, so audio
playback when looping animation in this renderer is bugged.

We may try to directly use [Symphonia](https://docs.rs/symphonia/latest/symphonia) to play audio in the future.