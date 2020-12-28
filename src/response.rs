use crate::Result;
use serde::Deserialize;

/// This struct represents each word and its associated data in the response.
/// It is constructed when parsing a [Response](Response) with the method list().
/// Note that all optional values can still be None even if the proper flag
/// is set
#[derive(Debug, PartialEq)]
pub struct WordElement {
    /// The word returned based on the search parameters
    pub word: String,
    /// A score which ranks the word based on how well it fit the provided parameters.
    /// Note that by default the words are ranked by score from highest to lowest
    pub score: usize,
    /// The number of syllables the word has. This will only have a value if
    /// the meta data flag [SyllableCount](crate::MetaDataFlag::SyllableCount) is set
    pub num_syllables: Option<usize>,
    /// The part(s) of speech a word can be. This will only have a value if
    /// the meta data flag [PartsOfSpeech](crate::MetaDataFlag::PartsOfSpeech) is set
    pub parts_of_speech: Option<Vec<PartOfSpeech>>,
    /// The pronunciation of the word. This will only have a value if
    /// the meta data flag [Pronunciation](crate::MetaDataFlag::Pronunciation) is set.
    /// If an IPA pronuncation is available, it takes precedence as it is optional
    pub pronunciation: Option<String>,
    /// The frequency of a word based on how many times the word is used per 1,000,000
    /// words of text. This will only have a value if the meta data flag
    /// [WordFrequency](crate::MetaDataFlag::WordFrequency) is set
    pub frequency: Option<f32>,
    /// Definitions of a word and the associated part of speech with its use. This will only
    /// have a value if the meta data flag [Definitions](crate::MetaDataFlag::Definitions) is set
    pub definitions: Option<Vec<Definition>>,
}

/// A struct representing a word definition
#[derive(Debug, PartialEq)]
pub struct Definition {
    /// The part of speech associated with the definition
    pub part_of_speech: Option<PartOfSpeech>,
    /// The definition itself
    pub definition: String,
}

/// A struct representing a response from a request.
/// This can be parsed into a word list using the list() method
#[derive(Debug)]
pub struct Response {
    json: String,
}

