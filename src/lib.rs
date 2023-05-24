pub mod youtube_api { 

    use std::collections::HashMap;
    use reqwest::blocking::{Client,Response};

    #[derive(Debug)]
    pub struct YoutubeVideoComments {
        pub video_id: String,
        pub api_key: String,
        pub client: Client,
    }


    impl YoutubeVideoComments {

        pub fn request_next_page(&self, url: &str,
                                 next_page_token: &str) -> Option<String> {
            //create the request for the next page

            // https://developers.google.com/youtube/v3/docs/commentThreads/list
            let mut params = HashMap::new();
            params.insert("part","id");
            params.insert("videoId", &self.video_id);
            params.insert("key",&self.api_key);
            params.insert("maxResults","5");
            params.insert("pageToken",&next_page_token);

            let request_next_page = match self.client.get(url)
                .query(&params)
                .build() {
                    Ok(req) => req,
                    Err(_) => return None
                };

            Some(request_next_page.url().to_string())

        }
        pub fn request_video_comment_thread(&self) -> Option<Response> {

            // add paramaters 
            let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
            // https://developers.google.com/youtube/v3/docs/commentThreads/list
            let mut params = HashMap::new();
            params.insert("part","id,replies");
            params.insert("videoId", &self.video_id);
            params.insert("key",&self.api_key);
            params.insert("maxResults","5");

            let request_get_comments = match self.client.get(base_url)
                .query(&params)
                .build() {
                    Ok(req) => req,
                    Err(_) => return None
                };

            match self.client.execute(request_get_comments)
            {
                Ok(response) => Some(response),
                Err(_) => None
            }

        }
    }
    


    pub mod youtube_url_parsing {

        use url;
        use core::panic;
        use std::error;
        use std::fmt;
        use regex::Regex;


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

        const YOUTUBE_DOMAINS: [&str;4] = ["youtube.com","youtu.be","www.youtube.com",
        "www.youtu.be"];

        //returns the String of the video id from a URL
        pub fn get_video_id_from_url(video_url: &str) 
            -> Result<String,YoutubeUrlError> {

            //parse the url and get the video id from the url
            //make sure the url is from youtube
            //the second is just a paramater of v
            let url_parser = url::Url::parse(video_url)?; 

            //parse the url that we now have
            //check if there is a host string
            if let Some(url_host_string) = url_parser.host_str() {
                return match YOUTUBE_DOMAINS.contains(&url_host_string){
                    true => {
                        //we want to now do something with the fact that we know
                        //we have a youtube url
                        parse_youtube_domain(&url_parser)
                    },
                    false => Err(YoutubeUrlError::InvalidDomain),
                }
            }else {
                panic!("There is no host string");
            }
        }

        fn parse_youtube_domain(url_parser: &url::Url)
            -> Result<String,YoutubeUrlError> {
            match url_parser.host_str() {
                Some(host) if host == YOUTUBE_DOMAINS[0] || host == YOUTUBE_DOMAINS[2] => {
                    //youtube.com
                    //there should be query v=video+id
                    let query = url_parser.query()
                        .ok_or(YoutubeUrlError::NoVideoIdFound)
                        .map(|s| s.to_string())?;

                    //take out the v=
                    let re = Regex::new(r"^v=(.+)").unwrap();
                    Ok(re.captures(&query)
                        .ok_or(YoutubeUrlError::NoVideoIdFound)?
                        .get(1)
                        .ok_or(YoutubeUrlError::NoVideoIdFound)?
                        .as_str()
                        .to_string())


                },
                Some(host) if host == YOUTUBE_DOMAINS[1] || host == YOUTUBE_DOMAINS[3] => {
                    //youtu.be
                    //there should just be a path /video_id
                    let path = url_parser.path().to_string();
                    // take out /
                    let re = Regex::new(r"^(?:/)(.+)").unwrap();
                    Ok(re.captures(&path)
                        .ok_or(YoutubeUrlError::NoVideoIdFound)?
                        .get(1)
                        .ok_or(YoutubeUrlError::NoVideoIdFound)?
                        .as_str()
                        .to_string())
                },
                _ => {
                    Err(YoutubeUrlError::NoVideoIdFound)
                },
            }
            }

    }
} 
