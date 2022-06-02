# lottie-rs
A Lottie file toolkit written in Rust.


# Samples

The following samples are gathered from lottiefiles.com community and lottiefiles.github.io. Credits
goes to original owners/creators of the files.

| **Name** | **Preview** |
|----------|-------------|
| Confetti | <details><summary>Preview</summary><video  src="https://user-images.githubusercontent.com/4921289/171586211-ba4ea8fe-fe91-4f91-a6ef-36e86ed3f12e.mp4"/></details> |
| Techno Penguin | <details><summary>Preview</summary><video  src="https://user-images.githubusercontent.com/4921289/171589321-8e9a812e-2b74-4395-963b-af868371b1da.mp4"/></details> |
| Drink | <details><summary>Preview</summary><video  src="https://user-images.githubusercontent.com/4921289/171589992-0dbb0280-b5cf-42fc-bace-2439c7e2ace6.mp4"/></details> |

# Try it out

The default player implementation uses [Bevy](https://github.com/bevyengine/bevy) to render the animation.

```bash
cd crates/player
cargo r --release -- ../../fixtures/ui/confetti.json
```

There are some lottie files for demonstration purpose under `fixtures/ui`
