use crate::{Error, Result};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// A built-in syllable transformation, indexed in the original TypeScript order.
/// Non-Hangul input is passed through, avoiding the original unchecked arithmetic
/// that accidentally produces jamo from Latin characters.
pub struct ChangeRule(pub usize);

impl ChangeRule {
    pub fn forward(&self, value: &str) -> Vec<String> {
        match self.0 {
            1 => standard(value, false),
            2 => forced(value, false),
            3 => standard(value, true),
            4 => forced(value, true),
            5 => one_way(value, false),
            6 => free_consonants(value, false),
            7 => vowel_reversal(value),
            8 => consonant_reversal(value),
            9 => free_consonants(value, true),
            10 => roblox(value, false),
            _ => vec![value.to_owned()],
        }
    }
    pub fn backward(&self, value: &str) -> Vec<String> {
        match self.0 {
            1 => standard(value, true),
            2 => forced_backward(value, false),
            3 => standard(value, false),
            4 => forced_backward(value, true),
            5 => one_way(value, true),
            10 => roblox(value, true),
            _ => self.forward(value),
        }
    }
}

const ONSETS: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];
const VOWELS: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];
const CODAS: [char; 27] = [
    'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ',
    'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// Disassemble a precomposed syllable or a compatibility jamo.
pub fn disassemble(value: &str) -> [Option<char>; 3] {
    let Some(character) = value.chars().next() else {
        return [None; 3];
    };
    let code = character as u32;
    if ('ㄱ'..='ㅎ').contains(&character) {
        return [Some(character), None, None];
    }
    if ('ㅏ'..='ㅣ').contains(&character) {
        return [None, Some(character), None];
    }
    if !(0xac00..=0xd7a3).contains(&code) {
        return [None; 3];
    }
    let code = (code - 0xac00) as usize;
    let coda = code % 28;
    [
        Some(ONSETS[code / 588]),
        Some(VOWELS[code / 28 % 21]),
        (coda > 0).then(|| CODAS[coda - 1]),
    ]
}

fn assemble(parts: [Option<char>; 3]) -> String {
    if let (Some(onset), Some(vowel)) = (parts[0], parts[1]) {
        let onset_index = ONSETS.iter().position(|&c| c == onset);
        let vowel_index = VOWELS.iter().position(|&c| c == vowel);
        let coda_index = match parts[2] {
            Some(coda) => CODAS.iter().position(|&c| c == coda).map(|i| i + 1),
            None => Some(0),
        };
        if let (Some(onset), Some(vowel), Some(coda)) = (onset_index, vowel_index, coda_index) {
            return char::from_u32(0xac00 + (onset * 588 + vowel * 28 + coda) as u32)
                .unwrap()
                .to_string();
        }
    }
    parts.into_iter().flatten().collect()
}

fn with_onset(value: &str, onset: char) -> String {
    let mut parts = disassemble(value);
    parts[0] = Some(onset);
    assemble(parts)
}

fn standard(value: &str, backward: bool) -> Vec<String> {
    let [onset, vowel, _] = disassemble(value);
    let mut result = vec![value.to_owned()];
    let Some(vowel) = vowel else {
        return result;
    };
    if backward {
        if (onset == Some('ㅇ') && "ㅑㅖ".contains(vowel))
            || (onset == Some('ㄴ') && "ㅏㅐㅗㅜㅡㅚ".contains(vowel))
        {
            result.push(with_onset(value, 'ㄹ'));
        } else if onset == Some('ㅇ') && "ㅕㅛㅠㅣ".contains(vowel) {
            result.push(with_onset(value, 'ㄴ'));
            result.push(with_onset(value, 'ㄹ'));
        }
    } else if (onset == Some('ㄹ') && "ㅑㅕㅛㅠㅣㅖ".contains(vowel))
        || (onset == Some('ㄴ') && "ㅕㅛㅠㅣ".contains(vowel))
    {
        result.push(with_onset(value, 'ㅇ'));
    } else if onset == Some('ㄹ') && "ㅏㅐㅗㅜㅡㅚ".contains(vowel) {
        result.push(with_onset(value, 'ㄴ'));
    }
    result
}

fn forced(value: &str, reverse: bool) -> Vec<String> {
    let mut result = standard(value, reverse);
    if result.len() > 1 {
        result.remove(0);
    }
    result
}

fn forced_backward(value: &str, reverse: bool) -> Vec<String> {
    let result = standard(value, !reverse);
    if result.len() > 1 || standard(value, reverse).len() == 1 {
        result
    } else {
        Vec::new()
    }
}

fn one_way(value: &str, backward: bool) -> Vec<String> {
    let [onset, _, _] = disassemble(value);
    let mut result = vec![value.to_owned()];
    if onset == Some(if backward { 'ㅇ' } else { 'ㄹ' }) {
        result.push(with_onset(value, 'ㄴ'));
        result.push(with_onset(value, if backward { 'ㄹ' } else { 'ㅇ' }));
    } else if onset == Some('ㄴ') {
        result.push(with_onset(value, if backward { 'ㄹ' } else { 'ㅇ' }));
    }
    result
}

fn free_consonants(value: &str, include_coda: bool) -> Vec<String> {
    let [onset, vowel, coda] = disassemble(value);
    let is_free = |c: Option<char>| matches!(c, Some('ㄹ' | 'ㄴ' | 'ㅇ'));
    if !is_free(onset) && (!include_coda || !is_free(coda)) {
        return vec![value.to_owned()];
    }
    let onsets = if is_free(onset) {
        vec![Some('ㄹ'), Some('ㄴ'), Some('ㅇ')]
    } else {
        vec![onset]
    };
    let codas = if include_coda && is_free(coda) {
        vec![Some('ㄹ'), Some('ㄴ'), Some('ㅇ')]
    } else {
        vec![coda]
    };
    let mut result = Vec::new();
    for onset in onsets {
        for &coda in &codas {
            result.push(assemble([onset, vowel, coda]));
        }
    }
    result
}

fn vowel_reversal(value: &str) -> Vec<String> {
    let mut parts = disassemble(value);
    let mut result = standard(value, false);
    let flipped = match parts[1] {
        Some('ㅏ') => Some('ㅓ'),
        Some('ㅑ') => Some('ㅕ'),
        Some('ㅓ') => Some('ㅏ'),
        Some('ㅕ') => Some('ㅑ'),
        Some('ㅗ') => Some('ㅜ'),
        Some('ㅛ') => Some('ㅠ'),
        Some('ㅜ') => Some('ㅗ'),
        Some('ㅠ') => Some('ㅛ'),
        _ => None,
    };
    if flipped.is_some() {
        parts[1] = flipped;
        result.push(assemble(parts));
    }
    result
}

fn consonant_reversal(value: &str) -> Vec<String> {
    let [onset, vowel, coda] = disassemble(value);
    let mut result = vec![value.to_owned()];
    if let (Some(onset), Some(coda)) = (onset, coda)
        && onset != coda
        && CODAS.contains(&onset)
        && ONSETS.contains(&coda)
    {
        result.push(assemble([Some(coda), vowel, Some(onset)]));
    }
    result
}

fn roblox(value: &str, backward: bool) -> Vec<String> {
    if !backward && value == "름" {
        return vec!["름".into(), "늠".into(), "음".into()];
    }
    if backward && value == "음" {
        return vec!["음".into(), "름".into()];
    }
    let result = standard(value, backward);
    if result.len() > 1 {
        return result;
    }
    let [onset, vowel, _] = disassemble(value);
    if (!backward && onset == Some('ㄹ'))
        || (backward && onset == Some('ㅇ') && !vowel.is_some_and(|v| "ㅏㅐㅗㅜㅡㅚ".contains(v)))
    {
        vec![
            value.to_owned(),
            with_onset(value, if backward { 'ㄹ' } else { 'ㅇ' }),
        ]
    } else {
        result
    }
}

pub const POSES: [&str; 9] = [
    "명사",
    "의존명사",
    "대명사",
    "수사",
    "부사",
    "관형사",
    "감탄사",
    "구",
    "무품사",
];
pub const CATES: [&str; 4] = ["일반어", "방언", "북한어", "옛말"];
pub const MANNERS: [&str; 4] = [
    "제거 안 함",
    "한 번만 제거",
    "모두 제거",
    "다음 단어 개수 제한",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleForm {
    pub id: String,
    pub metadata: RuleMetadata,
    pub content: RuleContent,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMetadata {
    pub title: String,
    pub updated_at: u64,
    pub color: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleContent {
    pub word_rule: WordRule,
    pub word_connection_rule: WordConnectionRuleForm,
    pub postprocessing: Postprocessing,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordRule {
    pub words: WordSource,
    pub regex_filter: String,
    pub removed_words: String,
    pub added_words: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "option", rename_all = "lowercase")]
pub enum WordSource {
    Manual(ManualWordsOption),
    Selected(SelectedWordsOption),
    File(FileWordsOption),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManualWordsOption {
    pub content: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileWordsOption {
    pub path: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectedWordsOption {
    pub dict: usize,
    pub pos: IndexMap<String, u8>,
    pub cate: IndexMap<String, u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordConnectionRuleForm {
    pub change_func_idx: usize,
    pub raw_head_idx: usize,
    pub head_dir: usize,
    pub raw_tail_idx: usize,
    pub tail_dir: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Postprocessing {
    pub manner: Manner,
    pub added_words: String,
    pub removed_words: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manner {
    pub r#type: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_words_limit: Option<usize>,
}

impl RuleForm {
    pub fn manual(words: &[&str], change_func_idx: usize) -> Self {
        Self {
            id: "manual".into(),
            metadata: RuleMetadata {
                title: "manual".into(),
                updated_at: 0,
                color: String::new(),
            },
            content: RuleContent {
                word_rule: WordRule {
                    words: WordSource::Manual(ManualWordsOption {
                        content: words.join(" "),
                    }),
                    regex_filter: ".*".into(),
                    added_words: String::new(),
                    removed_words: String::new(),
                },
                word_connection_rule: WordConnectionRuleForm {
                    change_func_idx,
                    raw_head_idx: 1,
                    head_dir: 0,
                    raw_tail_idx: 1,
                    tail_dir: 1,
                },
                postprocessing: Postprocessing {
                    manner: Manner {
                        r#type: 0,
                        next_words_limit: None,
                    },
                    added_words: String::new(),
                    removed_words: String::new(),
                },
            },
        }
    }
}
pub fn get_idx(raw: usize, dir: usize) -> Result<isize> {
    if raw == 0 || dir > 1 {
        return Err(Error::InvalidInput(
            "word indices must be positive; direction must be 0 or 1".into(),
        ));
    }
    let raw =
        isize::try_from(raw).map_err(|_| Error::InvalidInput("word index is too large".into()))?;
    Ok(if dir == 0 { raw - 1 } else { -raw })
}

pub fn get_head_tail(word: &str, head: isize, tail: isize) -> Result<(String, String)> {
    let characters: Vec<char> = word.chars().collect();
    let at = |index: isize| -> Result<String> {
        let resolved = if index < 0 {
            (characters.len() as isize).checked_add(index)
        } else {
            Some(index)
        };
        let character = resolved
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| characters.get(i));
        character
            .map(ToString::to_string)
            .ok_or_else(|| Error::InvalidInput(format!("{word} has no character at index {index}")))
    };
    Ok((at(head)?, at(tail)?))
}

pub fn load_words(rule: &WordRule) -> Result<Vec<String>> {
    let mut words: Vec<String> = match &rule.words {
        WordSource::Manual(option) => option
            .content
            .split_whitespace()
            .map(str::to_owned)
            .collect(),
        WordSource::File(option) => std::fs::read_to_string(&option.path)?
            .trim_start_matches('\u{feff}')
            .lines()
            .map(|line| line.trim().to_owned())
            .collect(),
        WordSource::Selected(option) => load_selected_words(option)?,
    };
    words.extend(rule.added_words.split_whitespace().map(str::to_owned));
    let removed: IndexSet<&str> = rule.removed_words.split_whitespace().collect();
    let regex = fancy_regex::Regex::new(&format!("^{}$", rule.regex_filter))?;
    let mut unique = IndexSet::new();
    for word in words {
        if !word.is_empty() && !removed.contains(word.as_str()) && regex.is_match(&word)? {
            unique.insert(word);
        }
    }
    Ok(unique.into_iter().collect())
}

/// The same dictionary URLs and selection ordering used by the TypeScript package.
pub fn dictionary_urls(option: &SelectedWordsOption) -> Result<Vec<String>> {
    if option.dict > 11 {
        return Err(Error::InvalidInput(format!(
            "unknown dictionary {}",
            option.dict
        )));
    }
    let mut result = IndexSet::new();
    for pos in POSES
        .into_iter()
        .filter(|pos| option.pos.get(*pos).copied().unwrap_or(0) != 0)
    {
        for cate in CATES
            .into_iter()
            .filter(|cate| option.cate.get(*cate).copied().unwrap_or(0) != 0)
        {
            let suffix = match option.dict {
                0 => format!("oldict/db/{pos}"),
                1 => format!("stdict/db/{pos}"),
                2 => format!("opendict/db/{cate}/{pos}"),
                3 => format!("naverdict/db/{pos}"),
                4 => "kkutu/db/노인정".into(),
                5 => "kkutu/db/어인정".into(),
                6 => "kkutu3/kkutu3-기초사전".into(),
                7 => "kkutu3/kkutu3-표준사전".into(),
                8 => "kkutu3/kkutu3-복합사전".into(),
                9 => "roblox".into(),
                10 => "usam_word".into(),
                11 => "usam_word_phrase".into(),
                _ => unreachable!(),
            };
            result.insert(format!("https://singrum.github.io/KoreanDict/{suffix}"));
        }
    }
    Ok(result.into_iter().collect())
}

fn load_selected_words(option: &SelectedWordsOption) -> Result<Vec<String>> {
    let urls = dictionary_urls(option)?;
    if urls.is_empty() {
        return Ok(Vec::new());
    }
    #[cfg(feature = "remote")]
    {
        let client = reqwest::blocking::Client::new();
        let mut words = Vec::new();
        for url in urls {
            let text = client
                .get(url)
                .send()
                .and_then(reqwest::blocking::Response::text)
                .map_err(|error| {
                    Error::InvalidInput(format!("dictionary download failed: {error}"))
                })?;
            words.extend(text.split('\n').map(|word| word.trim().to_owned()));
        }
        Ok(words)
    }
    #[cfg(not(feature = "remote"))]
    {
        Err(Error::InvalidInput("selected dictionaries require the optional remote feature; use a manual or file source offline".into()))
    }
}
