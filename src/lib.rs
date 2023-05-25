pub mod youtube_api { 

    use std::collections::HashMap;
    use reqwest::blocking::{Client,Response};
    use serde_json::Value;

    #[derive(Debug)]
    pub struct YoutubeVideoComments {
        pub video_id: String,
        pub api_key: String,
        pub client: Client,
    }


    impl YoutubeVideoComments {

        fn request_next_page(&self, url: &str,
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
        fn request_video_comment_thread_list(&self) -> Option<Response> {

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
                Ok(response) => {
                    Some(response)
                },
                Err(_) => None
            }
        }

        fn parse_video_comment_thread_list(&self,comment_thread_list_response: Option<Response>) {

            let request_response_text = &comment_thread_list_response
                .map(|response| response.text()
                     .unwrap_or_else(|_| "".to_string()))
                .unwrap_or_else(|| "".to_string());

            let yt_data: Value = serde_json::from_str(&request_response_text)
                .unwrap_or_else(|_| Value::Null);

            dbg!(&yt_data);

        }
        
        pub fn search_comments(&self, search_term: &str) {
            let response = self.request_video_comment_thread_list();
            self.parse_video_comment_thread_list(response);
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

#[cfg(test)]
mod tests {
    use crate::youtube_api::youtube_url_parsing::get_video_id_from_url;

    
    //test the url parser
    #[test]
    fn url_parser_video_id_success() {
        //test successfully getting the video id
        let video_url = "https://www.youtube.com/watch?v=JVtkE8cgdOw";
        let share_url = "https://youtu.be/JVtkE8cgdOw";
        let video_id = "JVtkE8cgdOw";
        
        let video_url_id = get_video_id_from_url(video_url);
        assert!(video_url_id.is_ok());
        match video_url_id {
            Ok(id) => assert_eq!(id,video_id),
            Err(_) => ()
        }

        let share_url_id = get_video_id_from_url(share_url);
        assert!(share_url_id.is_ok());
        match share_url_id {
            Ok(id) => assert_eq!(id,video_id),
            Err(_) => ()
        }
        
    }

    #[test]
    fn url_parser_video_id_fail() {

        //test when url is not correct
        let no_video_id = "https://www.youtube.com/";
        let no_share_id = "https://youtu.be/";
        let wrong_domain = "twitter.com";
        
        assert!(get_video_id_from_url(no_video_id).is_err());
        assert!(get_video_id_from_url(no_share_id).is_err());
        assert!(get_video_id_from_url(wrong_domain).is_err());

    }
}