/// An enum representing all possible parts of speech returned from the api
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PartOfSpeech {
    /// Noun
    Noun, //n
    /// Adjective
    Adjective, //adj
    /// Adverb
    Adverb, //adv
    /// Verb
    Verb, //v
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DatamuseWordObject {
    word: String,
    score: usize,
    num_syllables: Option<usize>,
    tags: Option<Vec<String>>,
    defs: Option<Vec<String>>,
}

impl Response {
    /// Parses the response into a list of word elements
    pub fn list(&self) -> Result<Vec<WordElement>> {
        parse_response(&self.json)
    }

    pub(crate) fn new(json: String) -> Response {
        Response { json }
    }
}

impl PartOfSpeech {
    fn from_str(pos: &str) -> Option<Self> {
        match pos {
            "n" => Some(Self::Noun),
            "adj" => Some(Self::Adjective),
            "adv" => Some(Self::Adverb),
            "v" => Some(Self::Verb),
            _ => None, //Also catches undefined option "u"
        }
    }
}

fn parse_response(response: &str) -> Result<Vec<WordElement>> {
    let word_list: Vec<DatamuseWordObject> = serde_json::from_str(response)?;
    let mut converted_word_list: Vec<WordElement> = Vec::new();

    for word in word_list {
        converted_word_list.push(word_obj_to_word_elem(word));
    }

    Ok(converted_word_list)
}

fn word_obj_to_word_elem(word_obj: DatamuseWordObject) -> WordElement {
    let word = word_obj.word;
    let score = word_obj.score;
    let num_syllables = word_obj.num_syllables;

    let mut parts_of_speech: Vec<PartOfSpeech> = Vec::new();
    let mut pronunciation = None;
    let mut frequency = None;

    if let Some(tags) = word_obj.tags {
        for tag in tags {
            let parts: Vec<&str> = tag.split(':').collect();

            match parts[0] {
                "f" => {
                    if parts.len() == 2 {
                        frequency = match parts[1].parse() {
                            Ok(val) => Some(val),
                            Err(_) => None,
                        }
                    }
                }
                "pron" => {
                    if let None = pronunciation {
                        //If pronunciation already has a value ignore b/c of ipa
                        if parts.len() == 2 {
                            pronunciation = Some(parts[1].to_string());
                        }
                    }
                }
                "ipa_pron" => {
                    if parts.len() == 2 {
                        pronunciation = Some(parts[1].to_string());
                    }
                }
                val => match PartOfSpeech::from_str(&val) {
                    Some(val) => parts_of_speech.push(val),
                    None => continue,
                },
            }
        }
    }

    let pos;
    if parts_of_speech.len() > 0 {
        pos = Some(parts_of_speech);
    } else {
        pos = None;
    }
    let parts_of_speech = pos;

    let mut definitions = None;
    if let Some(defs) = word_obj.defs {
        if defs.len() > 0 {
            let mut def_list: Vec<Definition> = Vec::new();

            for def in defs {
                let parts: Vec<&str> = def.split('\t').collect();

                if parts.len() == 2 {
                    let pos = PartOfSpeech::from_str(&parts[0]);
                    def_list.push(Definition {
                        part_of_speech: pos,
                        definition: parts[1].to_string(),
                    });
                }
            }

            definitions = Some(def_list);
        }
    }

    WordElement {
        word,
        score,
        num_syllables,
        parts_of_speech,
        pronunciation,
        frequency,
        definitions,
    }
}

#[cfg(test)]
mod tests {
    use super::DatamuseWordObject;
    use crate::{Definition, PartOfSpeech, WordElement};

    #[test]
    fn word_obj_to_word_elem() {
        let word_obj = DatamuseWordObject {
            word: String::from("cow"),
            score: 2168,
            num_syllables: Some(1),
            tags: Some(vec![
                String::from("n"),
                String::from("pron:K AW1 "),
                String::from("f:16.567268"),
            ]),
            defs: Some(vec![
                String::from("n\tmature female of mammals of which the male is called `bull'"),
                String::from("n\tfemale of domestic cattle"),
            ]),
        };

        let actual = super::word_obj_to_word_elem(word_obj);

        let expected = WordElement {
            word: String::from("cow"),
            score: 2168,
            num_syllables: Some(1),
            parts_of_speech: Some(vec![PartOfSpeech::Noun]),
            pronunciation: Some(String::from("K AW1 ")),
            frequency: Some(16.567268),
            definitions: Some(vec![
                Definition {
                    part_of_speech: Some(PartOfSpeech::Noun),
                    definition: String::from(
                        "mature female of mammals of which the male is called `bull'",
                    ),
                },
                Definition {
                    part_of_speech: Some(PartOfSpeech::Noun),
                    definition: String::from("female of domestic cattle"),
                },
            ]),
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn json_to_word_elem() {
        let json = r#"
        [
            {
                "word":"milk",
                "score":2168,
                "numSyllables":1,
                "tags": [],
                "defs": []
            },
            {
                "word":"cow",
                "score":2168,
                "numSyllables":1,
                "tags": [
                    "n",
                    "pron:K AW1 ",
                    "f:16.567268"
                ],
                "defs": [
                    "n\tmature female of mammals of which the male is called `bull'",
                    "n\tfemale of domestic cattle"
                ]
            }
        ]
        "#;

        let actual = super::parse_response(json).unwrap();

        let expected1 = WordElement {
            word: String::from("milk"),
            score: 2168,
            num_syllables: Some(1),
            parts_of_speech: None,
            pronunciation: None,
            frequency: None,
            definitions: None,
        };

        let expected2 = WordElement {
            word: String::from("cow"),
            score: 2168,
            num_syllables: Some(1),
            parts_of_speech: Some(vec![PartOfSpeech::Noun]),
            pronunciation: Some(String::from("K AW1 ")),
            frequency: Some(16.567268),
            definitions: Some(vec![
                Definition {
                    part_of_speech: Some(PartOfSpeech::Noun),
                    definition: String::from(
                        "mature female of mammals of which the male is called `bull'",
                    ),
                },
                Definition {
                    part_of_speech: Some(PartOfSpeech::Noun),
                    definition: String::from("female of domestic cattle"),
                },
            ]),
        };

        assert_eq!(expected1, actual[0]);
        assert_eq!(expected2, actual[1]);
    }
}
