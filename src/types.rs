use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thread {
    pub post: Post,
    pub replies: Vec<Reply>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    pub id: i64,
    pub name: String,
    pub board_sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub id: u64,
    pub authors: Vec<User>,
    pub board: BoardInPost,
    pub character: Option<Character>,
    pub content: String,
    pub created_at: String,
    pub description: Option<String>,
    pub icon: Option<Icon>,
    pub num_replies: u64,
    pub section: Option<Section>,
    pub section_order: u64,
    pub status: String,
    pub subject: String,
    pub tagged_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardInPost {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub id: u64,
    pub character: Option<Character>,
    pub character_name: Option<String>,
    pub content: String,
    pub created_at: String,
    pub icon: Option<Icon>,
    pub updated_at: String,
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub screenname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: u64,
    pub name: String,
    pub order: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: u64,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Icon {
    pub id: u64,
    pub keyword: String,
    pub url: String,
}
