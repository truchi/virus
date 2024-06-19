use editor::fuzzy::{Config, Fuzzy};
use std::io::{stdin, stdout, Write};

const HAYSTACKS: &[&str] = &[
    "crates/editor/Cargo.toml",
    "crates/editor/examples/fuzzy.rs",
    "crates/editor/examples/nucleo.rs",
    "crates/editor/src/cursor.rs",
    "crates/editor/src/document.rs",
    "crates/editor/src/fuzzy.rs",
    "crates/editor/src/lib.rs",
    "crates/editor/src/rope/cursor/chunk.rs",
    "crates/editor/src/rope/cursor/grapheme.rs",
    "crates/editor/src/rope/cursor/word.rs",
    "crates/editor/src/rope/extension.rs",
    "crates/editor/src/syntax/capture.rs",
    "crates/editor/src/syntax/theme.rs",
    "crates/editor/treesitter/rust/highlights.scm",
    "crates/graphics/Cargo.toml",
    "crates/graphics/examples/inspect_font.rs",
    "crates/graphics/src/lib.rs",
    "crates/graphics/src/muck.rs",
    "crates/graphics/src/text/font.rs",
    "crates/graphics/src/text/line.rs",
    "crates/graphics/src/text/mod.rs",
    "crates/graphics/src/types/color.rs",
    "crates/graphics/src/types/geom.rs",
    "crates/graphics/src/wgpu/atlas.rs",
    "crates/graphics/src/wgpu/glyph.rs",
    "crates/graphics/src/wgpu/glyph.wgsl",
    "crates/graphics/src/wgpu/line.rs",
    "crates/graphics/src/wgpu/line.wgsl",
    "crates/graphics/src/wgpu/mod.rs",
    "crates/graphics/src/wgpu/rectangle.rs",
    "crates/graphics/src/wgpu/rectangle.wgsl",
    "crates/ui/Cargo.toml",
    "crates/ui/src/lib.rs",
    "crates/ui/src/theme.rs",
    "crates/ui/src/tween.rs",
    "crates/ui/src/ui.rs",
    "crates/ui/src/views/document.rs",
    "crates/ui/src/views/files.rs",
    "crates/virus/Cargo.toml",
    "crates/virus/src/events.rs",
    "crates/virus/src/fps.rs",
    "crates/virus/src/main.rs",
    "crates/virus/src/virus.rs",
    "Cargo.lock",
    "Cargo.toml",
    "README.md",
    "rust-toolchain.toml",
];

fn main() {
    let config = Config {
        hits_bonus: 10,
        accumulated_hits_bonus: 1,
        accumulated_hits_bonus_limit: 10,
        needle_start_bonus: 1,
        haystack_start_bonus: 1,
        uppercase_bonus: 1,
        space_as_separator_malus: 1,
    };
    dbg!(config);

    let mut input = String::new();

    loop {
        input.clear();
        print!("> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        input = input.trim_end_matches('\n').to_string();

        if !input.is_empty() {
            score(config, &input);
        }
    }
}

fn score(config: Config, needle: &str) {
    let mut scored = Vec::new();
    let mut fuzzy = Fuzzy::new(config, needle);

    for haystack in HAYSTACKS {
        if let Some((score, matches)) = fuzzy.score(haystack) {
            scored.push((haystack, score, matches.to_owned()));
        }
    }

    if scored.is_empty() {
        println!("No matches :(");
        return;
    }

    scored.sort_by_key(|(_, score, _)| -score);

    let max = scored.first().map(|(_, score, _)| *score).unwrap();
    let min = scored.last().map(|(_, score, _)| *score).unwrap();

    for (str, score, matches) in scored {
        print!("{score:>5} ");

        let red = if max == min {
            255
        } else {
            let factor = (score as f32 - min as f32) / (max as f32 - min as f32);
            (128.0 + (factor * 127.0)) as u8
        };

        let mut last_index = 0;

        for range in matches {
            if range.start > last_index {
                print!("{}", &str[last_index..range.start]);
            }

            print!(
                "\x1b[38;2;{red};0;0m{}\x1b[0m",
                &str[range.start..range.end],
            );
            last_index = range.end;
        }

        if last_index < str.len() {
            print!("{}", &str[last_index..]);
        }

        println!();
    }
}
