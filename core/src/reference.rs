#[derive(Debug, Clone)]
pub struct Reference {
    pub id: String,
    pub key: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}
