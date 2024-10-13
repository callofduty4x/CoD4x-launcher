use crate::launcher::http;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::time::Duration;

pub struct AssetInformation {
    pub name: String,
    pub url: String,
}

pub struct ReleaseInformation {
    pub tag_name: String,
    pub assets: Vec<AssetInformation>,
}

pub fn fetch_release_information(repository_path: &str) -> Result<ReleaseInformation, ParseError> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        repository_path
    );
    let response = http::download_str(url.as_str(), Some(Duration::from_secs(3)))
        .map_err(|_| ParseError::FetchError)?;
    let response_json: json::Value =
        json::from_str(response.as_str()).map_err(|_| ParseError::InvalidResponse)?;

    let tag_name = response_json
        .pointer("/tag_name")
        .ok_or(ParseError::TagName)?
        .as_str()
        .ok_or(ParseError::TagName)?;

    let release_assets_json = response_json
        .pointer("/assets")
        .ok_or(ParseError::ReleaseAssets)?
        .as_array()
        .ok_or(ParseError::ReleaseAssets)?;

    let mut assets = Vec::<AssetInformation>::new();
    for asset_json in release_assets_json {
        let asset_name = asset_json
            .pointer("/name")
            .ok_or(ParseError::ReleaseAssets)?
            .as_str()
            .ok_or(ParseError::ReleaseAssets)?;

        let asset_url = asset_json
            .pointer("/browser_download_url")
            .ok_or(ParseError::ReleaseAssets)?
            .as_str()
            .ok_or(ParseError::ReleaseAssets)?;

        assets.push(AssetInformation {
            name: asset_name.to_string(),
            url: asset_url.to_string(),
        });
    }

    Ok(ReleaseInformation {
        tag_name: tag_name.to_string(),
        assets,
    })
}

pub fn find_asset<'a>(
    release_info: &'a ReleaseInformation,
    pattern: &str,
) -> Option<&'a AssetInformation> {
    let regex = Regex::new(pattern).expect("Failed to compile regex");
    release_info
        .assets
        .iter()
        .find(|asset| regex.is_match(asset.name.as_str()))
}

pub fn fetch_hashes(release_info: &ReleaseInformation) -> anyhow::Result<String> {
    let hashes_asset = match find_asset(release_info, "^hashes.txt$") {
        None => return Err(HashesError::AssetNotFound.into()),
        Some(hashes_asset) => hashes_asset,
    };

    Ok(http::download_str(hashes_asset.url.as_str(), None).map_err(|_| HashesError::FetchError)?)
}

pub fn parse_hashes(s: &str) -> HashMap<&str, &str> {
    let mut map = HashMap::new();

    for line in s.lines() {
        if let Some((hash, filename)) = line
            .split_once(char::is_whitespace)
            .map(|(h, f)| (h, f.trim_start()))
        {
            map.insert(filename, hash);
        }
    }

    map
}

pub enum ParseError {
    FetchError,
    InvalidResponse,
    TagName,
    ReleaseAssets,
}

impl ParseError {
    fn message(&self) -> &str {
        match self {
            Self::FetchError => "Failed to fetch latest release information",
            Self::InvalidResponse => "Invalid JSON response from GitHub API",
            Self::TagName => "Couldn't get tag name",
            Self::ReleaseAssets => "Couldn't get release assets",
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ParseError {}

pub enum HashesError {
    AssetNotFound,
    FetchError,
}

impl HashesError {
    fn message(&self) -> &str {
        match self {
            Self::AssetNotFound => "Couldn't find hashes.txt asset",
            Self::FetchError => "Failed to fetch hashes.txt",
        }
    }
}

impl Display for HashesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for HashesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for HashesError {}
