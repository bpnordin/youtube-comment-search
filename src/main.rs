use std::{error::Error, fmt::format};
use config::Config;
use std::collections::HashMap;
use url::Url;

fn main() -> Result<(), Box<dyn Error>> {

    // get the google youtube api
    let config = Config::builder()
        .add_source(config::File::new("secrets",config::FileFormat::Ini))
        .build()
        .unwrap();
    let api_key: String = config.get_string("youtube.api_key")?;

    let client = reqwest::blocking::Client::new();
    let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";

    // TODO make this a user input so this can be done with CLI
    let video_url: &str = "https://youtu.be/Ou5xmqgkN9c";


    let video_id = match parse_youtube_url(video_url){
        Some(value) => value,
        None =>{
            println!("Failed to parse the URL \x1b[31m{}\x1b[0m...exiting",video_url);
            return Ok(())
        }
    };


    // add paramaters 
    let mut params = HashMap::new();
    params.insert("part","snippet,replies");
    params.insert("videoId", &video_id);
    params.insert("key",&api_key);
    
    let resp_1 = client.get(base_url)
        .query(&params)
        .send()?
        .text()?;
    println!("{:#?}", resp_1);
    
    println!("Video ID: {}", video_id);
    Ok(())
}

fn parse_youtube_url(video_url: &str) -> Option<String> {
    //parse the url and get the video id from the url
    //make sure the url is from youtube
    //the second is just a paramater of v

    let url = match Url::parse(video_url){
        Ok(url) => url,
        Err(_) =>{
            println!("Failed to perform operation");
            return None
        },
    };

    //there are 2 types of url, the shared youtu.be/VIDEO_ID
    //and the youtube.com/watch?v=VIDEO_ID
    let video_id = match url.host_str()
    {
        Some("youtube.com") =>  url.query().map(|q| q.to_owned()),
        Some("youtu.be") =>  url.path().split('/').last().map(|s| s.to_owned()),
        _ =>  None
    };
    return video_id
   


}
