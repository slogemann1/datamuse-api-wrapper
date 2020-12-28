use crate::response::{Response, WordElement};
use crate::{DatamuseClient, Error, Result};
use reqwest;
use std::fmt::{self, Display, Formatter};

/// Use this struct to build requests to send to the Datamuse api.
/// This request can be sent either by building it into a Request with build()
/// and then using the send() method on the resulting Request or using send() to
/// send it directly. Note that not all parameters can be used for each vocabulary
/// and endpoint
#[derive(Debug)]
pub struct RequestBuilder<'a> {
    client: &'a DatamuseClient,
    endpoint: EndPoint,
    vocabulary: Vocabulary,
    parameters: Vec<Parameter>,
    topics: Vec<String>, //Makes adding topics make easier, later added to parameters
    meta_data_flags: Vec<MetaDataFlag>, //Same issue as topics
}

/// This struct represents a built request that can be sent using the send() method
#[derive(Debug)]
pub struct Request<'a> {
    client: &'a reqwest::Client,
    request: reqwest::Request,
}

/// This enum represents the different endpoints of the Datamuse api.
/// The "words" endpoint returns word lists based on a set of parameters,
/// whereas the "suggest" endpoint returns suggestions for words based on a
/// hint string (autocomplete).
/// For more detailed information visit the [Datamuse website](https://www.datamuse.com/api/)
#[derive(Clone, Copy, Debug)]
pub enum EndPoint {
    /// The "words" endpoint (the official endpoint is also "/words")
    Words,
    /// The "suggest" endpoint (the official endpoint is "/sug")
    Suggest,
}

/// This enum represents the different vocabulary lists which can be used as
/// a source for the requests. There are currently two language options
/// (English or Spanish) and an alternative English option from wikipedia.
/// For more detailed information visit the [Datamuse website](https://www.datamuse.com/api/)
#[derive(Clone, Copy, Debug)]
pub enum Vocabulary {
    /// The default vocabulary list with 550,000 words
    English,
    /// The Spanish vocabulary list with 500,000 words
    Spanish,
    /// The alternative English vocabulary list with 6 million words
    EnglishWiki,
}

/// This enum represents the different possibilites the "Related" parameter can take.
/// These parameters can be combined in any possible configuration, although very specific
/// queries can limit results. Each option is shortly explained below.
/// For more detailed information for each type visit the [Datamuse website](https://www.datamuse.com/api/)
#[derive(Clone, Copy, Debug)]
pub enum RelatedType {
    /// This parameter returns nouns that are typically modified by the given adjective
    NounModifiedBy,
    /// This parameter returns adjectives that typically modify by the given noun
    AdjectiveModifier,
    /// This parameter returns synonyms for the given word
    Synonym,
    /// This parameter returns associated words for the given word
    Trigger,
    /// This parameter returns antonyms for the given word
    Antonym,
    /// This parameter returns the kind of which a more specific word is
    KindOf,
    /// This parameter returns a more specific kind of the given category word (opposite of KindOf)
    MoreGeneral,
    /// This parameter returns words that describe things which the given word is comprised of
    Comprises,
    /// This parameter returns words that describe things which the given word is a part of (opposite of Comprises)
    PartOf,
    /// This parameter returns words that are typically found after the given word
    Follower,
    /// This parameter returns words that are typically found before the given word
    Predecessor,
    /// This parameter returns words that rhyme with the given word
    Rhyme,
    /// This parameter returns words that almost rhyme with the given word
    ApproximateRhyme,
    /// This parameter returns words that sound like the given word
    Homophones,
    /// This parameter returns words which have matching consonants but differing vowels from the given word
    ConsonantMatch,
}

