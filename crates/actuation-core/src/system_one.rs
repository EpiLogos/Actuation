//! General typed System One questions and validated determinations.
//!
//! Provider probabilities are model observations, not source standing, authority
//! or proof. Document, Wiki, Factory and QL practices supply their own questions.
//! Wire basis: https://docs.typesafe.ai/api, checked 2026-09-22.
use crate::{Error, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};
use std::{collections::BTreeMap, fmt};

/// A native material bound, not a claim about the provider's question limit.
pub const MAX_QUESTIONS: usize = 256;
const EPSILON: f64 = 0.0001;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Question {
    Noul {
        instructions: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<BTreeMap<String, Value>>,
    },
    Choice {
        instructions: Value,
        criteria: BTreeMap<String, Value>,
    },
    Score {
        instructions: Value,
        criteria: Vec<Value>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemOneRequest {
    pub state: Value,
    pub model: String,
    pub questions: BTreeMap<String, Question>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Answer {
    Noul {
        noul: f64,
    },
    Choice {
        choice: String,
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
    Score {
        score: f64,
        probabilities: BTreeMap<String, f64>,
        // Structured criteria are returned unchanged, not flattened to text.
        legend: BTreeMap<String, Value>,
        confidence: f64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemOneUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemOneResponse {
    pub model: String,
    pub answers: BTreeMap<String, Answer>,
    pub usage: SystemOneUsage,
}

/// Only `validate_response` constructs this result. Deserializing a plausible
/// response is deliberately insufficient to produce a determination.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SystemOneDetermination(SystemOneResponse);
impl SystemOneDetermination {
    pub fn response(&self) -> &SystemOneResponse {
        &self.0
    }
    pub fn into_response(self) -> SystemOneResponse {
        self.0
    }
}

fn described(value: &Value) -> bool {
    matches!(value, Value::String(_) | Value::Object(_) | Value::Array(_))
}
// Advanced System One EntryType allows an explicit null. State itself still
// requires text/object/array, and serde still requires the instructions field.
fn entry(value: &Value) -> bool { value.is_null() || described(value) }
fn identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
pub fn versioned_jev(model: &str) -> bool {
    model.strip_prefix("jev-").is_some_and(|version| {
        let parts: Vec<_> = version.split('.').collect();
        parts.len() == 3
            && parts.iter().all(|part| {
                !part.is_empty() && part.len() <= 10 && part.bytes().all(|b| b.is_ascii_digit())
            })
    })
}
fn probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
fn distribution(values: &BTreeMap<String, f64>, keys: &[String]) -> Result<()> {
    if values.len() != keys.len()
        || !keys.iter().all(|key| values.contains_key(key))
        || !values.values().all(|p| probability(*p))
        || (values.values().sum::<f64>() - 1.0).abs() > EPSILON
    {
        return Err(Error::new("system_one.invalid_distribution"));
    }
    Ok(())
}

impl SystemOneRequest {
    pub fn validate(&self) -> Result<()> {
        if !described(&self.state) {
            return Err(Error::new("system_one.state_requires_text_object_or_array"));
        }
        if !versioned_jev(&self.model)
            && !matches!(self.model.as_str(), "jev-latest" | "jev-preview")
        {
            return Err(Error::new("system_one.invalid_model_selection"));
        }
        if self.questions.is_empty() || self.questions.len() > MAX_QUESTIONS {
            return Err(Error::new("system_one.question_count_out_of_bounds"));
        }
        for (id, question) in &self.questions {
            if !identifier(id) {
                return Err(Error::new("system_one.invalid_question_id"));
            }
            let valid = match question {
                Question::Noul { instructions, criteria } => {
                    entry(instructions)
                        && criteria.as_ref().is_none_or(|c| {
                            c.iter().all(|(key, value)| {
                                    matches!(key.as_str(), "true" | "false") && entry(value)
                                })
                        })
                }
                Question::Choice { instructions, criteria } => {
                    entry(instructions)
                        && !criteria.is_empty()
                        && criteria.len() <= 255
                        && criteria.iter().all(|(key, value)| {
                            identifier(key) && entry(value)
                        })
                }
                Question::Score { instructions, criteria } => {
                    entry(instructions)
                        && (2..=10).contains(&criteria.len())
                        && criteria.iter().all(entry)
                }
            };
            if !valid {
                return Err(Error::new("system_one.invalid_question"));
            }
        }
        Ok(())
    }

    pub fn validate_response(&self, response: SystemOneResponse) -> Result<SystemOneDetermination> {
        self.validate()?;
        if !versioned_jev(&response.model)
            || (versioned_jev(&self.model) && response.model != self.model)
        {
            return Err(Error::new("system_one.returned_model_mismatch_or_unversioned"));
        }
        if response.answers.len() != self.questions.len()
            || !self.questions.keys().all(|key| response.answers.contains_key(key))
        {
            return Err(Error::new("system_one.incomplete_or_foreign_answers"));
        }
        for (key, question) in &self.questions {
            match (question, &response.answers[key]) {
                (Question::Noul { .. }, Answer::Noul { noul }) if probability(*noul) => {}
                (
                    Question::Choice { criteria, .. },
                    Answer::Choice { choice, probabilities, confidence },
                ) => {
                    distribution(probabilities, &criteria.keys().cloned().collect::<Vec<_>>())?;
                    let selected = probabilities.get(choice).ok_or_else(|| {
                        Error::new("system_one.choice_not_in_criteria")
                    })?;
                    if !probability(*confidence)
                        || probabilities.values().any(|p| p > &(selected + EPSILON))
                    {
                        return Err(Error::new("system_one.choice_or_confidence_inconsistent"));
                    }
                }
                (
                    Question::Score { criteria, .. },
                    Answer::Score { score, probabilities, legend, confidence },
                ) => {
                    let keys: Vec<_> = (0..criteria.len()).map(|i| i.to_string()).collect();
                    distribution(probabilities, &keys)?;
                    if legend.len() != criteria.len()
                        || criteria.iter().enumerate().any(|(i, value)| {
                            legend.get(&i.to_string()) != Some(value)
                        })
                    {
                        return Err(Error::new("system_one.score_legend_changed"));
                    }
                    let expected: f64 = keys.iter().enumerate()
                        .map(|(i, key)| i as f64 * probabilities[key]).sum();
                    if !score.is_finite()
                        || !(0.0..=(criteria.len() - 1) as f64).contains(score)
                        || (score - expected).abs() > EPSILON
                        || !probability(*confidence)
                    {
                        return Err(Error::new("system_one.score_or_confidence_inconsistent"));
                    }
                }
                _ => return Err(Error::new("system_one.answer_type_or_probability_invalid")),
            }
        }
        Ok(SystemOneDetermination(response))
    }

    pub fn parse_response(&self, bytes: &[u8]) -> Result<SystemOneDetermination> {
        self.validate_response(serde_json::from_value(unique_json(bytes)?)?)
    }
}

/// Refuse duplicate keys before a JSON map could silently replace a question,
/// answer, legend or usage value. Also reject trailing data and non-finite JSON.
pub fn unique_json(bytes: &[u8]) -> Result<Value> {
    struct Unique(Value);
    impl<'de> Deserialize<'de> for Unique {
        fn deserialize<D: Deserializer<'de>>(de: D) -> std::result::Result<Self, D::Error> {
            struct Visitor;
            impl<'de> de::Visitor<'de> for Visitor {
                type Value = Unique;
                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("JSON with unique object keys")
                }
                fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Bool(v)))
                }
                fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Number(v.into())))
                }
                fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Number(v.into())))
                }
                fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                    Number::from_f64(v).map(|n| Unique(Value::Number(n)))
                        .ok_or_else(|| E::custom("non-finite JSON number"))
                }
                fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::String(v.to_owned())))
                }
                fn visit_unit<E: de::Error>(self) -> std::result::Result<Unique, E> {
                    Ok(Unique(Value::Null))
                }
                fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<Unique, A::Error> {
                    let mut values = Vec::new();
                    while let Some(Unique(value)) = seq.next_element()? { values.push(value); }
                    Ok(Unique(Value::Array(values)))
                }
                fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> std::result::Result<Unique, A::Error> {
                    let mut values = Map::new();
                    while let Some((key, Unique(value))) = map.next_entry::<String, Unique>()? {
                        if values.insert(key, value).is_some() {
                            return Err(de::Error::custom("system_one.duplicate_json_key"));
                        }
                    }
                    Ok(Unique(Value::Object(values)))
                }
            }
            de.deserialize_any(Visitor)
        }
    }
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let Unique(value) = Unique::deserialize(&mut parser)?;
    parser.end()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request() -> SystemOneRequest {
        serde_json::from_value(json!({
            "model":"jev-1.13.0", "state":{"weather":"dry", "sail":"damaged"},
            "questions":{
                "repair":{"type":"noul","instructions":"Is repair required?"},
                "route":{"type":"choice","instructions":{"question":"Choose safe transport"},
                    "criteria":{"walk":null,"sail":{"condition":"safe sail"}}},
                "risk":{"type":"score","instructions":"Rate the risk",
                    "criteria":[{"description":"low"},["high","damage"]]}
            }
        })).unwrap()
    }
    fn response() -> Value {
        json!({"model":"jev-1.13.0", "usage":{"input_tokens":75,"output_tokens":21},
            "answers":{
                "repair":{"type":"noul","noul":0.99},
                "route":{"type":"choice","choice":"walk","probabilities":{"walk":0.9,"sail":0.1},"confidence":0.6},
                "risk":{"type":"score","score":0.8,"probabilities":{"0":0.2,"1":0.8},
                    "legend":{"0":{"description":"low"},"1":["high","damage"]},"confidence":0.3}
            }})
    }
    fn check(value: Value) -> Result<SystemOneDetermination> {
        request().parse_response(&serde_json::to_vec(&value).unwrap())
    }
    #[test]
    fn system_one_general_questions_and_structured_legend() {
        let answer = check(response()).unwrap();
        assert_eq!(answer.response().model, "jev-1.13.0");
        assert_eq!(answer.response().usage.input_tokens, 75);
        assert_eq!(answer.response().answers.len(), 3);
    }
    #[test]
    fn system_one_omission_is_never_a_determination() {
        for key in ["repair", "route", "risk"] {
            let mut v = response(); v["answers"].as_object_mut().unwrap().remove(key);
            assert!(check(v).is_err());
        }
        let mut v = response(); v["answers"]["foreign"] = json!({"type":"noul","noul":1.0});
        assert!(check(v).is_err());
    }
    #[test]
    fn system_one_distribution_type_legend_and_usage_negatives() {
        for (pointer, value) in [
            ("/answers/repair/noul", json!(1.01)),
            ("/answers/repair/type", json!("choice")),
            ("/answers/route/choice", json!("sail")),
            ("/answers/route/probabilities/walk", json!(0.8)),
            ("/answers/route/confidence", json!(-0.1)),
            ("/answers/risk/score", json!(0.2)),
            ("/answers/risk/legend/0", json!("low")),
            ("/usage/input_tokens", json!(-1)),
            ("/usage/output_tokens", json!(null)),
            ("/model", json!("jev-latest")),
            ("/model", json!("jev-1.12.0")),
        ] {
            let mut v = response(); *v.pointer_mut(pointer).unwrap() = value;
            assert!(check(v).is_err(), "{pointer}");
        }
        let mut v = response(); v["answers"]["route"]["probabilities"].as_object_mut().unwrap().remove("sail");
        assert!(check(v).is_err());
    }
    #[test]
    fn system_one_alias_retains_actual_returned_version() {
        let mut req = request(); req.model = "jev-latest".into();
        assert!(req.parse_response(&serde_json::to_vec(&response()).unwrap()).is_ok());
    }
    #[test]
    fn system_one_refuses_duplicate_and_trailing_json() {
        for bytes in [br#"{"a":1,"a":2}"#.as_slice(), br#"{"x":{"a":0,"a":1}}"#, b"{} {}", b"NaN"] {
            assert!(unique_json(bytes).is_err());
        }
    }
    #[test]
    fn system_one_request_shape_limits() {
        let mut req = request(); req.state = Value::Null; assert!(req.validate().is_err());
        let mut req = request(); req.questions.clear(); assert!(req.validate().is_err());
        let mut req = request(); req.model = "jev-latest\n".into(); assert!(req.validate().is_err());
        let mut req = request(); req.questions.insert("empty-scale".into(), Question::Score {
            instructions: json!("rate"), criteria: vec![json!("only")]
        }); assert!(req.validate().is_err());
    }
}

