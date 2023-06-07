use futures::{prelude::stream::FuturesUnordered, StreamExt};
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, Frame};
use reqwest::*;

const OUTPUT: &str = "/home/romain/Desktop/noto_animated_emoji";
const SIZE: u32 = 64;
const FILTER_TYPE: FilterType = FilterType::CatmullRom;
const SPEED: i32 = 30; // 1..=30 (hq..lq)

#[tokio::main]
async fn main() {
    dbg!(EMOJIS.len());

    std::fs::create_dir_all(OUTPUT).unwrap();
    std::fs::create_dir_all(format!("{OUTPUT}/{SIZE}/")).unwrap();

    let client = Client::new();

    EMOJIS
        .into_iter()
        .map(|emoji| {
            let client = client.clone();
            async move {
                println!("\x1B[0;34mDownloading {emoji}\x1B[0m");
                let downloaded = download(&client, &emoji).await;

                println!("\x1B[0;35mResize {emoji}\x1B[0m");
                let gif = encode(resize(&downloaded));

                println!("\x1B[0;32mSave {emoji}\x1B[0m");
                std::fs::write(format!("{OUTPUT}/{emoji}.gif"), &gif).unwrap();
            }
        })
        .collect::<FuturesUnordered<_>>()
        .count()
        .await;
}

async fn download(client: &Client, emoji: &str) -> Vec<u8> {
    client
        .get(format!(
            "https://fonts.gstatic.com/s/e/notoemoji/latest/{emoji}/512.gif"
        ))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .into()
}

fn resize(bytes: &[u8]) -> impl '_ + Iterator<Item = Frame> {
    GifDecoder::new(bytes)
        .unwrap()
        .into_frames()
        .map(move |frame| {
            let frame = frame.unwrap();
            let (top, left, delay) = (frame.top(), frame.left(), frame.delay());
            assert!(top == 0);
            assert!(left == 0);

            let image = DynamicImage::from(frame.into_buffer());
            assert!(image.width() == 512);
            assert!(image.height() == 512);

            let image = image.resize(SIZE, SIZE, FILTER_TYPE);
            assert!(image.width() == SIZE);
            assert!(image.height() == SIZE);

            Frame::from_parts(image.to_rgba8(), left, top, delay)
        })
}

fn encode(frames: impl Iterator<Item = Frame>) -> Vec<u8> {
    let mut gif = Vec::new();
    let mut encoder = GifEncoder::new_with_speed(&mut gif, SPEED);
    encoder.set_repeat(Repeat::Infinite).unwrap();
    encoder.encode_frames(frames).unwrap();
    drop(encoder);

    gif
}