/// This enum represents the various flags which can be set for retrieving metadata for each word.
/// These metadata flags can be combined in any manner. Each is shortly described below
#[derive(Clone, Copy, Debug)]
pub enum MetaDataFlag {
    /// Provides definitions for each of the words in the response
    Definitions,
    /// Provides what type of speech each of the words in the response is
    PartsOfSpeech,
    /// Provides how many syllables each of the words in the response has
    SyllableCount,
    /// Provides pronunciations for each of the words in the response based on
    /// the given pronunciation format
    Pronunciation(PronunciationFormat),
    /// Provides how frequently each of the words in the response is found
    WordFrequency,
}

/// This enum represents the ways pronunciations returned by the "Pronunciation" metadata flag
/// can be given
#[derive(Clone, Copy, Debug)]
pub enum PronunciationFormat {
    /// The [ARPABET](https://en.wikipedia.org/wiki/ARPABET) pronunciation format
    Arpabet,
    /// The [International Phonetic Alphabet](https://en.wikipedia.org/wiki/International_Phonetic_Alphabet) pronunciation format
    Ipa,
}

#[derive(Clone, Debug)]
struct RelatedTypeHolder {
    related_type: RelatedType,
    value: String,
}

#[derive(Clone, Debug)]
enum Parameter {
    MeansLike(String),
    SoundsLike(String),
    SpelledLike(String),
    Related(RelatedTypeHolder),
    Topics(Vec<String>),
    LeftContext(String),
    RightContext(String),
    MaxResults(u16), //Also supported for sug endpoint
    MetaData(Vec<MetaDataFlag>),
    HintString(String), //Only supported for sug endpoint
}

impl<'a> RequestBuilder<'a> {
    /// Sets a query parameter for words which have a similar meaning to the given word
    pub fn means_like(mut self, word: &str) -> Self {
        self.parameters
            .push(Parameter::MeansLike(String::from(word)));

        self
    }

    /// Sets a query parameter for words which sound similar to the given word
    pub fn sounds_like(mut self, word: &str) -> Self {
        self.parameters
            .push(Parameter::SoundsLike(String::from(word)));

        self
    }

    /// Sets a query parameter for words which have a similar spelling to the given word.
    /// This parameter allows for wildcard charcters with '?' matching a single letter and
    /// '*' matching any number of letters
    pub fn spelled_like(mut self, word: &str) -> Self {
        self.parameters
            .push(Parameter::SpelledLike(String::from(word)));

        self
    }

    /// Sets a query parameter for words which are related to the given word.
    /// The various options for relations are given in the [RelatedType](RelatedType) enum.
    /// See its documentation for more information on the options.
    /// Note that this is currently **not available** for the Spanish vocabulary set
    pub fn related(mut self, rel_type: RelatedType, word: &str) -> Self {
        self.parameters.push(Parameter::Related(RelatedTypeHolder {
            related_type: rel_type,
            value: String::from(word),
        }));

        self
    }

    /// Sets a query parameter for words which fall under the topic of the given word.
    /// Multiple topics can be specified at once, however requests are limited to five
    /// topics and as such any specified over this limit will be ignored
    pub fn add_topic(mut self, word: &str) -> Self {
        self.topics.push(String::from(word));

        self
    }

    /// Sets a query parameter to refer to the word directly before the main query term
    pub fn left_context(mut self, word: &str) -> Self {
        self.parameters
            .push(Parameter::LeftContext(String::from(word)));

        self
    }

    /// Sets a query parameter to refer to the word directly after the main query term
    pub fn right_context(mut self, word: &str) -> Self {
        self.parameters
            .push(Parameter::RightContext(String::from(word)));

        self
    }

    /// The maximum number of results that should be returned. By default this is set to 100
    /// and it can be increased to a maximum of 1000. This parameter is also **allowed** for the
    /// "suggest" endpoint
    pub fn max_results(mut self, maximum: u16) -> Self {
        self.parameters.push(Parameter::MaxResults(maximum));

        self
    }

    /// Sets a metadata flag to specify data returned with each word.
    /// The various options for flags are given in the [MetaDataFlag](MetaDataFlag) enum.
    /// See its documentation for more information on the options
    pub fn meta_data(mut self, flag: MetaDataFlag) -> Self {
        self.meta_data_flags.push(flag);

        self
    }