#[cfg(test)]
mod nullable_entry_regression {
    use super::*;
    use serde_json::json;
    #[test]
    fn explicit_null_entries_are_preserved_but_missing_instructions_are_refused() {
        let request: SystemOneRequest = serde_json::from_value(json!({"model":"jev-1.13.0","state":"test",
            "questions": {
                "n":{"type":"noul","instructions":null,"criteria":{"true":null,"false":null}},
                "c":{"type":"choice","instructions":null,"criteria":{"one":null}},
                "s":{"type":"score","instructions":null,"criteria":[null,{"meaning":"full"}]}
            }})).unwrap();
        request.validate().unwrap();
        let answer = request.parse_response(br#"{"model":"jev-1.13.0","answers":{
            "n":{"type":"noul","noul":0.5},
            "c":{"type":"choice","choice":"one","probabilities":{"one":1.0},"confidence":1.0},
            "s":{"type":"score","score":0.7,"probabilities":{"0":0.3,"1":0.7},"legend":{"0":null,"1":{"meaning":"full"}},"confidence":0.2}
        },"usage":{"input_tokens":0,"output_tokens":0}}"#).unwrap();
        assert_eq!(answer.response().usage.input_tokens,0);
        let mut value=serde_json::to_value(request).unwrap();
        value["questions"]["n"].as_object_mut().unwrap().remove("instructions");
        assert!(serde_json::from_value::<SystemOneRequest>(value).is_err());
    }
}
