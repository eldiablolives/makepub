use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EpubInfo {
    pub id: Option<String>,
    pub name: String,
    pub author: String,
    pub title: String,
    pub start: Option<String>,
    pub start_title: Option<String>,
    pub fonts: Option<Vec<String>>
}

#[derive(Clone)]
pub struct Page {
    pub name: String,
    pub file: String,
    pub title: String,
    pub body: String,
}