    /// Sets the hint string for the "suggest" endpoint. Note that this is
    /// **not allowed** for the "words" endpoint
    pub fn hint_string(mut self, hint: &str) -> Self {
        self.parameters
            .push(Parameter::HintString(String::from(hint)));

        self
    }

    /// Converts the RequestBuilder into a Request which can be executed by calling the send()
    /// method on it. This method will return an error if any of the given parameters have not been
    /// used correctly or the underlying call to reqwest to build the request fails
    pub fn build(&self) -> Result<Request> {
        let mut params_list: Vec<(String, String)> = Vec::new();
        let mut parameters = self.parameters.clone();

        if self.topics.len() > 0 {
            parameters.push(Parameter::Topics(self.topics.clone()));
        }

        if self.meta_data_flags.len() > 0 {
            parameters.push(Parameter::MetaData(self.meta_data_flags.clone()));

            for flag in self.meta_data_flags.clone() {
                if let MetaDataFlag::Pronunciation(PronunciationFormat::Ipa) = flag {
                    params_list.push((String::from("ipa"), 1.to_string()));
                }
            }
        }

        let vocab_params = self.vocabulary.build();
        if let Some(val) = vocab_params {
            params_list.push(val);
        }

        for param in parameters {
            params_list.push(param.build(&self.vocabulary, &self.endpoint)?);
        }

        let request = self
            .client
            .client
            .get(&format!(
                "https://api.datamuse.com/{}",
                self.endpoint.get_string()
            ))
            .query(&params_list)
            .build()?;

        Ok(Request {
            request,
            client: &self.client.client,
        })
    }

    /// A convenience method to build and send the request in one step. The resulting
    /// response can be parsed with its list() method
    pub async fn send(&self) -> Result<Response> {
        self.build()?.send().await
    }

    /// A convenience method to build and send the request as well as parse the json in one step
    pub async fn list(&self) -> Result<Vec<WordElement>> {
        self.send().await?.list()
    }

    pub(crate) fn new(
        client: &'a DatamuseClient,
        vocabulary: Vocabulary,
        endpoint: EndPoint,
    ) -> Self {
        RequestBuilder {
            client,
            endpoint,
            vocabulary,
            parameters: Vec::new(),
            topics: Vec::new(),
            meta_data_flags: Vec::new(),
        }
    }
}

impl<'a> Request<'a> {
    /// Sends the built request and returns the response. This response can later be parsed with its
    /// list() method
    pub async fn send(self) -> Result<Response> {
        let json = self.client.execute(self.request).await?.text().await?;
        Ok(Response::new(json))
    }
}

