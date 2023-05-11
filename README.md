# youtube-comment-search
search youtube comments

### TODO 
- [etags](https://stackoverflow.com/questions/21752421/youtube-api-v3-and-etag)
tell you if something has been changed. We can have a local cache for 
searches, and only go get the comment when it changed
- a search takes 100 quota. While we can get 100 comment threads with 1 quota.
    - do the math to find out which one is better
- figure out the async stuff for CLI (main.rs) and for the API (lib.rs)
- integration tests

### Structure of search + thoughts
- [Comment Threads](https://developers.google.com/youtube/v3/docs/commentThreads/list)
get a list of comment threads with the following query terms:
    -("part","id,replies,snippet") 
    -("videoId",&videoId)
    -("maxResults","100")
    -("pageToken",&pageToken) *when we need to go to the next page*
- snippet makes it sound like it is not the whole comment, but I am pretty
sure that it is the whole comment


- I have to get all of the comments with the comment api, but only if the etag 
changes
    - >([Note](https://developers.google.com/youtube/v3/docs/commentThreads)
    that a commentThread resource does not necessarily
    contain all replies to a comment, and you need to use
    the comments.list method if you want to retrieve all replies for a particular comment.)
- use a text search ono the machine itself (https://github.com/quickwit-oss/tantivy)
