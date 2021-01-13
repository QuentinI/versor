use std::any::Any;
use std::fmt::Display;
use url::{Url};
use chrono::{ DateTime, Utc, Duration };
use serde::{Serialize, Deserialize, Deserializer};
use anyhow::Result;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    category: String,
    id: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    tag_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumb {
    height: u32,
    width: u32,
    size: Thumbsize,
    src: Url,
}

fn from_duration<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
    where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    let nums = s.split(":").map(i64::from_str).collect::<Result<Vec<i64>, std::num::ParseIntError>>().map_err(serde::de::Error::custom)?;
    if nums.len() != 2 {
        return Err(serde::de::Error::invalid_length(nums.len(), &"2"));
    }
    Ok(Duration::minutes(nums[0]) + Duration::seconds(nums[1]))
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct Video {
    #[serde(deserialize_with = "from_str")]
    pub video_id: u32,
    pub views: u32,
    pub url: Url,
    pub title: String,
    pub thumb: Url,
    pub default_thumb: Url,
    pub thumbs: Vec<Thumb>,
    #[serde(deserialize_with = "from_duration")]
    pub duration: Duration,
    pub embed_url: Url,
    pub publish_date: String,
    #[serde(deserialize_with = "from_str")]
    pub rating: f64,
    pub ratings: u64,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Copy)]
enum Period {
    Weekly,
    Monthly,
    AllTime
}

impl Default for Period {
    fn default() -> Period {
        Period::AllTime
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let string = match self {
            Period::Weekly => "weekly",
            Period::Monthly => "monthly",
            Period::AllTime => "alltime"
        };
        write!(f, "{}", string)
    }
}

#[derive(Clone, Copy)]
enum Ordering {
    MostViewed,
    Newest,
    Rating
}

impl Default for Ordering {
    fn default() -> Ordering {
        Ordering::Rating
    }
}

impl Display for Ordering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let string = match self {
            Ordering::MostViewed => "mostviewed",
            Ordering::Newest => "newest",
            Ordering::Rating => "rating"
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")] 
enum Thumbsize {
    Medium,
    Small,
    Big,
    Medium1,
    Medium2,
    All
}

impl Default for Thumbsize {
    fn default() -> Thumbsize {
        Thumbsize::All
    }
}

impl Display for Thumbsize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let string = match self {
            Thumbsize::Medium => "medium",
            Thumbsize::Small => "small",
            Thumbsize::Big => "big",
            Thumbsize::Medium1 => "medium1",
            Thumbsize::Medium2 => "medium2",
            Thumbsize::All => "all"
        };
        write!(f, "{}", string)
    }
}

enum Error {
    Io
}

#[derive(Default)]
pub struct SearchVideo {
    category: Option<String>,
    page: Option<u32>,
    search: Option<String>,
    tags: Option<Vec<String>>,
    stars: Option<Vec<String>>,
    ordering: Option<Ordering>,
    thumbsize: Option<Thumbsize>,
    period: Option<Period>
}

#[derive(Debug, Deserialize)]
struct SearchVideoResultItem {
    video: Video
}

#[derive(Debug, Deserialize)]
struct SearchVideoResult {
    count: u64,
    videos: Vec<SearchVideoResultItem>,
}

impl SearchVideo {
    pub async fn execute(&self) -> Result<Vec<Video>> {
        let url = format!(
            "{base}&output=json&search={search}&page={page}&tags[]={tags}&thumbsize={thumbsize}&category={category}&stars={stars}{ordering}&period={period}",
            base = "https://api.redtube.com/?data=redtube.Videos.searchVideos",
            search = self.search.clone().unwrap_or_default(),
            page = self.page.unwrap_or(1),
            tags = self.tags.clone().unwrap_or_default().join(","),
            thumbsize = self.thumbsize.unwrap_or_default(),
            category = self.category.clone().unwrap_or_default(),
            stars = self.stars.clone().unwrap_or_default().join(","),
            ordering = match self.ordering {
                Some(ord) => format!("&ordering={}", ord),
                None => String::new()
            },
            period = self.period.unwrap_or_default()
        );
        let response = reqwest::get(&url).await?.text().await?;
        let videos: SearchVideoResult = serde_json::from_str(&response)?;
        Ok(videos.videos.into_iter().map(|it| it.video).collect())
    }

    pub fn search(self, term: &str) -> Self {
        Self {
            search: Some(term.to_string()),
            .. self
        }
    }
}
