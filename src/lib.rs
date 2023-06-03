use anyhow as ah;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::collections::HashMap;

pub struct GitHubApi {
    req_acc: &'static str,
    repo_owner: String,
    repo_name: String,
    client: Client,
}

impl GitHubApi {
    pub fn new(req_acc: &'static str, repo_owner: String, repo_name: String) -> GitHubApi {
        GitHubApi {
            req_acc,
            repo_owner,
            repo_name,
            client: Client::new(),
        }
    }

    pub async fn get_html_from_markdown(&self, path_to_md: String) -> ah::Result<String> {
        if path_to_md.starts_with("/") == false {
            panic!("github path expected to start with slash");
        }
        let req_url = format!(
            "https://api.github.com/repos/{}/{}/contents{}",
            &self.repo_owner, &self.repo_name, &path_to_md
        );

        let mut headers = HeaderMap::new();
        // add the account as the user agent
        headers.insert("User-Agent", HeaderValue::from_static(self.req_acc));

        // tell it that we want back the html from the markdown file
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github.html"),
        );

        let html = self
            .client
            .get(&req_url)
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;
        Ok(html)
    }
}

pub fn render_to_template(html_template: &str, var_replace: HashMap<&str, &str>) -> String {
    let mut lookup = &html_template[..];
    let mut output = String::new();

    while let Some(start) = lookup.find("<!--") {
        output.push_str(&lookup[..start]);

        let Some(end) = lookup[start..].find("-->") else {
            break;
        };
        // it needs to be offset by the start index
        let end = end + start;

        let var_name = &lookup[start + 4..end];
        match var_replace.get(var_name) {
            Some(new) => output.push_str(&new),
            None => output.push_str(&lookup[start..end + 3]),
        }

        // the index of the first char past -->
        lookup = &lookup[end + 3..];
    }

    // push all of the values that happen after the last comment
    output.push_str(&lookup[..]);

    output
}
