use serde::de::{self};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    mbid: String,
    #[serde(rename = "#text")]
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    #[serde(skip)]
    fields: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct TrackWrapper {
    pub track: Vec<Track>,
}

#[derive(Deserialize, Debug, Default)]
pub struct AlbumInfo {
    pub name: String,
    #[serde(skip)]
    artist: String,
    #[serde(skip)]
    mbid: String,
    #[serde(skip)]
    url: String,

    #[serde(rename = "image")]
    pub images: Vec<Image>,

    #[serde(skip)]
    listeners: String,
    #[serde(skip)]
    playcount: String,

    pub tracks: TrackWrapper,

    #[serde(skip)]
    wiki: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Image {
    #[serde(rename = "#text")]
    pub url: String,
    pub size: String,
}

#[derive(Deserialize, Debug)]
pub struct AlbumInfoWrapper {
    #[serde(rename = "album")]
    pub album_info: AlbumInfo,
}

#[derive(Deserialize, Debug)]
pub struct Album {
    pub artist: Artist,
    #[serde(rename = "@attr")]
    attributes: HashMap<String, String>,
    pub mbid: String,
    #[serde(deserialize_with = "from_str")]
    pub playcount: usize,
    pub name: String,
    url: String,
    #[serde(skip)]
    pub image_url: String,
    #[serde(skip)]
    pub tracks: usize,
}

#[derive(Deserialize, Debug)]
pub struct Chart {
    #[serde(rename = "album")]
    pub albums: Vec<Album>,
    #[serde(rename = "@attr", skip)]
    attributes: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct WeeklyAlbumChart {
    #[serde(rename = "weeklyalbumchart")]
    pub chart: Chart,
}
