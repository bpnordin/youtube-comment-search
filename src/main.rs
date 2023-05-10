use std::error::Error;
use config::Config;
use std::collections::HashMap;
use url::Url;
use reqwest::blocking::{Client,Response};
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {

    // get the google youtube api
    let config = Config::builder()
        .add_source(config::File::new("secrets",config::FileFormat::Ini))
        .build()
        .unwrap();
    let api_key: String = config.get_string("youtube.api_key")?;

    // TODO make this a user input so this can be done with CLI
    let video_url: &str = "https://youtu.be/Ou5xmqgkN9c";

    let video_id = match parse_youtube_url(video_url) 
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
    let request_get_comments = request_video_comment_thread(
        &video_id, &api_key, &client).unwrap().text()?;
    
    let yt_data: Value = serde_json::from_str(&request_get_comments)?;
    println!("{:#?}",yt_data["items"][0]["id"]);

    println!("Video ID: {}", video_id);

    Ok(())
}

fn request_next_page(video_id: &str, api_key: &str, client: &Client, url: &str,
                                next_page_token: &str) -> Option<String> {
    //create the request for the next page

    // https://developers.google.com/youtube/v3/docs/commentThreads/list
    let mut params = HashMap::new();
    params.insert("part","id");
    params.insert("videoId", &video_id);
    params.insert("key",&api_key);
    params.insert("maxResults","5");
    params.insert("pageToken",&next_page_token);

    let request_next_page = match client.get(url)
        .query(&params)
        .build() {
            Ok(req) => req,
            Err(_) => return None
        };
    
    Some(request_next_page.url().to_string())

}

fn request_video_comment_thread(video_id: &str, api_key: &str,
                                client: &Client) -> Option<Response> {

    // add paramaters 
    let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
    // https://developers.google.com/youtube/v3/docs/commentThreads/list
    let mut params = HashMap::new();
    params.insert("part","id");
    params.insert("videoId", &video_id);
    params.insert("key",&api_key);
    params.insert("maxResults","5");

    let request_get_comments = match client.get(base_url)
        .query(&params)
        .build() {
            Ok(req) => req,
            Err(_) => return None
        };

    match client.execute(request_get_comments)
    {
        Ok(response) => Some(response),
        Err(_) => None
    }

}

fn parse_youtube_url(video_url: &str) -> Option<String> {
    //parse the url and get the video id from the url
    //make sure the url is from youtube
    //the second is just a paramater of v

    let url = match Url::parse(video_url){
        Ok(url) => url,
        Err(_) => return None
    };

    //there are 2 types of url, the shared youtu.be/VIDEO_ID
    //and the youtube.com/watch?v=VIDEO_ID
    let video_id = match url.host_str()
    {
        //https://doc.rust-lang.org/std/option/enum.Option.html#method.map
        Some("youtube.com") =>  url.query().map(|q| q.to_owned()),
        Some("youtu.be") =>  url.path().split('/').last().map(|s| s.to_owned()),
        _ =>  None
    };
    return video_id
   


}
