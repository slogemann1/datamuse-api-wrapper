//TODO: add examples
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

extern crate reqwest;
extern crate serde_json;
extern crate serde;

use std::fmt::{ self, Display, Formatter };
use std::error;
use std::result;

mod request;
mod response;

pub use request::*;
pub use response::*;

/// This struct represents the client which can be used to make requests
/// to the Datamuse api. Requests can be created using the new_query() method
#[derive(Debug)]
pub struct DatamuseClient {
    client: reqwest::Client
}

impl DatamuseClient {
    /// Returns a new DatamuseClient struct
    pub fn new() -> Self {
        DatamuseClient {
            client: reqwest::Client::new()
        }
    }

    /// Returns a new [RequestBuilder](request::RequestBuilder) struct with which requests can be created
    /// and later sent. As parameters the vocabulary set and endpoint of the request are required. See
    /// their individual documentations for more information
    pub fn new_query<'a>(&'a self, vocabulary: Vocabulary, endpoint: EndPoint) -> RequestBuilder<'a> {
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
            Self::VocabularyError((lang, param)) => write!(f, "Error: The parameter {} is not yet supported for {}", param, lang),
            Self::EndPointError((endpoint, param)) => write!(f, "Error: The parameter {} is not supported for {}", param, endpoint)
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