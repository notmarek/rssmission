use reqwest;
use rss::Channel;
use std::fs::OpenOptions;
use std::io::Write;
mod models;
use data_encoding::HEXLOWER;
use models::{Config, Matcher, RegexMatcher};
use regex::Regex;
use ring::digest::{Context, SHA256};
use std::fs;
use transmission_rpc::types::BasicAuth;
use transmission_rpc::types::TorrentAddArgs;
use transmission_rpc::TransClient;
// fn build_regexp(matchers: Vec<Matcher>) -> Regex { // Combines all defined matchers into one just realize im very dumb tho..................................................................
//     let mut regex_matcher_string: String = "".to_owned();
//     for (i, matcher) in matchers.iter().enumerate() {
//         if i == 0 {
//             regex_matcher_string.push_str(format!("({})", matcher.regexp.as_ref().unwrap()).as_str())
//         } else {
//             regex_matcher_string.push_str(format!("|({})", matcher.regexp.as_ref().unwrap()).as_str())
//         }
//     }
//     let result = Regex::new(regex_matcher_string.as_ref()).unwrap();
//     result
// }
fn build_regexp(matchers: Vec<Matcher>) -> Vec<RegexMatcher> {
    // creates a vector of regex matchers
    let mut result: Vec<RegexMatcher> = Vec::new();
    for matcher in matchers {
        result.push(RegexMatcher {
            matcher: Regex::new(format!(r"{}", matcher.regexp.unwrap()).as_str()).unwrap(),
            path: matcher.path,
        });
    }
    result
}

#[tokio::main]
async fn main() {
    let config_file: String = fs::read_to_string("rssmission.json")
        .expect("Something went wrong reading the configuration file");
    let mut seen_file: String = fs::read_to_string(".seen").unwrap_or(String::new());
    let config: Config = serde_json::from_str(format!("{}", config_file).as_str()).unwrap();
    let url: &String = &config.server.as_ref().unwrap().url.as_ref().unwrap();
    let username: &String = &config.server.as_ref().unwrap().username.as_ref().unwrap();
    let password: &String = &config.server.as_ref().unwrap().password.as_ref().unwrap();
    let transmission: TransClient = TransClient::with_auth(
        url,
        BasicAuth {
            user: username.to_string(),
            password: password.to_string(),
        },
    );
    let client = reqwest::Client::new();
    for feed in config.feeds.unwrap() {
        println!("Collecting data from {}.", feed.url.as_ref().unwrap());
        let content = client
            .get(&feed.url.unwrap_or(String::new()))
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let channel = Channel::read_from(&content[..]).unwrap();
        let items = channel.items();
        let matchers: Vec<RegexMatcher> = build_regexp(feed.matchers.unwrap());
        for item in items {
            let title = item.title().unwrap();
            for matcher in &matchers {
                if matcher.matcher.is_match(title) {
                    let mut context: Context = Context::new(&SHA256);
                    context.update(&item.link().unwrap().as_bytes());
                    let sha256digest: String = HEXLOWER.encode(context.finish().as_ref());
                    if !seen_file.contains(sha256digest.as_str()) {
                        println!("Adding \"{}\" to transmission.", title);
                        let path = match matcher.path.as_ref() {
                            Some(path) => Some(path.to_string()),
                            _ => None,
                        };
                        let torrent_link: String = {
                            if let Some(enc) = item.enclosure() {
                                if enc.mime_type() == "application/x-bittorrent" {
                                    enc.url().to_string()
                                } else {
                                    item.link().unwrap().to_string()
                                }
                            } else {
                                item.link().unwrap().to_string()
                            }
                        };
                        let add: TorrentAddArgs = TorrentAddArgs {
                            filename: Some(torrent_link),
                            download_dir: path,
                            ..TorrentAddArgs::default()
                        };
                        match transmission.torrent_add(add).await.unwrap().is_ok() {
                            true => println!("Torrent added successfully."),
                            false => println!(
                                "Torrent couldn't be added, make sure your config is correct."
                            ),
                        };
                        seen_file.push_str(format!("{}\n", sha256digest).as_str());
                    }
                }
            }
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(".seen")
                .unwrap();
            f.write_all(seen_file.as_bytes()).unwrap();
        }
    }
}
