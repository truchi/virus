use futures::{stream::FuturesUnordered, StreamExt};
use image::{
    codecs::{
        gif::GifDecoder,
        png::{self, PngEncoder},
    },
    imageops::FilterType,
    AnimationDecoder, ColorType, DynamicImage, Frame, ImageEncoder,
};
use rayon::prelude::*;
use reqwest::*;
use std::{collections::HashSet, io::Write, time::Instant};

const ENDPOINT: &str = "https://fonts.gstatic.com/s/e/notoemoji/latest";
const SIZE: u32 = 512;
const RESIZE: u32 = 64;
const OUTPUT: &str = "/home/romain/Desktop/noto_animated_emoji";
const COMPRESSION: png::CompressionType = png::CompressionType::Best;
const PNG_FILTER: png::FilterType = png::FilterType::Adaptive;
const RESIZE_FILTER: FilterType = FilterType::CatmullRom;
const CHUNK: usize = 30;

#[derive(Default)]
struct Main {
    client: Client,
    gifs: Vec<(String, Vec<Frame>)>,
}

impl Main {
    async fn download(&mut self, emojis: &[&str]) {
        let mut stream = emojis
            .iter()
            .map(|emoji| {
                let client = self.client.clone();
                async move { (emoji.to_string(), download(&client, &emoji).await) }
            })
            .collect::<FuturesUnordered<_>>();

        while let Some((emoji, gif)) = stream.next().await {
            self.gifs.push((emoji, decode_gif(&gif).collect()));
        }
    }

    fn write(self) {
        self.gifs
            .into_par_iter()
            .flat_map(|(emoji, frames)| {
                std::fs::create_dir_all(format!("{OUTPUT}/{emoji}")).unwrap();
                frames
                    .into_par_iter()
                    .enumerate()
                    .map(move |(i, frame)| (emoji.clone(), i, frame))
            })
            .map(|(emoji, i, frame)| {
                let delay = frame.delay();
                let (numer, denom) = delay.numer_denom_ms();
                let name = format!("frame:{i}-numer:{numer}-denom:{denom}");
                let frame = resize_frame(frame, RESIZE, RESIZE_FILTER);
                let png = encode_png(&frame);

                std::fs::write(format!("{OUTPUT}/{emoji}/{name}.png"), &png).unwrap();
            })
            .count();
    }
}

// (https://github.com/ImageOptim/gifski)?
#[tokio::main]
async fn main() {
    std::fs::create_dir_all(OUTPUT).unwrap();

    let emojis = HashSet::<&str>::from_iter(EMOJIS.into_iter().filter_map(|emoji| {
        const SKIN_PREFIX: &str = "_1f3f";
        const HAIR_PREFIX: &str = "_1f9b";
        const NON_RED_HEARTS: &[&str] = &[
            "u1f9e1", "u1f49b", "u1f49a", "u1fa75", "u1f499", "u1f49c", "u1f90e", "u1f5a4",
            "u1fa76", "u1f90d", "u1fa77",
        ];

        if NON_RED_HEARTS.contains(emoji)
            || emoji.contains(HAIR_PREFIX)
            || emoji.contains(SKIN_PREFIX)
        {
            None
        } else {
            Some(*emoji)
        }
    }))
    .into_iter()
    .collect::<Vec<_>>();

    let chunks = (emojis.len() as f32 / CHUNK as f32).ceil() as u32;
    println!(
        "{} emojis, chunk size: {CHUNK}, chunks: {}",
        emojis.len(),
        chunks
    );

    let start = Instant::now();

    for (i, emojis) in emojis.chunks(CHUNK).enumerate() {
        let i = i as u32 + 1;
        let mut main = Main::default();

        print!("{}/{chunks}: downloading & decoding", i);
        std::io::stdout().flush().unwrap();

        let now = Instant::now();
        main.download(emojis).await;

        print!(" ({:?}), writing", now.elapsed());
        std::io::stdout().flush().unwrap();

        let now = Instant::now();
        main.write();

        println!(
            " ({:?}), {:?} left",
            now.elapsed(),
            (start.elapsed() / i) * (chunks - i),
        );
    }

    println!("Took {:?}", start.elapsed());
}

