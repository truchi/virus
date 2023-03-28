# Virus

> Vi with/for 🦀

My attempt at the best code editor in the universe. (ETA Sep. 3023)

# TL;DR

What `virus` aims to be/have:

- **GUI** (ligatures, emojis and smooth scroll!)
- **Modal** (selection first, multi cursor)
- **Tree-sitter** (read my rant below)
- **Telescope** (but better)
- **Cargo/Git tasks**
- **Command palette/Which key**
- **LSP** (can we dream?)

What I *should* resist doing to achieve these goals:
- Rewrite `ropey`
- Font variations axis animation (with Recursive)
- Lottie renderer (for Noto animated emojis)
- Magit levels of git integration

What `virus` will never be/have:
- Mouse
- Windows/Tabs (ok, maybe)
- Configuration
- Plugins
- Terminal
- DAP

# Design overview

Efficient: simple yet powerful. Focused on Rust. Creative and experimental.

As a first iteration, we want to have a binary that is able to open a single file, provide you with a few basic editing commands and save that file back to disk. Ligatures, emojis and smooth scroll is a must have from day 1. We are close to that! Let's have syntax highlight there as well, since I already did that.

Then it would make sense to implement file picking and multiple buffers. Make it format on save and I would use it everyday 😍. Dogfood asap.

We can extend beyond that with Cargo and Git tasks for day to day productivity. There is everything to imagine in this area! More editing commands could be very welcome as well.

Final boss: LSP. Hope I'll have enough HP.

# Technical details

## Swash

I started this journey trying to render text with `swash`. Why swash? Swash is great. Swash lets me think I understand text rendering. Swash supports ligatures, font variations and emojis. To my knowledge, not a single Rust GUI toolkit supports ligatures. Fallbacks, BiDi, layout, what have you..., but all I want is ligatures.

So swash has **shaping** (going from UTF-8 to glyphs) and **scaling** (going from glyphs to pixels).

The hardest thing with shaping is that there is very little chance that your favorite font has both glyphs for 'a' and '🚀'. Swash will tell you when glyphs are missing for a font and you'll have to juggle with fonts accordingly. In `virus`, caller provides the "text" font and the shaper will look for missing glyphs in the "emoji" font. As simple as that!

Scaling goes through `zeno` (which we could also use for vector ornaments in the UI) and results in a list of pixels. We have to handle characters (alpha mask) and emojis (RGBA) separately. Since rasterisation is costly we cache those images. I'm not sure what the best cache strategy would be for `virus`, but for a single static font UI the current `(font, glyph, size)` seems fine. Remember we said no to Recursive and animated Noto! 🥲

## Winit/Pixels

Time for those delicious pixels to thrill my eyes! Right now the `main` code should not be very different from `pixels`' `minimal-winit` example. I'm afraid to dive deep in those two libs because I know I'll hit the wall fast. I'm just happy that things are working after days of graphic drivers updates! My only worry is compatibility: pixels is giving me RGBA `u32`s but if it gives you another format you might not enjoy `virus`.

One thing I have to investigate about `pixels` (`winit`?): my fps is low in debug even though I `opt-level = 3` my `[profile.dev.package."*"]`.

My next objective is to have some sort of keyboard handler and an mvp modal editing system. Stay tuned!

## Ropey

## Tree-sitter


