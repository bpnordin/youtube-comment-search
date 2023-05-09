use std::error::Error;
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

    let client = reqwest::blocking::Client::builder().build()?;

    let request_get_comments = request_video_comment_thread(
        &video_id, &api_key, &client).unwrap();

    //use ths to open the request in a browser -- for now
    println!("{}",request_get_comments);
    println!("Video ID: {}", video_id);

    Ok(())
}

fn request_video_comment_thread(video_id: &str,
                                api_key: &str,
                                client: &reqwest::blocking::Client) -> Option<String> {

    // add paramaters 
    let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
    // https://developers.google.com/youtube/v3/docs/commentThreads/list
    let mut params = HashMap::new();
    params.insert("part","id");
    params.insert("videoId", &video_id);
    params.insert("key",&api_key);
    params.insert("maxResults","100");

    let request_get_comments = match client.get(base_url)
        .query(&params)
        .build() {
            Ok(req) => req,
            Err(_) => return None
        };
    //TODO return the request in json so I can parse
    Some(request_get_comments.url().to_string())

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
