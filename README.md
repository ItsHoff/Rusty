# Rusty the Rendering Engine

WIP physically based renderer.

## Features

- OpenGl preview
- Path tracing
- Diffuse reflection
- Specular reflection + refraction
- Glossy reflection + refraction

## Installation

Requires nightly [Rust](https://www.rust-lang.org/en-US/install.html)
```
git clone https://github.com/ItsHoff/Rusty.git
cd Rusty
cargo run --release
```

## Keybindings
| Key | Function |
|-----|----------|
| W A S D E Q | Move camera |
| Left Mouse + drag | Rotate camera |
| Arrow Keys | Rotate camera |
| Space | Start & stop path tracing |
| Number Keys | Change scene |

## Loading scenes
Number keys change between the default scenes. Alternate scenes can be loaded by dragging and dropping a scene file into the window. Currently only .obj scenes are supported. Most scenes should render properly, but not all quirks will be supported.
