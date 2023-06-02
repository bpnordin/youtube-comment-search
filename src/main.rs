use clap::Parser;
use config as config_reader;
use reqwest::blocking::Client;
use std::error::Error;
use youtube_comment_search::{youtube_api, youtube_url_parsing};
use youtube_url_parsing::YoutubeUrlError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    video_url: Option<String>,
    search_term: Option<String>,
    #[arg(short,long,default_value_t = String::from("secrets.ini"))]
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let video_url: String = match cli.video_url {
        Some(url) => url,
        None => {
            eprintln!("No video url argument provided, please provide one");
            std::process::exit(1);
        }
    };

    #[allow(unused_variables)]
    let search_term: String = match cli.search_term {
        Some(search) => search,
        None => {
            eprintln!("No search term provided, please provide one");
            std::process::exit(1);
        }
    };

    let config_file = config_reader::File::new(&cli.config, config::FileFormat::Ini);
    let config = config_reader::Config::builder()
        .add_source(config_file)
        .build()
        .unwrap_or_else(|error| {
            eprintln!("Error reading config file: {error}");
            std::process::exit(1);
        });

    let api_key: String = config
        .get_string("youtube.api_key")
        .unwrap_or_else(|error| {
            eprintln!("Error reading api_key from config file: {error}");
            std::process::exit(1);
        });

    let video_id = match youtube_url_parsing::get_video_id_from_url(&video_url) {
        Ok(value) => value,
        Err(YoutubeUrlError::InvalidDomain) => {
            eprintln!(
                "Invalid Domain, please provide either a youtube.com or 
                      youtu.be url"
            );
            return Ok(()); // early return
        }
        Err(YoutubeUrlError::NoVideoIdFound) => {
            eprintln!("No video ID found in the url {video_url}");
            return Ok(()); // early return
        }
        Err(_) => {
            eprintln!(
                "Failed to parse the URL \x1b[31m{}\x1b[0m...exiting",
                video_url
            );
            return Ok(()); // early return
        }
    };

    let client = Client::builder().build()?;

    let video1 = youtube_api::YoutubeVideo {
        video_id: video_id.to_string(),
        api_key: api_key.to_string(),
        client: client.to_owned(),
    };
    dbg!(&video1);
    //parse with serde_json
    youtube_api::get_all_comments(video1);

    Ok(())
}
