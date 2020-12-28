# Datamuse Api Wrapper
This library provides an unofficial wrapper around the Datamuse api for use in rust projects.
The Datamuse api allows users to retrieve lists of words from different vocabulary lists
based on a large number of different parameters. Additionally, it can be used to retrieve
words based on a short string of letters, allowing for autocomplete functionality.
Currently, the api does not require an api key and can be used to make up to 100,000
requests per day after which the requests may be rate-limited (for higher usage see website).
If you use this api you should make reference to the Datamuse api in your project.
For more information see the official documentation at [https://www.datamuse.com/api/](https://www.datamuse.com/api/)

## Examples
### Words Endpoint
```rust
extern crate tokio;
extern crate datamuse_api_wrapper;
use datamuse_api_wrapper::{ DatamuseClient, Vocabulary, EndPoint, RelatedType };

#[tokio::main]
async fn main() -> datamuse_api_wrapper::Result<()> {
    let client = DatamuseClient::new();
    let request = client.new_query(Vocabulary::English, EndPoint::Words)
        .means_like("breakfast") // The words we're looking for are related to "breakfast"
        .related(RelatedType::Rhyme, "grape"); // and rhyme with "grape"
    let word_list = request.list().await?; // Get the list of words from the api

    assert_eq!("crepe", word_list[0].word); // "crepe" is the only result as of writing this

    Ok(())
}
```

### Suggest EndPoint
```rust
extern crate tokio;
extern crate datamuse_api_wrapper;
use datamuse_api_wrapper::{ DatamuseClient, Vocabulary, EndPoint };

#[tokio::main]
async fn main() -> datamuse_api_wrapper::Result<()> {
    let client = DatamuseClient::new();
    let request = client.new_query(Vocabulary::English, EndPoint::Suggest)
        .hint_string("hello wor") // The user has alread typed in "hello wor"
        .max_results(2); // We only want the first 2 results to be returned

    let request = request.build()?; // Build the request
    let response = request.send().await?; // Send the request
    let word_list = response.list()?; // Parse the response into a word_list

    assert_eq!("hello world", word_list[0].word); // "hello world" is the first result as of writing this

    Ok(())
}
```