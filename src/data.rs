use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Exploit {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "EDB-ID")]
    pub edb_id: String,
    #[serde(rename = "Platform")]
    pub platform: String,
    #[serde(rename = "Type")]
    pub exploit_type: String,
    #[serde(rename = "Path")]
    pub path: String,
}

#[derive(Deserialize)]
pub struct SearchSploitOutput {
    #[serde(rename = "RESULTS_EXPLOIT")]
    pub results: Vec<Exploit>,
}