impl Parameter {
    fn build(&self, vocab: &Vocabulary, endpoint: &EndPoint) -> Result<(String, String)> {
        if let Parameter::Related(_) = self {
            //Error for using related with spanish vocabulary
            if let Vocabulary::Spanish = vocab {
                return Err(Error::VocabularyError((
                    String::from("Spanish"),
                    String::from("Related"),
                )));
            }
        }

        if let EndPoint::Words = endpoint {
            //Error for using hint string for the words endpoint
            if let Parameter::HintString(_) = self {
                return Err(Error::EndPointError((
                    String::from("Words"),
                    String::from("HintString"),
                )));
            }
        }

        if let EndPoint::Suggest = endpoint {
            match self {
                Parameter::MaxResults(_) => (),
                Parameter::HintString(_) => (),
                val => {
                    return Err(Error::EndPointError((
                        String::from("Suggest"),
                        format!("{}", val),
                    )));
                }
            }
        }

        let param = match self {
            Self::MeansLike(val) => (String::from("ml"), val.clone()),
            Self::SoundsLike(val) => (String::from("sl"), val.clone()),
            Self::SpelledLike(val) => (String::from("sp"), val.clone()),
            Self::Related(val) => (format!("rel_{}", val.get_type_identifier()), val.get_word()),
            Self::Topics(topic_list) => {
                let mut topics_concat = String::from("");
                let mut len = topic_list.len();

                if len > 5 {
                    len = 5;
                }

                let mut i = 0;
                while i < len - 1 {
                    topics_concat = topics_concat + &topic_list[i];
                    topics_concat.push(',');
                    i += 1;
                }
                topics_concat = topics_concat + &topic_list[len - 1];

                (String::from("topics"), topics_concat)
            }
            Self::LeftContext(val) => (String::from("lc"), val.clone()),
            Self::RightContext(val) => (String::from("rc"), val.clone()),
            Self::MaxResults(val) => (String::from("max"), val.to_string()),
            Self::MetaData(flags) => {
                let mut flags_concat = String::from("");
                for flag in flags {
                    flags_concat.push(flag.get_letter_identifier());
                }

                (String::from("md"), flags_concat)
            }
            Self::HintString(val) => (String::from("s"), val.clone()),
        };

        Ok(param)
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::MeansLike(_) => "MeansLike",
            Self::SoundsLike(_) => "SoundsLike",
            Self::SpelledLike(_) => "SpelledLike",
            Self::Related(_) => "Related",
            Self::Topics(_) => "Topic",
            Self::LeftContext(_) => "LeftContext",
            Self::RightContext(_) => "RightContext",
            Self::MaxResults(_) => "MaxResults",
            Self::MetaData(_) => "MetaData",
            Self::HintString(_) => "HintString",
        };

        write!(f, "{}", name)
    }
}

impl RelatedTypeHolder {
    fn get_type_identifier(&self) -> String {
        match self.related_type {
            RelatedType::NounModifiedBy => String::from("jja"),
            RelatedType::AdjectiveModifier => String::from("jjb"),
            RelatedType::Synonym => String::from("syn"),
            RelatedType::Trigger => String::from("trg"),
            RelatedType::Antonym => String::from("ant"),
            RelatedType::KindOf => String::from("spc"),
            RelatedType::MoreGeneral => String::from("gen"),
            RelatedType::Comprises => String::from("com"),
            RelatedType::PartOf => String::from("par"),
            RelatedType::Follower => String::from("bga"),
            RelatedType::Predecessor => String::from("bgb"),
            RelatedType::Rhyme => String::from("rhy"),
            RelatedType::ApproximateRhyme => String::from("nry"),
            RelatedType::Homophones => String::from("hom"),
            RelatedType::ConsonantMatch => String::from("cns"),
        }
    }

    fn get_word(&self) -> String {
        self.value.clone()
    }
}

impl MetaDataFlag {
    fn get_letter_identifier(&self) -> char {
        match self {
            Self::Definitions => 'd',
            Self::PartsOfSpeech => 'p',
            Self::SyllableCount => 's',
            Self::Pronunciation(_) => 'r',
            Self::WordFrequency => 'f',
        }
    }
}

impl EndPoint {
    fn get_string(&self) -> String {
        match self {
            Self::Words => String::from("words"),
            Self::Suggest => String::from("sug"),
        }
    }
}

