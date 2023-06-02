pub mod youtube_api {

    use reqwest::blocking::{Client, Response};
    use serde_json::Value;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct YoutubeVideo {
        pub video_id: String,
        pub api_key: String,
        pub client: Client,
    }

    #[derive(Debug)]
    struct YoutubeComment {
        comment_id: String,
        parent_id: Option<String>,
        total_reply_count: usize,
        text: String,
    }

    impl YoutubeComment {
        pub fn is_parent(&self) -> bool {
            self.total_reply_count > 0
        }
    }
    //I think I will have to implement my own errors
    //but not before i get the data parsing out of the way
    //I am excited to see how that goes in rust

    impl YoutubeVideo {
        fn youtube_request(&self, url: &str, params: HashMap<&str, &str>) -> Option<Response> {
            let request_get_comments = match self.client.get(url).query(&params).build() {
                Ok(req) => req,
                Err(_) => return None,
            };

            match self.client.execute(request_get_comments) {
                Ok(response) => Some(response),
                Err(_) => None,
            }
        }

        pub fn request_video_comment_thread_list(
            &self,
            next_page_token: Option<&str>,
        ) -> Option<Value> {
            // add paramaters
            let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
            // https://developers.google.com/youtube/v3/docs/commentThreads/list
            let mut params = HashMap::new();
            params.insert("part", "id,snippet");
            params.insert("videoId", &self.video_id);
            params.insert("textFormat", "plainText");
            params.insert("key", &self.api_key);
            params.insert("maxResults", "100");

            if next_page_token.is_some() {
                params.insert("pageToken", &next_page_token.unwrap());
            }

            self.parse_raw_response(self.youtube_request(base_url, params))
        }

        pub fn request_comment_replies(
            &self,
            comment_id: &str,
            next_page_token: Option<&str>,
        ) -> Option<Value> {
            let base_url: &str = "https://www.googleapis.com/youtube/v3/comments";
            let mut params = HashMap::new();

            params.insert("part", "id,snippet");
            params.insert("parentId", comment_id);
            params.insert("key", &self.api_key);

            if next_page_token.is_some() {
                params.insert("pageToken", &next_page_token.unwrap());
            }

            self.parse_raw_response(self.youtube_request(base_url, params))
        }

        fn parse_raw_response(&self, yt_req_response: Option<Response>) -> Option<Value> {
            //takes any request from youtube and gives back the serde_json

            let request_response_text = &yt_req_response?.text().ok()?;

            serde_json::from_str(&request_response_text).unwrap_or_else(|_| None)
        }
    }

    fn get_comment_threads(json_response: &Value) -> Vec<YoutubeComment> {
        let item_iter = json_response["items"].as_array().unwrap();
        let mut item_vec: Vec<YoutubeComment> = Vec::new();

        for item in item_iter {
            let json_object = item.as_object();

            let id = json_object
                .and_then(|obj| obj.get("id"))
                .and_then(|id| id.as_str())
                .unwrap()
                .to_owned();

            let reply_count = json_object
                .and_then(|obj| obj.get("snippet"))
                .and_then(|snippet| snippet.get("totalReplyCount"))
                .and_then(|number| number.as_u64())
                .unwrap();

            let text_original = json_object
                .and_then(|obj| obj.get("snippet"))
                .and_then(|snippet| snippet.get("topLevelComment"))
                .and_then(|top_comment| top_comment.get("snippet"))
                .and_then(|snippet| snippet.get("textOriginal"))
                .unwrap();

            item_vec.push(YoutubeComment {
                comment_id: id,
                text: text_original.as_str().unwrap().to_string(),
                total_reply_count: reply_count as usize,
                parent_id: None,
            });
        }

        item_vec
    }

    fn parse_video_comment_list(parsed_response: &Value) -> String {
        parsed_response["items"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .get("snippet")
            .unwrap()
            .get("textOriginal")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }

    fn get_next_page_token(parsed_response: &Value) -> Option<String> {
        //TODO error parsing
        parsed_response["nextPageToken"]
            .as_str()
            .map(|s| s.to_string())
    }

    pub fn get_all_comments(video: YoutubeVideo) {
        let mut next_page_token: Option<String> = None;
        let mut comment_thread_map: Vec<YoutubeComment> = Vec::new();

        loop {
            let token_str: Option<&str> = next_page_token.as_deref();
            let response = { video.request_video_comment_thread_list(token_str) }.unwrap();

            next_page_token = get_next_page_token(&response);
            comment_thread_map.extend(get_comment_threads(&response));

            if next_page_token.is_none() {
                break;
            }
        }

        //get more YoutubeComment if they have replies
        //set their parent id
        let comment_parent_iter = comment_thread_map
            .iter()
            .filter(|comment| comment.is_parent());

        next_page_token = None;

        for parent_comment in comment_parent_iter {
            let token_str: Option<&str> = next_page_token.as_deref();
            let reponse = video
                .request_comment_replies(&parent_comment.comment_id, token_str)
                .unwrap();
            //do something with the response
        }
    }
}

pub mod youtube_url_parsing {

    use core::panic;
    use regex::Regex;
    use std::error;
    use std::fmt;
    use url;

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
                YoutubeUrlError::ParseError(ref err) => write!(f, "Url Parse Error: {}", err),
                YoutubeUrlError::InvalidDomain => write!(f, "Not a valid youtube video url"),
                YoutubeUrlError::NoVideoIdFound => write!(f, "No video found from URL"),
            }
        }
    }

    const YOUTUBE_DOMAINS: [&str; 4] =
        ["youtube.com", "youtu.be", "www.youtube.com", "www.youtu.be"];

    //returns the String of the video id from a URL
    pub fn get_video_id_from_url(video_url: &str) -> Result<String, YoutubeUrlError> {
        //parse the url and get the video id from the url
        //make sure the url is from youtube
        //the second is just a paramater of v
        let url_parser = url::Url::parse(video_url)?;

        //parse the url that we now have
        //check if there is a host string
        if let Some(url_host_string) = url_parser.host_str() {
            return match YOUTUBE_DOMAINS.contains(&url_host_string) {
                true => {
                    //we want to now do something with the fact that we know
                    //we have a youtube url
                    parse_youtube_domain(&url_parser)
                }
                false => Err(YoutubeUrlError::InvalidDomain),
            };
        } else {
            panic!("There is no host string");
        }
    }

    fn parse_youtube_domain(url_parser: &url::Url) -> Result<String, YoutubeUrlError> {
        match url_parser.host_str() {
            Some(host) if host == YOUTUBE_DOMAINS[0] || host == YOUTUBE_DOMAINS[2] => {
                //youtube.com
                //there should be query v=video+id
                let query = url_parser
                    .query()
                    .ok_or(YoutubeUrlError::NoVideoIdFound)
                    .map(|s| s.to_string())?;

                //take out the v=
                let re = Regex::new(r"^v=(.+)").unwrap();
                Ok(re
                    .captures(&query)
                    .ok_or(YoutubeUrlError::NoVideoIdFound)?
                    .get(1)
                    .ok_or(YoutubeUrlError::NoVideoIdFound)?
                    .as_str()
                    .to_string())
            }
            Some(host) if host == YOUTUBE_DOMAINS[1] || host == YOUTUBE_DOMAINS[3] => {
                //youtu.be
                //there should just be a path /video_id
                let path = url_parser.path().to_string();
                // take out /
                let re = Regex::new(r"^(?:/)(.+)").unwrap();
                Ok(re
                    .captures(&path)
                    .ok_or(YoutubeUrlError::NoVideoIdFound)?
                    .get(1)
                    .ok_or(YoutubeUrlError::NoVideoIdFound)?
                    .as_str()
                    .to_string())
            }
            _ => Err(YoutubeUrlError::NoVideoIdFound),
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
            Ok(id) => assert_eq!(id, video_id),
            Err(_) => (),
        }

        let share_url_id = get_video_id_from_url(share_url);
        assert!(share_url_id.is_ok());
        match share_url_id {
            Ok(id) => assert_eq!(id, video_id),
            Err(_) => (),
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
