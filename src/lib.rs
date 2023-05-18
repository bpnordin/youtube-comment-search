pub mod youtube_api { 

    use std::collections::HashMap;
    use reqwest::blocking::{Client,Response};

    pub fn request_next_page(video_id: &str, api_key: &str, client: &Client, url: &str,
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

    pub fn request_video_comment_thread(video_id: &str, api_key: &str,
                                    client: &Client) -> Option<Response> {

        // add paramaters 
        let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
        // https://developers.google.com/youtube/v3/docs/commentThreads/list
        let mut params = HashMap::new();
        params.insert("part","id,replies");
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

    pub mod youtube_url_parsing {

        use url;
        use core::panic;
        use std::error;
        use std::fmt;
        use regex::RegexSet;


        #[derive(Debug)]
        pub enum YoutubeUrlError {
            ParseError(url::ParseError),
            InvalidDomain,
            NoVideoIdFound,
        }

        impl error::Error for YoutubeUrlError {}

        impl From<url::ParseError> for YoutubeUrlError {
            fn from(err: url::ParseError) -> YoutubeUrlError {
                YoutubeUrlError::ParseError(err)
            }
        }

        impl fmt::Display for YoutubeUrlError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                    YoutubeUrlError::ParseError(ref err) => 
                        write!(f,"Url Parse Error: {}",err),
                    YoutubeUrlError::InvalidDomain =>
                        write!(f,"Not a valid youtube video url"),
                    YoutubeUrlError::NoVideoIdFound =>
                        write!(f,"No video found from URL"),
                }
            }
        }

        //returns the String of the video id from a URL
        pub fn get_video_id_from_url(video_url: &str) -> Result<String,YoutubeUrlError> {

            //parse the url and get the video id from the url
            //make sure the url is from youtube
            //the second is just a paramater of v
            let re_video_domain: 
                RegexSet = RegexSet::new(&[
                                         r"^w*\.*youtube.com",//for youtube.com
                                         r"^w*\.*.youtu.be",//for youtu.be
                ]).unwrap();

            let youtube_domains = vec!["youtube.com","youtu.be","www.youtube.com",
            "www.youtu.be"];


            let url_parser = url::Url::parse(video_url)?; 

            dbg!(&url_parser);

            //parse the url that we now have
            //check if there is a host string
            if let Some(url_host_string) = url_parser.host_str() {
                return match youtube_domains.contains(&url_host_string){
                    true => match re_video_domain.matches(&url_host_string)
                        .into_iter()
                        .collect::<Vec<_>>()
                        .as_slice() {
                            [0] => { //youtube.com
                                url_parser.query().ok_or(
                                    YoutubeUrlError::NoVideoIdFound)
                                    .map(|s| s.to_owned())
                            },
                            [1] => { //youtu.be
                                Ok(url_parser.path().to_string())
                            },
                            [0,1] => panic!("we have mathced too many urls"),
                            _ => panic!("we have not matched any urls"),
                        },
                    false => Err(YoutubeUrlError::InvalidDomain),
                }
            }else {
                panic!("There is no host string on ");
            }
        }
    }
} 