impl Vocabulary {
    fn build(&self) -> Option<(String, String)> {
        match self {
            Vocabulary::Spanish => Some((String::from("v"), String::from("es"))),
            Vocabulary::EnglishWiki => Some((String::from("v"), String::from("enwiki"))),
            Vocabulary::English => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DatamuseClient, EndPoint, MetaDataFlag, PronunciationFormat, RelatedType, Vocabulary,
    };

    #[test]
    fn means_like_and_sounds_like() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .means_like("cap")
            .sounds_like("flat");

        assert_eq!(
            "https://api.datamuse.com/words?ml=cap&sl=flat",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn left_context_and_spelled_like() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .left_context("drink")
            .spelled_like("w*");

        assert_eq!(
            "https://api.datamuse.com/words?lc=drink&sp=w*",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn right_context_and_max_results() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .right_context("food")
            .max_results(500);

        assert_eq!(
            "https://api.datamuse.com/words?rc=food&max=500",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn topics_and_sounds_like() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .add_topic("color")
            .sounds_like("clue")
            .add_topic("sad");

        assert_eq!(
            "https://api.datamuse.com/words?sl=clue&topics=color%2Csad", //%2C = ','
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn suggest_endpoint() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Suggest)
            .hint_string("hel")
            .max_results(20);

        assert_eq!(
            "https://api.datamuse.com/sug?s=hel&max=20",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    #[should_panic]
    fn suggest_endpoint_fail() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Suggest)
            .add_topic("color");
        request.build().unwrap();
    }

    #[test]
    #[should_panic]
    fn words_endpoint_fail() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .add_topic("color")
            .hint_string("blu");
        request.build().unwrap();
    }

    #[test]
    #[should_panic]
    fn spanish_vocabulary_fail() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::Spanish, EndPoint::Words)
            .related(RelatedType::Trigger, "frutas")
            .sounds_like("manta");

        request.build().unwrap();
    }

    #[test]
    fn noun_and_adjective_modifiers() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::AdjectiveModifier, "food")
            .related(RelatedType::NounModifiedBy, "fresh");

        assert_eq!(
            "https://api.datamuse.com/words?rel_jjb=food&rel_jja=fresh",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn synonyms_and_triggers() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Synonym, "grass")
            .related(RelatedType::Trigger, "cow");

        assert_eq!(
            "https://api.datamuse.com/words?rel_syn=grass&rel_trg=cow",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn antonyms_and_consonant_match() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Antonym, "good")
            .related(RelatedType::ConsonantMatch, "bed");

        assert_eq!(
            "https://api.datamuse.com/words?rel_ant=good&rel_cns=bed",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn kind_of_and_more_general() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::KindOf, "wagon")
            .related(RelatedType::MoreGeneral, "vehicle");

        assert_eq!(
            "https://api.datamuse.com/words?rel_spc=wagon&rel_gen=vehicle",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn comprises_and_part_of() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Comprises, "car")
            .related(RelatedType::PartOf, "glass");

        assert_eq!(
            "https://api.datamuse.com/words?rel_com=car&rel_par=glass",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn follows_and_precedes() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Follower, "soda")
            .related(RelatedType::Predecessor, "drink");

        assert_eq!(
            "https://api.datamuse.com/words?rel_bga=soda&rel_bgb=drink",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn both_rhymes_and_homophones() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Rhyme, "cat")
            .related(RelatedType::Homophones, "mate")
            .related(RelatedType::ApproximateRhyme, "fate");

        assert_eq!(
            "https://api.datamuse.com/words?rel_rhy=cat&rel_hom=mate&rel_nry=fate",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn all_meta_data_flags() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Trigger, "cow")
            .meta_data(MetaDataFlag::Definitions)
            .meta_data(MetaDataFlag::PartsOfSpeech)
            .meta_data(MetaDataFlag::SyllableCount)
            .meta_data(MetaDataFlag::WordFrequency)
            .meta_data(MetaDataFlag::Pronunciation(PronunciationFormat::Arpabet));

        assert_eq!(
            "https://api.datamuse.com/words?rel_trg=cow&md=dpsfr",
            request.build().unwrap().request.url().as_str()
        );
    }

    #[test]
    fn pronunciation_ipa() {
        let client = DatamuseClient::new();
        let request = client
            .new_query(Vocabulary::English, EndPoint::Words)
            .related(RelatedType::Trigger, "soda")
            .meta_data(MetaDataFlag::Pronunciation(PronunciationFormat::Ipa));

        assert_eq!(
            "https://api.datamuse.com/words?ipa=1&rel_trg=soda&md=r",
            request.build().unwrap().request.url().as_str()
        );
    }
}
