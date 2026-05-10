#[derive(Debug, Clone)]
pub struct Reference {
    pub id: String,
    pub entry_key: String,
    pub entry_type: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}
