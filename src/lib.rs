use anyhow as ah;
use html_parser::{Dom, Element, Node};
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

    // gets the rendered html from file on github specified by path starting with a /
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

// takes an html template with comments as the key to replace
// for example <!--PROJECT--> would be replaced from value { "PROJECT", "<h1>project</h1>" }
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

fn get_h1_title_recur(node: &Element) -> Option<String> {
    for elem in &node.children {
        if let Node::Element(e) = elem {
            // if it is not an h1 then check all of it's children to find
            // the h1
            if &e.name != "h1" {
                let res = get_h1_title_recur(&e);
                if res.is_some() {
                    return res;
                }
                continue;
            }

            // if it is an h1 then go through its children to find
            // it's text
            for elem in &e.children {
                if let Node::Text(text) = elem {
                    return Some(text.to_owned());
                }
            }
        }
    }

    None
}

// gets the value in the <h1> first title if there is one without any
// other inner elements
pub fn get_h1_title(html: &str) -> Option<String> {
    let dom = Dom::parse(html).ok()?;
    for elem in dom.children {
        if let Node::Element(e) = elem {
            let res = get_h1_title_recur(&e);
            if res.is_some() {
                return res;
            }
        }
    }

    None
}