// https://googlefonts.github.io/noto-emoji-animation/
// Scroll down the whole page, and `Array.from(document.querySelectorAll(".icon-asset")).map(el => el.src).toString()`
// https://fonts.gstatic.com/s/e/notoemoji/latest/{emoji}/512.gif
const EMOJIS: &[&str] = &[
    "1f600",
    "1f603",
    "1f604",
    "1f601",
    "1f606",
    "1f605",
    "1f602",
    "1f923",
    "1f62d",
    "1f609",
    "1f617",
    "1f619",
    "1f61a",
    "1f618",
    "1f970",
    "1f60d",
    "1f929",
    "1f973",
    "1fae0",
    "1f643",
    "1f642",
    "1f972",
    "1f979",
    "1f60a",
    "263a_fe0f",
    "1f60c",
    "1f60f",
    "1f634",
    "1f62a",
    "1f924",
    "1f60b",
    "1f61b",
    "1f61d",
    "1f61c",
    "1f92a",
    "1f974",
    "1f614",
    "1f97a",
    "1f62c",
    "1f611",
    "1f610",
    "1f636",
    "1f636_200d_1f32b_fe0f",
    "1fae5",
    "1f910",
    "1fae1",
    "1f914",
    "1f92b",
    "1fae2",
    "1f92d",
    "1f971",
    "1f917",
    "1fae3",
    "1f631",
    "1f928",
    "1f9d0",
    "1f612",
    "1f644",
    "1f62e_200d_1f4a8",
    "1f624",
    "1f620",
    "1f621",
    "1f92c",
    "1f61e",
    "1f613",
    "1f61f",
    "1f625",
    "1f622",
    "2639_fe0f",
    "1f641",
    "1fae4",
    "1f615",
    "1f630",
    "1f628",
    "1f627",
    "1f626",
    "1f62e",
    "1f62f",
    "1f632",
    "1f633",
    "1f92f",
    "1f616",
    "1f623",
    "1f629",
    "1f62b",
    "1f635",
    "1f635_200d_1f4ab",
    "1fae8",
    "1f976",
    "1f975",
    "1f922",
    "1f92e",
    "1f927",
    "1f912",
    "1f915",
    "1f637",
    "1f925",
    "1f607",
    "1f920",
    "1f911",
    "1f913",
    "1f60e",
    "1f978",
    "1f921",
    "1f608",
    "1f47f",
    "1f47b",
    "1f383",
    "1f4a9",
    "1f916",
    "1f47d",
    "1f31b",
    "1f31c",
    "1f31e",
    "1f525",
    "1f4af",
    "1f31f",
    "2728",
    "1f4a5",
    "1f389",
    "1f648",
    "1f649",
    "1f64a",
    "1f63a",
    "1f638",
    "1f639",
    "1f63b",
    "1f63c",
    "1f63d",
    "1f640",
    "1f63f",
    "1f63e",
    "2764_fe0f",
    "1f9e1",
    "1f49b",
    "1f49a",
    "1fa75",
    "1f499",
    "1f49c",
    "1f90e",
    "1f5a4",
    "1fa76",
    "1f90d",
    "1fa77",
    "1f498",
    "1f49d",
    "1f496",
    "1f497",
    "1f493",
    "1f49e",
    "1f495",
    "1f48c",
    "2763_fe0f",
    "2764_fe0f_200d_1fa79",
    "1f494",
    "2764_fe0f_200d_1f525",
    "1f48b",
    "1f463",
    "1fac0",
    "1fa78",
    "1f9a0",
    "1f480",
    "1f440",
    "1f441_fe0f",
    "1fae6",
    "1f9bf",
    "1f9be",
    "1f4aa",
    "1f4aa_1f3fb",
    "1f4aa_1f3fc",
    "1f4aa_1f3fd",
    "1f4aa_1f3fe",
    "1f4aa_1f3ff",
    "1f44f",
    "1f44f_1f3fb",
    "1f44f_1f3fc",
    "1f44f_1f3fd",
    "1f44f_1f3fe",
    "1f44f_1f3ff",
    "1f44d",
    "1f44d_1f3fb",
    "1f44d_1f3fc",
    "1f44d_1f3fd",
    "1f44d_1f3fe",
    "1f44d_1f3ff",
    "1f44e",
    "1f44e_1f3fb",
    "1f44e_1f3fc",
    "1f44e_1f3fd",
    "1f44e_1f3fe",
    "1f44e_1f3ff",
    "1f64c",
    "1f64c_1f3fb",
    "1f64c_1f3fc",
    "1f64c_1f3fd",
    "1f64c_1f3fe",
    "1f64c_1f3ff",
    "1f44b",
    "1f44b_1f3fb",
    "1f44b_1f3fc",
    "1f44b_1f3fd",
    "1f44b_1f3fe",
    "1f44b_1f3ff",
    "270c_fe0f",
    "270c_1f3fb",
    "270c_1f3fc",
    "270c_1f3fd",
    "270c_1f3fe",
    "270c_1f3ff",
    "1f91e",
    "1f91e_1f3fb",
    "1f91e_1f3fc",
    "1f91e_1f3fd",
    "1f91e_1f3fe",
    "1f91e_1f3ff",
    "261d_fe0f",
    "261d_1f3fb",
    "261d_1f3fc",
    "261d_1f3fd",
    "261d_1f3fe",
    "261d_1f3ff",
    "1f64f",
    "1f64f_1f3fb",
    "1f64f_1f3fc",
    "1f64f_1f3fd",
    "1f64f_1f3fe",
    "1f64f_1f3ff",
    "1f483",
    "1f483",
    "1f483_1f3fb",
    "1f483_1f3fc",
    "1f483_1f3fd",
    "1f483_1f3fe",
    "1f483_1f3ff",
    "1f339",
    "1f339",
    "1f940",
    "1f342",
    "1f331",
    "1f340",
    "2744_fe0f",
    "1f30b",
    "1f305",
    "1f304",
    "1f308",
    "1f32c_fe0f",
    "26a1",
    "1f4ab",
    "2604_fe0f",
    "1f30d",
    "1f984",
    "1f98e",
    "1f409",
    "1f996",
    "1f422",
    "1f40d",
    "1f438",
    "1f407",
    "1f400",
    "1f415",
    "1f416",
    "1f40e",
    "1facf",
    "1f402",
    "1f410",
    "1f998",
    "1f405",
    "1f412",
    "1f43f_fe0f",
    "1f9a6",
    "1f987",
    "1f413",
    "1f423",
    "1f424",
    "1f425",
    "1f985",
    "1f54a_fe0f",
    "1fabf",
    "1f99a",
    "1f9ad",
    "1f42c",
    "1f433",
    "1f421",
    "1f980",
    "1f419",
    "1fabc",
    "1f40c",
    "1f41c",
    "1f99f",
    "1f41d",
    "1f98b",
    "1f43e",
    "1f345",
    "1f345",
    "1f37f",
    "2615",
    "1f37b",
    "1f942",
    "1f37e",
    "1f377",
    "1f379",
    "1f6a8",
    "1f6a8",
    "1f6f8",
    "1f680",
    "1f6eb",
    "1f6ec",
    "1f3a2",
    "1f38a",
    "1f38a",
    "1f388",
    "1f382",
    "1f386",
    "1faa9",
    "26bd",
    "1f3af",
    "1f3bb",
    "1f941",
    "1fa87",
    "1f50b",
    "1f50b",
    "1faab",
    "1f4b8",
    "1f4a1",
    "1f393",
    "2602_fe0f",
    "1f48e",
    "23f0",
    "1f6ce_fe0f",
    "1f514",
    "2648",
    "2648",
    "2649",
    "264a",
    "264b",
    "264c",
    "264d",
    "264e",
    "264f",
    "2650",
    "2651",
    "2652",
    "2653",
    "26ce",
    "203c_fe0f",
    "274c",
    "1f3b6",
    "2705",
    "1f192",
    "2795",
    "1f3c1",
    "1f3c1",
];