async fn download(client: &Client, emoji: &str) -> Vec<u8> {
    client
        .get(format!("{ENDPOINT}/{emoji}/{SIZE}.gif"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .into()
}

fn decode_gif(bytes: &[u8]) -> impl '_ + Iterator<Item = Frame> {
    GifDecoder::new(bytes).unwrap().into_frames().map(|frame| {
        let frame = frame.unwrap();
        debug_assert!(frame.top() == 0);
        debug_assert!(frame.left() == 0);
        debug_assert!(frame.buffer().width() == SIZE);
        debug_assert!(frame.buffer().height() == SIZE);

        frame
    })
}

fn resize_frame(frame: Frame, size: u32, filter: FilterType) -> Frame {
    let (top, left, delay, buffer) = (
        frame.top(),
        frame.left(),
        frame.delay(),
        frame.into_buffer(),
    );

    Frame::from_parts(
        DynamicImage::from(buffer)
            .resize(size, size, filter)
            .into_rgba8(),
        left,
        top,
        delay,
    )
}

fn encode_png(frame: &Frame) -> Vec<u8> {
    let buffer = frame.buffer();

    let mut png = Vec::new();
    PngEncoder::new_with_quality(&mut png, COMPRESSION, PNG_FILTER)
        .write_image(buffer, buffer.width(), buffer.height(), ColorType::Rgba8)
        .unwrap();

    png
}

// https://googlefonts.github.io/noto-emoji-animation/
// Scroll down the whole page, and `Array.from(document.querySelectorAll(".icon-asset")).map(el => el.src).toString()`
// https://fonts.gstatic.com/s/e/notoemoji/latest/{emoji}/512.gif
const EMOJIS: &[&str] = &[
    "1f62f",
    "1f649",
    "1f64c",
    "1f975",
    "1f976",
    "2650",
    "2651",
    "1f49a",
    "1f47f",
    "270c_fe0f",
    "1f30b",
    "1f630",
    "1f424",
    "1f421",
    "1f985",
    "1fae5",
    "261d_1f3fc",
    "1f40d",
    "1f928",
    "1f63d",
    "2604_fe0f",
    "1f600",
    "1f90e",
    "1f4aa_1f3fe",
    "261d_1f3fd",
    "1f433",
    "1f3af",
    "1f3bb",
    "1f91e_1f3fd",
    "1f609",
    "1f192",
    "1f495",
    "1f402",
    "261d_1f3fb",
    "1f4aa_1f3fb",
    "270c_1f3ff",
    "1f483_1f3fd",
    "1f415",
    "2728",
    "1f60b",
    "1f9a6",
    "1f4a1",
    "1f62e",
    "1f929",
    "2795",
    "1f974",
    "1f925",
    "270c_1f3fb",
    "1f48c",
    "270c_1f3fe",
    "1f64c_1f3fe",
    "1f483_1f3fb",
    "2639_fe0f",
    "1f405",
    "1f54a_fe0f",
    "1f910",
    "1f622",
    "2763_fe0f",
    "1f90d",
    "261d_fe0f",
    "1f483_1f3fe",
    "1f620",
    "1f61f",
    "1f60e",
    "1f48e",
    "1f605",
    "1fabc",
    "1fae2",
    "1f379",
    "1f62b",
    "1f914",
    "1f613",
    "1f44d_1f3fe",
    "1f924",
    "1f44e_1f3fb",
    "1f44b_1f3fb",
    "1f64f_1f3fe",
    "1f41d",
    "1f92a",
    "1f305",
    "1f602",
    "1f62d",
    "1f44b",
    "1f9ad",
    "1f62c",
    "1f639",
    "1f44e_1f3fc",
    "1f998",
    "1f64c_1f3ff",
    "1f615",
    "2649",
    "2705",
    "1f638",
    "1f480",
    "1f44f_1f3fd",
    "1f44e_1f3fd",
    "1f6eb",
    "1f44d_1f3fd",
    "263a_fe0f",
    "1f63c",
    "1f91e_1f3fc",
    "1f345",
    "1f389",
    "1f923",
    "1fa75",
    "1f40e",
    "1f603",
    "1fa76",
    "1f49d",
    "1f308",
    "1f61b",
    "1fa87",
    "1f514",
    "1f634",
    "1f9d0",
    "1f44f_1f3fe",
    "1f44d_1f3fb",
    "1f44d_1f3fc",
    "1f921",
    "1f44e_1f3fe",
    "1f44e_1f3ff",
    "1f44b_1f3fd",
    "1f91e",
    "1faa9",
    "1f99f",
    "1f916",
    "261d_1f3fe",
    "1f978",
    "1f304",
    "1f98b",
    "1f63b",
    "1f493",
    "1f60f",
    "1faab",
    "1f31f",
    "1f616",
    "1f44e",
    "1f393",
    "1f920",
    "1f377",
    "1f98e",
    "1f31c",
    "1f92c",
    "1f632",
    "1f5a4",
    "1f636_200d_1f32b_fe0f",
    "1f913",
    "1f60c",
    "1f9be",
    "2602_fe0f",
    "264a",
    "1f48b",
    "1f4aa_1f3fd",
    "1f996",
    "1f635_200d_1f4ab",
    "1f49c",
    "1f644",
    "1f438",
    "1f987",
    "1facf",
    "1f43e",
    "1f624",
    "26bd",
    "1f911",
    "1f4b8",
    "264f",
    "1f625",
    "1f627",
    "1f463",
    "1f44b_1f3fc",
    "1f3b6",
    "1f601",
    "1f407",
    "1f410",
    "1f607",
    "1f4aa",
    "1fae8",
    "1f49e",
    "1f382",
    "1f612",
    "1f640",
    "264d",
    "1f628",
    "1f44b_1f3fe",
    "1f388",
    "1f91e_1f3fe",
    "1f422",
    "1f63e",
    "1f643",
    "1f4aa_1f3ff",
    "1f614",
    "1f636",
    "1f641",
    "1f483_1f3fc",
    "1f525",
    "1f498",
    "1f483",
    "1f972",
    "1f635",
    "1f9a0",
    "1f61e",
    "1f44f_1f3fb",
    "1f940",
    "1f61c",
    "1fabf",
    "1f980",
    "1f41c",
    "1f6a8",
    "1f619",
    "2652",
    "203c_fe0f",
    "1f340",
    "1f3c1",
    "1f4ab",
    "1f64c_1f3fb",
    "1f6ce_fe0f",
    "1f63f",
    "1f494",
    "1f62e_200d_1f4a8",
    "1f606",
    "1f64f_1f3fc",
    "1f60a",
    "1f608",
    "1f64f_1f3fd",
    "1f942",
    "23f0",
    "1fae3",
    "1f331",
    "1f37f",
    "1f61d",
    "1f441_fe0f",
    "1f31e",
    "1f499",
    "2744_fe0f",
    "1f4af",
    "1f970",
    "1f623",
    "1f915",
    "1f31b",
    "1f400",
    "1f917",
    "1f64f_1f3ff",
    "1f984",
    "1f496",
    "2764_fe0f",
    "1f383",
    "1fa77",
    "1f38a",
    "1fae0",
    "1f44b_1f3ff",
    "1f621",
    "1f64a",
    "2764_fe0f_200d_1fa79",
    "1f413",
    "1f642",
    "1f610",
    "1f973",
    "264b",
    "1f47b",
    "1fac0",
    "26ce",
    "274c",
    "1f423",
    "1f92e",
    "1f97a",
    "1f629",
    "1f62a",
    "1f927",
    "1f64c_1f3fc",
    "1f61a",
    "1f971",
    "264e",
    "1f617",
    "1f6ec",
    "1f483_1f3ff",
    "1f922",
    "1f342",
    "1f4aa_1f3fc",
    "1f633",
    "261d_1f3ff",
    "2653",
    "1f648",
    "1f49b",
    "1f44d",
    "1f497",
    "1f64f",
    "1f37e",
    "1f941",
    "1f43f_fe0f",
    "264c",
    "1f9bf",
    "1f4a5",
    "1f631",
    "270c_1f3fc",
    "1f64f_1f3fb",
    "1f40c",
    "1f425",
    "1f637",
    "1f44d_1f3ff",
    "1fae1",
    "1fae4",
    "1f92f",
    "1f91e_1f3fb",
    "2648",
    "1f604",
    "1f44f_1f3fc",
    "1f912",
    "1f92d",
    "1f4a9",
    "1f44f_1f3ff",
    "1f611",
    "1f386",
    "1f339",
    "1f32c_fe0f",
    "1f3a2",
    "1f419",
    "1f618",
    "1fa78",
    "1f409",
    "1f64c_1f3fd",
    "2615",
    "1f37b",
    "1f47d",
    "1f680",
    "1f412",
    "1f626",
    "1fae6",
    "1f416",
    "1f99a",
    "1f44f",
    "1f91e_1f3ff",
    "2764_fe0f_200d_1f525",
    "1f63a",
    "1f92b",
    "270c_1f3fd",
    "1f9e1",
    "1f42c",
    "26a1",
    "1f30d",
    "1f50b",
    "1f979",
    "1f60d",
    "1f440",
    "1f6f8",
];
