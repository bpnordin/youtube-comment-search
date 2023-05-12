use std::error::Error;
use reqwest::blocking::{Client,Response};
use serde_json::Value;
use youtube_comment_search::{youtube_api,youtube_url};
use config as config_reader;
use std::env;


fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    dbg!(args);
    //TODO improve the config intake error handling
    //and move this to a config option ffrom the cli
    let config_file_name: &str = "secrets.ini";
    let config_file = config_reader::File::new(config_file_name, config::FileFormat::Ini);
    let config = config_reader::Config::builder()
        .add_source(config_file)
        .build()  
        .unwrap_or_else(|error| {
            eprintln!("Error reading config file: {error}");
            std::process::exit(1);
        }
        );
    let api_key: String = config.get_string("youtube.api_key").unwrap_or_else(|error| {
        eprintln!("Error reading api_key from config file: {error}");
        std::process::exit(1);
    }
    );
    // TODO make this a user input so this can be done with CLI
    // and more 
    let video_url: &str = "https://youtu.be/Ou5xmqgkN9c";

    let video_id = match youtube_url::parse_youtube_url(video_url) 
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

