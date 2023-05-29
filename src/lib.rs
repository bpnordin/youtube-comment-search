pub mod youtube_api { 

    use std::collections::HashMap;
    use lazy_static::__Deref;
    use reqwest::blocking::{Client,Response};
    use serde_json::Value;

    #[derive(Debug)]
    pub struct YoutubeVideoComments {
        pub video_id: String,
        pub api_key: String,
        pub client: Client,
    }

    //I think I will have to implement my own errors
    //but not before i get the data parsing out of the way
    //I am excited to see how that goes in rust
    
    impl YoutubeVideoComments {

        fn request_video_comment_thread_list(&self, next_page_token: Option<&str>) -> Option<Response> {

            // add paramaters 
            let base_url: &str = "https://www.googleapis.com/youtube/v3/commentThreads";
            // https://developers.google.com/youtube/v3/docs/commentThreads/list
            let mut params = HashMap::new();
            params.insert("part","id,snippet");
            params.insert("videoId", &self.video_id);
            params.insert("textFormat","plainText");
            params.insert("key",&self.api_key);
            params.insert("maxResults","100");

            if next_page_token.is_some() {
                params.insert("pageToken",&next_page_token.unwrap() );
            }
            
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

        fn request_video_comments(&self, comment_id: &str, reply_count: usize) -> Option<Response> {

            // add paramaters 
            let base_url: &str = "https://www.googleapis.com/youtube/v3/comments";
            // https://developers.google.com/youtube/v3/docs/commentThreads/list
            let mut params = HashMap::new();
            params.insert("part","id,snippet");
            params.insert("id", comment_id);

            if reply_count > 0 { params.insert("parentId", comment_id);}
            else{ params.insert("id", comment_id);}

            params.insert("key",&self.api_key);

            
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

        fn parse_response(&self,yt_req_response: Option<Response>) -> Option<Value> {
            //takes any request from youtube and gives back the serde_json

            let request_response_text = &yt_req_response?.text().ok()?;

             serde_json::from_str(&request_response_text)
                .unwrap_or_else(|_| None)
        } 

        fn parse_video_comment_thread_list(&self,parsed_response: &Value) -> 
            HashMap<String,usize> {

            let item_iter = parsed_response["items"].as_array().unwrap();
            let mut return_hash_map = HashMap::new();

            //use return_hash_map.insert() to put items in here
            for item in item_iter {
                    // a list of items that are object 
                    // item = {"id":String(id), snippet: object{"totalReplyCount" : i64}}
                    let json_object = item.as_object();

                    let id = json_object.and_then(|obj| obj.get("id"))
                        .and_then(|id| id.as_str())
                        .unwrap();

                    let reply_count = json_object.and_then(|obj| obj.get("snippet"))
                        .and_then(|snippet| snippet.get("totalReplyCount"))
                        .and_then(|number| number.as_u64())
                        .unwrap();
                    return_hash_map.insert(String::from(id), reply_count as usize);
            }
            return_hash_map
        }

        fn parse_video_comment_list(&self,parsed_response: &Value) -> String {

            parsed_response["items"].as_array().unwrap()
                .first().unwrap()
                .get("snippet").unwrap()
                .get("textOriginal").unwrap().as_str().unwrap()
                .to_string()
        }


        fn get_next_page_token(&self,parsed_response: &Value) -> Option<String> {
            
            //TODO error parsing
            parsed_response["nextPageToken"].as_str().map(|s| s.to_string())

        }
        
        pub fn search_comments(&self, search_term: &str) {

            let mut next_page_token:Option<String> = None;
            let mut comment_thread_list: HashMap<String,usize> = HashMap::new();

            loop {

                let token_str: Option<&str> = next_page_token.as_deref();
                let response = self.request_video_comment_thread_list(token_str);
                let parsed_response = self.parse_response(response).unwrap();

                next_page_token = self.get_next_page_token(&parsed_response);
                comment_thread_list.extend(self.parse_video_comment_thread_list(&parsed_response));

                if next_page_token.is_none() {
                    break;
                }
            }
            dbg!(&comment_thread_list);
            dbg!(&comment_thread_list.len());

            //now we take that list, and request each comment
            //if totalReplyCount > 0 then we need to request the parent id
            //there might be a next page if the comment has enough replies
            //the replies are in chronological order
            //
            
            //code:
            //loop through vec of comment ids
            //request to youtube
            //parse the request
            //save the text
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
