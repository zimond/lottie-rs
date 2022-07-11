# lottie-rs
A Lottie file toolkit written in Rust.


# Samples

The following samples are gathered from lottiefiles.com community and lottiefiles.github.io. Credits
goes to original owners/creators of the files.

| **Name** | **Preview** |
|----------|-------------|
| Confetti | <img src="fixtures/results/confetti.webp" width="200"> |
| Techno Penguin | <img src="fixtures/results/techno_penguin.webp" width="200"> |
| Drink | <img src="fixtures/results/drink.webp" width="200"> |

# Try it out

The default player implementation uses [Bevy](https://github.com/bevyengine/bevy) to render the animation.

```bash
cd crates/player
cargo r --release -- --input ../../fixtures/ui/7148-the-nyan-cat.json
```

There are some lottie files for demonstration purpose under `fixtures/ui`

# Headless runner

Exporting animation headlessly is also supported, aiming to render animations on a server. Currently we support
`webp` exporting for test purpose.

```bash
cd crates/player
cargo r --release -- --input ../../fixtures/ui/7148-the-nyan-cat.json --headless
```

# Font Loading

This library uses [font-toolkit](https://github.com/alibaba/font-toolkit) to manage/load/use fonts, which
is also MIT-licensed.
