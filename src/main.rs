use std::error::Error;
use config::Config;
use reqwest::blocking::{Client,Response};
use serde_json::Value;
use youtube_comment_search::{youtube_api,config_youtube};

fn main() -> Result<(), Box<dyn Error>> {

    //TODO improve the config intake error handling
    let api_key: String = config_youtube::new("secrets.ini")?;

    // TODO make this a user input so this can be done with CLI
    let video_url: &str = "https://youtu.be/Ou5xmqgkN9c";

    let video_id = match youtube_api::parse_youtube_url(video_url) 
        {
        Some(value) => value,
        None =>
            {
            println!("Failed to parse the URL \x1b[31m{}\x1b[0m...exiting"
                     ,video_url);
            return Ok(())
            }
        };

    let client = Client::builder().build()?;

    //parse with serde_json
    let request_get_comments = youtube_api::request_video_comment_thread(
        &video_id, &api_key, &client).unwrap().text()?;
    
    let yt_data: Value = serde_json::from_str(&request_get_comments)?;
    println!("{:#?}",yt_data);

    println!("Video ID: {}", video_id);

    Ok(())
}

