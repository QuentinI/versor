use std::any::Any;
use std::fmt::Display;
use std::marker::PhantomData;
use url::{Url};
use chrono::{ DateTime, Utc, Duration };
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{Visitor, IntoDeserializer};
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

fn from_str_or_float<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr,
        T::Err: Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            T::from_str(&value).map_err(serde::de::Error::custom)
        }

        fn visit_f64<E>(self, value: f64) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            Deserialize::deserialize(value.into_deserializer())
        }

        fn visit_i64<E>(self, value: i64) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            Deserialize::deserialize(value.into_deserializer())
        }

        fn visit_u64<E>(self, value: u64) -> Result<T, E>
        where
            E: serde::de::Error,
        {
            Deserialize::deserialize(value.into_deserializer())
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
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
    #[serde(deserialize_with = "from_str_or_float")]
    pub rating: f64,
    pub ratings: u64,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] 
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

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] 
enum Ordering {
    MostViewed,
    Newest,
    Rating
}

impl Default for Ordering {
    fn default() -> Ordering {
        Ordering::MostViewed
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
        let page = self.page.unwrap_or(1);
        let tags = self.tags.clone().unwrap_or_default().join(",");
        let thumbsize = self.thumbsize.unwrap_or_default();
        let stars = self.stars.clone().unwrap_or_default().join(",");

        let client = reqwest::Client::new();
        let mut req = client.get("https://api.redtube.com/?data=redtube.Videos.searchVideos&output=json");

        if let Some(page) = self.page {
            req = req.query(&[("page", page)]);
        }

        if let Some(search) = &self.search {
            req = req.query(&[("search", search)]);
        }

        if let Some(tags) = &self.tags {
            req = req.query(&[("tags", tags.join(","))]);
        }

        if let Some(thumbsize) = self.thumbsize {
            req = req.query(&[("thumbsize", thumbsize)]);
        }

        if let Some(stars) = &self.stars {
            req = req.query(&[("stars", stars.join(","))]);
        }

        if let Some(ordering) = self.ordering {
            req = req.query(&[("ordering", ordering)]);
        }

        if let Some(period) = self.period {
            req = req.query(&[("period", period)]);
        }

        let req = req.build()?;

        trace!("Requesting video {}", req.url());

        let response = client.execute(req).await?.text().await?;
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
