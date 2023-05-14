use std::error::Error;
use reqwest::blocking::Client;
use serde_json::Value;
use youtube_comment_search::{youtube_api,youtube_url};
use config as config_reader;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    video_url: Option<String>,
    #[arg(short,long)]
    search_term: Option<String>,
    #[arg(short,long,default_value_t = String::from("secrets.ini"))]
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {

    let cli = Cli::parse();

    let video_url: String = match cli.video_url{ 
        Some(url) => url,
        None => {
            eprintln!("No video url argument provided, please provide one");
            std::process::exit(1);
        }
    };
    dbg!(&video_url);

    let config_file = config_reader::File::new(&cli.config, config::FileFormat::Ini);
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


    let video_id = match youtube_url::parse_youtube_url(&video_url.as_str()) 
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
    
    let _yt_data: Value = serde_json::from_str(&request_get_comments)?;

    dbg!(&video_id);

    Ok(())
}

