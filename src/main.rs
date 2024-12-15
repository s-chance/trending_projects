use chrono::{Datelike, Utc};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Clone)]
struct Repo {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    language: Option<String>,
    updated_at: String,
}

#[derive(Serialize, Deserialize)]
struct ReposResp {
    items: Vec<Repo>,
}

async fn fectch_trending_repos() -> Result<Vec<Repo>, Error> {
    let url = "https://api.github.com/search/repositories?q=stars:>1&sort=stars&order=desc";
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "trending-repos")
        .send()
        .await?
        .json::<ReposResp>()
        .await?;
    Ok(resp.items)
}

fn save_to_file(repos: Vec<Repo>, json_dir: &str, filename: &str) -> std::io::Result<()> {
    create_dir_all(json_dir)?;
    let json_filepath = format!("{}/{}", json_dir, filename);
    let mut file = File::create(&json_filepath)?;
    let json = serde_json::to_string_pretty(&repos)?;
    file.write_all(json.as_bytes())?;

    // Extract the path excluding "metadata/" for markdown directory
    let md_dir = &json_filepath["metadata/".len()..].replace(filename, "");
    convert_json_to_markdown(&json_filepath, md_dir)
}

fn convert_json_to_markdown(json_filename: &str, md_dir: &str) -> std::io::Result<()> {
    let mut file = File::open(json_filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let repos: Vec<Repo> = serde_json::from_str(&data)?;

    create_dir_all(md_dir)?;
    let md_filepath = format!("{}/summary.md", md_dir);
    let mut md_file = File::create(md_filepath)?;

    writeln!(md_file, "# Trending GitHub Repositories")?;
    for repo in repos {
        writeln!(md_file, "## [{}]({})", repo.full_name, repo.html_url)?;
        if let Some(desc) = &repo.description {
            writeln!(md_file, "_{}_", desc)?;
        }
        writeln!(md_file, "- **Stars**: {}", repo.stargazers_count)?;
        if let Some(lang) = &repo.language {
            writeln!(md_file, "- **Language**: {}", lang)?;
        }
        writeln!(md_file, "- **Last Updated**: {}", repo.updated_at)?;
        writeln!(md_file)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let repos = fectch_trending_repos()
        .await
        .expect("Failed to fetch trending repos");

    // Save daily report
    let today = Utc::now();
    let daily_dir = format!(
        "metadata/{}/{:02}/{:02}",
        today.year(),
        today.month(),
        today.day()
    );
    save_to_file(repos.clone(), &daily_dir, "daily.json").expect("Failed to save daily file");

    // Save monthly report
    // let monthly_dir = format!("metadata/{}/{:02}", today.year(), today.month());
    // save_to_file(repos.clone(), &monthly_dir, "summary.json").expect("Failed to save monthly file");
    //
    // // Save yearly report
    // let yearly_dir = format!("metadata/{}", today.year());
    // save_to_file(repos, &yearly_dir, "summary.json").expect("Failed to save yearly file");
}
