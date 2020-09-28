use bytes::Bytes;
use futures::{stream, StreamExt};
use image::{imageops::FilterType, load_from_memory, GenericImage, DynamicImage, Rgba, RgbaImage};
use imageproc::{drawing, rect::Rect};
use reqwest::{Client, Error};
use rusttype::{Font, Scale};
use std::{
    fs::File,
    io::prelude::*,
    env::args,
    collections::HashMap,
    convert::TryInto,
    sync::{Arc, Mutex},
};
use types::*;

mod types;

const CONCURRENT_REQUESTS: usize = 100;
const IMAGE_LENGTH: u32 = 300;
const MARGIN: u32 = 50;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let user = args().nth(1).unwrap_or("usefulalgorithm".to_string());
    let url = "http://ws.audioscrobbler.com/2.0/";
    let mut key_file = File::open("key.env").unwrap();
    let mut api_key = String::new();
    key_file.read_to_string(&mut api_key).unwrap();
    let api_key = api_key.as_str();
    let client = Client::new();
    let chart = client
        .get(url)
        .query(&[
            ("method", "user.getweeklyalbumchart"),
            ("user", &user),
            ("api_key", &api_key),
            ("format", "json"),
        ])
        .send()
        .await?
        .json::<WeeklyAlbumChart>()
        .await?
        .chart;

    let info_wrappers = stream::iter(&chart.albums)
        .map(|album| {
            let client = &client;
            async move {
                let album_info = client
                    .get(url)
                    .query(&[
                        ("method", "album.getinfo"),
                        ("artist", &album.artist.name),
                        ("album", &album.name),
                        ("api_key", &api_key),
                        ("format", "json"),
                    ])
                    .send()
                    .await?
                    .json::<AlbumInfoWrapper>()
                    .await?
                    .album_info;
                let bytes = match album_info
                    .images
                    .iter()
                    .filter(|i| i.size.is_empty() && !i.url.is_empty())
                    .next()
                {
                    Some(i) => client.get(&i.url).send().await?.bytes().await?,
                    None => Bytes::new(),
                };
                Ok::<(String, (usize, bytes::Bytes)), Error>((
                    album.name.to_owned(),
                    (album_info.tracks.track.len(), bytes),
                ))
            }
        })
        .buffered(CONCURRENT_REQUESTS);
    let album_infos = Arc::new(Mutex::new(HashMap::new()));
    info_wrappers
        .for_each(|i| async {
            match i {
                Ok(i) => {
                    let mut album_infos = album_infos.lock().unwrap();
                    album_infos.insert(i.0, i.1);
                }
                Err(e) => eprintln!("Uh oh: {}", e),
            }
        })
        .await;
    let size: u32 = (1..100)
        .take_while(|&x| x * x <= chart.albums.len())
        .last()
        .unwrap()
        .try_into()
        .unwrap();

    let word_margin = 2u32;
    let word_height = (IMAGE_LENGTH * size - word_margin * (chart.albums.len() as u32 - 1u32)) / (chart.albums.len() as u32);
    let word_length = word_height * chart.albums.iter().map(|a| a.name.len() as u32).max().unwrap();
    let height = 2 * MARGIN + IMAGE_LENGTH * size;
    let width = height + MARGIN + word_length;
    let mut canvas = RgbaImage::new(width, height);
    let rect = Rect::at(0, 0).of_size(width, height);
    let white = Rgba([255u8, 255u8, 255u8, 255u8]);
    let black = Rgba([0u8, 0u8, 0u8, 255u8]);
    let font_data = include_bytes!("../fonts/WenQuanYiMicroHei.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).unwrap();
    drawing::draw_filled_rect_mut(&mut canvas, rect, black);


    for (i, album) in chart.albums.iter().enumerate() {
        let i = i as u32;
        let info = album_infos.lock().unwrap();
        let info = info.get(&album.name).unwrap();
        if i < size * size {
            let image = load_from_memory(&info.1).unwrap_or(DynamicImage::new_rgba8(300, 300));
            image.resize(IMAGE_LENGTH, IMAGE_LENGTH, FilterType::Triangle);
            let x = IMAGE_LENGTH * (i % size);
            let y = IMAGE_LENGTH * (i / size);
            canvas.copy_from(&image, MARGIN + x, MARGIN + y).unwrap();
        }
        // TODO don't show playcount and trackcount for now
        let s = format!(
            "{}. {} - {}",
            i + 1,
            album.artist.name,
            album.name,
        );
        // let s = format!(
        //     "{}. {} - {} ({}/{})",
        //     i + 1,
        //     album.artist.name,
        //     album.name,
        //     album.playcount,
        //     info.0
        // );
        println!("{}", s);
        drawing::draw_text_mut(
            &mut canvas,
            white,
            2 * MARGIN + IMAGE_LENGTH * size,
            MARGIN + i * (word_height + word_margin),
            Scale::uniform(word_height as f32),
            &font,
            &s,
        );
    }

    canvas.save("output.png").unwrap();
    Ok(())
}
