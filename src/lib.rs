#![deny( //These lint options are copied from https://pascalhertleif.de/artikel/good-practices-for-writing-rust-libraries/
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

//! # Datamuse Api Wrapper
//! This library provides an unofficial wrapper around the Datamuse api for use in rust projects.
//! The Datamuse api allows users to retrieve lists of words from different vocabulary lists
//! based on a large number of different parameters. Additionally, it can be used to retrieve
//! words based on a short string of letters, allowing for autocomplete functionality.
//! Currently, the api does not require an api key and can be used to make up to 100,000
//! requests per day after which the requests may be rate-limited (for higher usage see website).
//! If you use this api you should make reference to the Datamuse api in your project.
//! For more information see the official documentation at [https://www.datamuse.com/api/](https://www.datamuse.com/api/)
//!
//! ## Examples
//! ### Words Endpoint
//! ```rust
//! extern crate tokio;
//! extern crate datamuse_api_wrapper;
//! use datamuse_api_wrapper::{ DatamuseClient, Vocabulary, EndPoint, RelatedType };
//!
//! #[tokio::main]
//! async fn main() -> datamuse_api_wrapper::Result<()> {
//!     let client = DatamuseClient::new();
//!     let request = client.new_query(Vocabulary::English, EndPoint::Words)
//!         .means_like("breakfast") // The words we're looking for are related to "breakfast"
//!         .related(RelatedType::Rhyme, "grape"); // and rhyme with "grape"
//!     let word_list = request.list().await?; // Get the list of words from the api
//!
//!     assert_eq!("crepe", word_list[0].word); // "crepe" is the only result as of writing this
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Suggest EndPoint
//! ```rust
//! extern crate tokio;
//! extern crate datamuse_api_wrapper;
//! use datamuse_api_wrapper::{ DatamuseClient, Vocabulary, EndPoint };
//!
//! #[tokio::main]
//! async fn main() -> datamuse_api_wrapper::Result<()> {
//!     let client = DatamuseClient::new();
//!     let request = client.new_query(Vocabulary::English, EndPoint::Suggest)
//!         .hint_string("hello wor") // The user has alread typed in "hello wor"
//!         .max_results(2); // We only want the first 2 results to be returned
//!
//!     let request = request.build()?; // Build the request
//!     let response = request.send().await?; // Send the request
//!     let word_list = response.list()?; // Parse the response into a word_list
//!
//!     assert_eq!("hello world", word_list[0].word); // "hello world" is the first result as of writing this
//!
//!     Ok(())
//! }
//! ```

extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::error;
use std::fmt::{self, Display, Formatter};
use std::result;

mod request;
mod response;

pub use request::*;
pub use response::*;

/// This struct represents the client which can be used to make requests
/// to the Datamuse api. Requests can be created using the new_query() method
#[derive(Debug)]
pub struct DatamuseClient {
    client: reqwest::Client,
}

impl DatamuseClient {
    /// Returns a new DatamuseClient struct
    pub fn new() -> Self {
        DatamuseClient {
            client: reqwest::Client::new(),
        }
    }

    /// Returns a new [RequestBuilder](request::RequestBuilder) struct with which requests can be created
    /// and later sent. As parameters the vocabulary set and endpoint of the request are required. See
    /// their individual documentations for more information
    pub fn new_query<'a>(
        &'a self,
        vocabulary: Vocabulary,
        endpoint: EndPoint,
    ) -> RequestBuilder<'a> {
        RequestBuilder::new(self, vocabulary, endpoint)
    }
}

/// A type alias for Results with the library Error type
pub type Result<T> = result::Result<T, Error>;

/// An enum representing the different kind of Errors that can be returned within the library
#[derive(Debug)]
pub enum Error {
    /// An error resulting from an underlying call to reqwest
    ReqwestError(reqwest::Error),
    /// An error resulting from an underlying call to serde
    SerdeError(serde_json::Error),
    /// An error resulting from the use of a parameter not availible for a specific vocabulary list
    VocabularyError((String, String)),
    /// An error resulting from the use of a parameter not intended for the specified endpoint
    EndPointError((String, String)),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReqwestError(err) => write!(f, "{}", err),
            Self::SerdeError(err) => write!(f, "{}", err),
            Self::VocabularyError((lang, param)) => write!(
                f,
                "Error: The parameter {} is not yet supported for {}",
                param, lang
            ),
            Self::EndPointError((endpoint, param)) => write!(
                f,
                "Error: The parameter {} is not supported for {}",
                param, endpoint
            ),
        }
    }
}

impl error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeError(error)
    }
}
