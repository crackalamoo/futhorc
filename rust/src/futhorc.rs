#![cfg(feature = "zip")]

//! Futhorc created by Harys Dalvi and David Campbell (<https://www.harysdalvi.com/futhorc/>)

use crate::{
    AmbiguityMap, detect_ambiguities, disambiguate, remove_stress_markers, translate_to_runic,
    translate_to_runic_2,
};
use std::collections::HashMap;

#[cfg(feature = "js")]
use wasm_bindgen::prelude::wasm_bindgen;

/// # Javascript Package
///
/// In order to run the javascript pkg:
///
/// ```sh
/// cargo install wasm-pack
/// make js
/// ```
///
/// Then copy the pkg folder to your web browser's site folder. For example with Apache on Debian:
///
/// ```sh
/// sudo mkdir --parent /var/www/html/pkg
/// sudo cp -r pkg /var/www/html
/// ```
///
/// Or if you installed the package via npm:
///
/// ```sh
/// sudo mkdir --parent /var/www/html/pkg
/// sudo cp ~/node_modules/futhorc/* /var/www/html/pkg
/// ```
///
/// Then load the javascript on a webpage:
///
/// ```sh
/// cat << EOF > /var/www/html/index.html
/// <!DOCTYPE html>
/// <html>
/// <head>
///     <title>Futhorc</title>
/// </head>
/// <body>
///     <h1>Futhorc</h1>
///     <script type="module">
///         import init, { EnglishToRunes } from '../pkg/futhorc.js';
///
///         init().then(() => {
///             const word = "know";
///             console.log(word);
///
///             const dictionary = new EnglishToRunes();
///             const runes = dictionary.translate_js(word)
///             console.log(runes);
///         });
///     </script>
/// </body>
/// </html>
/// EOF
/// ```
#[must_use]
pub fn words_to_runes(words: String) -> String {
    let dictionary = EnglishToRunes::default();
    dictionary.translate(words)
}

#[cfg(not(feature = "js"))]
#[derive(Clone, Debug)]
pub struct EnglishToRunes {
    pub english_to_ipa: HashMap<String, String>,
    pub ambiguity_map: AmbiguityMap,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Clone, Debug)]
pub struct EnglishToRunes {
    #[wasm_bindgen(skip)]
    pub english_to_ipa: HashMap<String, String>,
    #[wasm_bindgen(skip)]
    pub ambiguity_map: AmbiguityMap,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl EnglishToRunes {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new() -> EnglishToRunes {
        EnglishToRunes::default()
    }

    #[must_use]
    pub fn translate_js(&self, words: String) -> String {
        self.translate(words)
    }
}

impl EnglishToRunes {
    // 1.  Apostrophes and punctuation are used just like in standard English.
    // 2.  /i/ at the end of a word or before an apostrophe is simply ᛁ. So you
    //     have we'll/ᚹᛁ'ᛚ, will/ᚹᛁᛚ, wheel/ᚹᛁᛁᛚ. This applies to morphemes too:
    //     you have any/ᛖᚾᛁ, anything/ᛖᚾᛁᚦᛁᛝ. Also note that ᛁᛁ may represent /iɪ/ as in being/ᛒᛁᛁᛝ.
    // 3.  If there is ambiguity between /f/ and /v/, use ᚠᚠ for /f/ and ᚠ for /v/.
    //     So you have live/ᛚᛁᚠ, leave/ᛚᛁᛁᚠ, leaf/ᛚᛁᛁᚠᚠ, lives/ᛚᛁᚠᛋ, leaves/ᛚᛁᛁᚠᛋ.
    //     Note that rules apply in the order they are listed here in case of a conflict.
    // 4.  There is similar ambiguity clarification as above for /s/ (ᛋᛋ) and /z/
    //     (ᛋ), So you have ones/ᚹᚢᚾᛋ, once/ᚹᚢᚾᛋᛋ
    // 5.  "No" is spelled ᚾᚩ and "know" is spelled ᚾᚩᚹ.
    // 6.  Words which use "tr" for /tʃɹ/ in standard English are spelled with ᛏᚱ,
    //     not ᚳᚻᚱ. Similar for "dr"/ᛞᚱ and /dʒɹ/; "x"/ᛉ and /ks/. So you have
    //     truck/ᛏᚱᚢᚳ, draw/ᛞᚱᛟ, tax/ᛏᚫᛉ, racks/ᚱᚫᚳᛋ.
    // 7.  Word-final /ə/ (Added or ʌ or ɜ) is written ᚪ. So you have comma/ᚳᛟᛗᚪ (not ᚳᛟᛗᚢ),
    //     vanilla/ᚠᚢᚾᛁᛚᚪ. Exception: the/ᚦᛖ.
    // 8.  Syllabic consonants are spelled with ᚢ before the consonant. So you have bottle/ᛒᛟᛏᚢᛚ.
    // 9.  ᛋ and ᛏ are optionally written together as the ligature ᛥ as in stone/ᛥᚩᚾ.
    //     Likewise for ᚳᚹ becoming ᛢ (Optional, not doing...).
    // 10. The name of this alphabet is written ᚠᚢᚦᚩᚱᚳ, but pronounced /fuθork/
    //     like "FOO-thork" as if it were spelled ᚠᚣᚦᚩᚱᚳ.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn translate(&self, mut words: String) -> String {
        words.make_ascii_lowercase();
        let mut ipa_words = Vec::new();

        let mut whitespaces = parse_whitespace(&words);
        let mut j = 0;

        for (i, word) in words.split_whitespace().enumerate() {
            let mut word = word.to_string();
            let mut ch = ' ';

            if word.ends_with(',')
                || word.ends_with(':')
                || word.ends_with(';')
                || word.ends_with('.')
                || word.ends_with('!')
                || word.ends_with('?')
            {
                let mut chars = word.chars();
                ch = chars.next_back().unwrap();
                word = chars.as_str().to_string();
            }

            if let Some(ipa_word) = self.english_to_ipa.get(&word) {
                handle_ipa_word(&mut ipa_words, ipa_word, &word);
                handle_punctuation(ch, &mut ipa_words, &mut whitespaces, i + j + 1);
            } else if word.contains('-') {
                let mut skip = true;

                for split_word in word.split('-') {
                    if let Some(ipa_word) = self.english_to_ipa.get(split_word) {
                        handle_ipa_word(&mut ipa_words, ipa_word, &word);
                    } else {
                        ipa_words.push((split_word.to_string(), false));
                    }

                    if !skip {
                        whitespaces.insert(i + j + 1, "-".to_string());
                        j += 1;
                    }

                    skip = false;
                }

                handle_punctuation(ch, &mut ipa_words, &mut whitespaces, i + j + 1);
            } else {
                ipa_words.push((word, false));
                handle_punctuation(ch, &mut ipa_words, &mut whitespaces, i + j + 1);
            }
        }

        let mut translated = String::new();
        translated.push_str(&whitespaces[0]);

        for (i, (word, translated_to_ipa)) in ipa_words.iter().enumerate() {
            if *translated_to_ipa {
                let rune_word = translate_to_runic(&translate_to_runic_2(&disambiguate(
                    word,
                    &self.ambiguity_map,
                )));

                #[cfg(feature = "debug")]
                println!("{word} {rune_word}");

                translated.push_str(&rune_word);
            } else {
                translated.push_str(word);
            }

            translated.push_str(&translate_to_runic(&whitespaces[i + 1]));
        }

        translated
    }
}

impl Default for EnglishToRunes {
    fn default() -> Self {
        let ipa = include_str!("../CMU.in.IPA.txt");

        let mut english_to_ipa = HashMap::new();
        for line in ipa.lines() {
            let words: Vec<_> = line.split_ascii_whitespace().collect();
            let mut words_0 = words[0].chars();
            words_0.next_back();
            let words_0 = words_0.as_str();

            if words[0] == "XXXXX" {
                continue;
            }

            english_to_ipa.insert(words_0.to_string(), words[1].to_string());
        }

        english_to_ipa.insert("know".to_string(), "noʊw".to_string());
        english_to_ipa.insert("futhorc".to_string(), "vʌθɑɹk".to_string());
        english_to_ipa.insert("the".to_string(), "ðɛ".to_string());
        english_to_ipa.insert("'tis".to_string(), "'tɪz".to_string());
        english_to_ipa.insert("and".to_string(), "ænd".to_string());
        english_to_ipa.insert("of".to_string(), "ɔv".to_string());
        english_to_ipa.insert("a".to_string(), "ᚢ".to_string());
        english_to_ipa.insert("from".to_string(), "fɹɔm".to_string());
        english_to_ipa.insert("aren't".to_string(), "ɑɹnt".to_string());
        english_to_ipa.insert("isn't".to_string(), "ɪznt".to_string());
        english_to_ipa.insert("didn't".to_string(), "dɪdnt".to_string());
        english_to_ipa.insert("doesn't".to_string(), "dʌznt".to_string());
        english_to_ipa.insert("shouldn't".to_string(), "ʃʊdənt".to_string());
        english_to_ipa.insert("couldn't".to_string(), "kʊdnt".to_string());
        english_to_ipa.insert("wouldn't".to_string(), "wʊdnt".to_string());
        english_to_ipa.insert("i'm".to_string(), "aɪ'm".to_string());
        english_to_ipa.insert("for".to_string(), "vɔɹ".to_string());
        english_to_ipa.insert("so".to_string(), "zow".to_string());
        english_to_ipa.insert("use".to_string(), "juz".to_string());
        english_to_ipa.insert("first".to_string(), "vɚst".to_string());
        english_to_ipa.insert("vase".to_string(), "vaz".to_string());
        english_to_ipa.insert("worse".to_string(), "wɚz".to_string());
        english_to_ipa.insert("either".to_string(), "aɪðɚ".to_string());
        english_to_ipa.insert("neither".to_string(), "naɪðɚ".to_string());
        english_to_ipa.insert("else".to_string(), "ɛlz".to_string());
        english_to_ipa.insert("since".to_string(), "zɪns".to_string());

        let ambiguity_map = detect_ambiguities(&english_to_ipa);

        Self {
            english_to_ipa,
            ambiguity_map,
        }
    }
}

fn handle_ipa_word(ipa_words: &mut Vec<(String, bool)>, ipa_word: &str, word: &str) {
    if word.chars().count() == 1 {
        let ch = word.chars().next().unwrap();
        if ch != 'a' && ch != 'i' {
            ipa_words.push((word.to_string(), false));
            return;
        }
    }

    let mut ipa_word = remove_stress_markers(ipa_word);

    if ipa_word.ends_with('ə') || ipa_word.ends_with('ʌ') || ipa_word.ends_with('ɜ') {
        let mut chars = ipa_word.chars();
        chars.next_back().unwrap();
        ipa_word = chars.as_str().to_string();
        ipa_word.push('a');
    }

    if ipa_word.ends_with('i') {
        let mut chars = ipa_word.chars();
        chars.next_back().unwrap();
        ipa_word = chars.as_str().to_string();
        ipa_word.push('I');
    }

    if word.ends_with("'t") {
        let mut chars = ipa_word.chars();
        let c = chars.next_back().unwrap();
        ipa_word = chars.as_str().to_string();
        ipa_word.push('\'');
        ipa_word.push(c);
    } else if word.ends_with("'d") {
        let mut chars = ipa_word.chars();
        let c = chars.next_back().unwrap();

        if Some('ʌ') == chars.clone().last() || Some('ɪ') == chars.clone().last() {
            chars.next_back().unwrap();
            ipa_word = chars.as_str().to_string();
            ipa_word.push_str("'d");
        } else {
            ipa_word = chars.as_str().to_string();
            ipa_word.push('\'');
            ipa_word.push(c);
        }
    } else if word.ends_with("'s") {
        let mut chars = ipa_word.chars();
        let c = chars.next_back().unwrap();

        if Some('i') == chars.clone().last() {
            chars.next_back().unwrap();
            ipa_word = chars.as_str().to_string();
            ipa_word.push('I');
        } else {
            ipa_word = chars.as_str().to_string();
        }

        ipa_word.push('\'');
        ipa_word.push(c);
    } else if word.ends_with("'ll") {
        let mut chars: Vec<char> = ipa_word.chars().collect();

        if let Some('l') = chars.pop() {
            if let Some(prev) = chars.last() {
                if matches!(*prev, 'ə' | 'ʌ' | 'ɜ' | 'a') {
                    chars.pop();
                }
            }

            if let Some(prev) = chars.last() {
                if *prev == 'i' {
                    chars.pop();
                    chars.push('I');
                }
            }

            chars.push('\'');
            chars.push('l');
            ipa_word = chars.into_iter().collect();
        } else {
            ipa_word.push_str("'l");
        }
    } else if word.ends_with('\'') {
        ipa_word.push('\'');
    } else if word.ends_with("'re") {
        let mut chars = ipa_word.chars();
        chars.next_back().unwrap();
        ipa_word = chars.as_str().to_string();
        ipa_word.push_str("'ɹ");
    } else if word.ends_with("'ve") {
        let mut chars: Vec<char> = ipa_word.chars().collect();

        if let Some(c) = chars.pop() {
            if let Some(prev) = chars.last() {
                if matches!(*prev, 'ə' | 'ʌ' | 'ɜ' | 'a') {
                    chars.pop();
                }
            }

            if let Some(prev) = chars.last() {
                if *prev == 'i' {
                    chars.pop();
                    chars.push('I');
                }
            }

            chars.push('\'');
            chars.push(c);
            ipa_word = chars.into_iter().collect();
        } else {
            ipa_word.push_str("'v");
        }
    }

    ipa_word = mark_letter_x(word, ipa_word);

    ipa_words.push((ipa_word, true));
}

fn mark_letter_x(word: &str, ipa_input: String) -> String {
    if !word.chars().any(|c| c == 'x' || c == 'X') {
        return ipa_input;
    }

    let mut ipa_chars: Vec<char> = ipa_input.chars().collect();
    let mut search_start = 0usize;

    for ch in word.chars() {
        if ch.eq_ignore_ascii_case(&'x') {
            let mut idx = search_start;
            let mut found = None;
            while idx + 1 < ipa_chars.len() {
                if ipa_chars[idx] == 'k' && ipa_chars[idx + 1] == 's' {
                    found = Some(idx);
                    break;
                }
                idx += 1;
            }

            if let Some(pos) = found {
                ipa_chars[pos] = 'ˣ';
                ipa_chars.remove(pos + 1);
                search_start = pos + 1;
            }
        }
    }

    ipa_chars.into_iter().collect()
}

fn handle_punctuation(
    ch: char,
    ipa_words: &mut [(String, bool)],
    whitespaces: &mut [String],
    index: usize,
) {
    if ch != ' ' {
        let (ipa_word, _) = ipa_words.last_mut().unwrap();
        ipa_word.push(ch);

        let whitespace = &mut whitespaces[index];
        if whitespace.starts_with(' ') {
            whitespace.remove(0);
            whitespace.insert(0, 'X');
        }
    }
}

fn parse_whitespace(words: &str) -> Vec<String> {
    let mut space = String::new();
    let mut spaces = Vec::new();
    let mut on_space = true;

    for ch in words.chars() {
        if on_space {
            if ch.is_whitespace() {
                space.push(ch);
            } else {
                on_space = false;
                spaces.push(space.clone());
                space.clear();
            }
        } else if ch.is_whitespace() {
            on_space = true;
            space.push(ch);
        }
    }

    if on_space {
        spaces.push(space);
    } else {
        spaces.push(String::new());
    }

    spaces
}

#[cfg(test)]
mod tests {
    use crate::futhorc::EnglishToRunes;

    #[test]
    fn know_no_etc() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("no");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚾᚩ");

        let mut words = String::new();
        words.push_str("know");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚾᚩᚹ");
    }

    #[test]
    fn newlines() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("apple banana\ncarrot\n\n");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚫᛈᚢᛚ᛫ᛒᚢᚾᚫᚾᚪ\nᚳᚫᚱᚢᛏ\n\n");
    }

    #[test]
    fn apostrophes() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("abram's");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛠᛒᚱᚢᛗ'ᛋ");

        let mut words = String::new();
        words.push_str("absolut's");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚫᛒᛋᚢᛚᚣᛏ'ᛋ");

        let mut words = String::new();
        words.push_str("company'll");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚳᚢᛗᛈᚢᚾᛁ'ᛚ");

        let mut words = String::new();
        words.push_str("he'll");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᛁ'ᛚ");
    }

    #[test]
    fn final_ə() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("comma");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚳᛟᛗᚪ");

        let mut words = String::new();
        words.push_str("the");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚦᛖ");
    }

    #[test]
    fn syllabic_consonants() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("bottle");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛒᛟᛏᚢᛚ");
    }

    #[test]
    fn ends_with_i() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("wheel");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚹᛁᛁᛚ");

        let mut words = String::new();
        words.push_str("any");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛖᚾᛁ");
    }

    #[test]
    fn i_apostrophe() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("lady's");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᛠᛞᛁ'ᛋ");

        let mut words = String::new();
        words.push_str("we'll");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚹᛁ'ᛚ");
    }

    #[test]
    fn plural_possessive() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("immigrants'");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛁᛗᛁᚷᚱᚢᚾᛏᛋ'");
    }

    #[test]
    fn apostrophe_d() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("who'd");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᛞ");

        let mut words = String::new();
        words.push_str("it'd");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛁᛏ'ᛞ");

        let mut words = String::new();
        words.push_str("that'd");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚦᚫᛏ'ᛞ");
    }

    #[test]
    fn other_apostrophes() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("who'd");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᛞ");

        let mut words = String::new();
        words.push_str("who'll");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᛚ");

        let mut words = String::new();
        words.push_str("who're");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᚱ");

        let mut words = String::new();
        words.push_str("who's");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᛋ");

        let mut words = String::new();
        words.push_str("who've");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚻᚣ'ᚠ");

        let mut words = String::new();
        words.push_str("should've");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛋᚻᚣᛞ'ᚠ");
    }

    #[test]
    fn ambiguity() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("leaves");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᛁᛁᚠᛋ");

        let mut words = String::new();
        words.push_str("leaf");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᛁᛁᚠᚠ");

        let mut words = String::new();
        words.push_str("leave");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᛁᛁᚠ");

        let mut words = String::new();
        words.push_str("lose");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᚣᛋ");

        let mut words = String::new();
        words.push_str("loose");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᛚᚣᛋᛋ");
    }

    #[test]
    fn no_ambiguity() {
        let dictionary = EnglishToRunes::default();

        let mut words = String::new();
        words.push_str("after");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚫᚠᛏᚢᚱ");

        let mut words = String::new();
        words.push_str("ask");
        let output = dictionary.translate(words);
        assert_eq!(output, "ᚫᛋᚳ");
    }
}

/*
The text:
To be, or not to be, that is the question:
Whether 'tis nobler in the mind to suffer
The slings and arrows of outrageous fortune,
Or to take arms against a sea of troubles,
And by opposing end them: to die, to sleep
No more; and by a sleep, to say we end
The heart-ache, and the thousand natural shocks
That Flesh is heir to? 'Tis a consummation
Devoutly to be wished. To die, to sleep,
To sleep, perchance to Dream; aye, there's the rub,
For in that sleep of death, what dreams may come,
When we have shuffled off this mortal coil,
Must give us pause.

My translation:
ᛏᚣ᛫ᛒᛁ,᛫ᛟᚱ᛫ᚾᚪᛏ᛫ᛏᚣ᛫ᛒᛁ,᛫ᚦᚫᛏ᛫ᛁᛋ᛫ᚦᛖ᛫ᚳᚹᛖᛋᛋᚳᚻᚢᚾ:
ᚹᛖᚦᚢᚱ᛫'ᛏᛁᛋ᛫ᚾᚩᛒᛚᚢᚱ᛫ᛁᚾ᛫ᚦᛖ᛫ᛗᛁᚾᛞ᛫ᛏᚣ᛫ᛋᛋᚢᚠᚠᚢᚱ
ᚦᛖ᛫ᛋᛋᛚᛁᛝᛋ᛫ᚢᚾᛞ᛫ᚫᚱᚩᛋ᛫ᚢᚠ᛫ᚪᚹᛏᚱᛠᚷᚻᚢᛋᛋ᛫ᚠᚠᛟᚱᚳᚻᚢᚾ,
ᛟᚱ᛫ᛏᚣ᛫ᛏᛠᚳ᛫ᚪᚱᛗᛋ᛫ᚢᚷᛖᚾᛋᛋᛏ᛫ᚪ᛫ᛋᛋᛁ᛫ᚢᚠ᛫ᛏᚱᚢᛒᚢᛚᛋ,
ᚢᚾᛞ᛫ᛒᛁ᛫ᚢᛈᚩᛋᛁᛝ᛫ᛖᚾᛞ᛫ᚦᛖᛗ:᛫ᛏᚣ᛫ᛞᛁ,᛫ᛏᚣ᛫ᛋᛋᛚᛁᛁᛈ
ᚾᚩ᛫ᛗᛟᚱ;᛫ᚢᚾᛞ᛫ᛒᛁ᛫ᚪ᛫ᛋᛋᛚᛁᛁᛈ,᛫ᛏᚣ᛫ᛋᛋᛠ᛫ᚹᛁ᛫ᛖᚾᛞ
ᚦᛖ᛫ᚻᚪᚱᛏ-ᛠᚳ,᛫ᚢᚾᛞ᛫ᚦᛖ᛫ᚦᚪᚹᛋᚢᚾᛞ᛫ᚾᚫᚳᚻᚢᚱᚢᛚ᛫ᛋᚻᚪᚳᛋᛋ
ᚦᚫᛏ᛫ᚠᚠᛚᛖᛋᚻ᛫ᛁᛋ᛫ᛖᚱ᛫ᛏᚣ?᛫'ᛏᛁᛋ᛫ᚪ᛫ᚳᚪᚾᛋᛋᚢᛗᛠᛋᚻᚢᚾ
ᛞᛁᚠᚪᚹᛏᛚᛁ᛫ᛏᚣ᛫ᛒᛁ᛫ᚹᛁᛋᚻᛏ.᛫ᛏᚣ᛫ᛞᛁ,᛫ᛏᚣ᛫ᛋᛋᛚᛁᛁᛈ,
ᛏᚣ᛫ᛋᛋᛚᛁᛁᛈ,᛫ᛈᚢᚱᚳᚻᚫᚾᛋᛋ᛫ᛏᚣ᛫ᛞᚱᛁᛁᛗ;᛫ᛁ,᛫ᚦᛖᚱ'ᛋ᛫ᚦᛖ᛫ᚱᚢᛒ,
ᚠᚠᛟᚱ᛫ᛁᚾ᛫ᚦᚫᛏ᛫ᛋᛋᛚᛁᛁᛈ᛫ᚢᚠ᛫ᛞᛖᚦ,᛫ᚹᚢᛏ᛫ᛞᚱᛁᛁᛗᛋ᛫ᛗᛠ᛫ᚳᚢᛗ,
ᚹᛖᚾ᛫ᚹᛁ᛫ᚻᚫᚠ᛫ᛋᚻᚢᚠᚠᚢᛚᛞ᛫ᛟᚠᚠ᛫ᚦᛁᛋᛋ᛫ᛗᛟᚱᛏᚢᛚ᛫ᚳᛟᛄᛚ,
ᛗᚢᛋᛋᛏ᛫ᚷᛁᚠ᛫ᚢᛋᛋ᛫ᛈᛟᛋ.

The original translation:
ᛏᚣ᛫ᛒᛁ, ᚪᚱ᛫ᚾᛟᛏ᛫ᛏᚣ᛫ᛒᛁ, ᚦᚫᛏ᛫ᛁᛋ᛫ᚦᛖ᛫ᚳᚹᛖᛋᚳᚻᚢᚾ:
ᚹᛖᚦᚢᚱ᛫'ᛏᛁᛋ᛫ᚾᚩᛒᛚᚢᚱ᛫ᛁᚾ᛫ᚦᛖ᛫ᛗᛡᚾᛞ᛫ᛏᚣ᛫ᛋᚢᚠᚢᚱ
ᚦᛖ᛫ᛋᛚᛁᛝᛋ᛫ᚫᚾᛞ᛫ᚫᚱᚩᛋ᛫ᛟᚠ᛫ᚪᚹᛏᚱᛠᚷᚻᚢᛋ᛫ᚠᚩᚱᚳᚻᚢᚾ,
ᚪᚱ᛫ᛏᚣ᛫ᛏᛠᚳ᛫ᚪᚱᛗᛋ᛫ᚢᚷᛖᚾᛥ᛫ᚢ᛫ᛋᛁ᛫ᛟᚠ᛫ᛏᚱᚢᛒᚢᛚᛋ,
ᚫᚾᛞ᛫ᛒᛡ᛫ᚢᛈᚩᛋᛁᛝ᛫ᛖᚾᛞ᛫ᚦᛖᛗ: ᛏᚣ᛫ᛞᛡ, ᛏᚣ᛫ᛋᛚᛁᛁᛈ
ᚾᚩ᛫ᛗᚩᚱ; ᚫᚾᛞ᛫ᛒᛡ᛫ᚢ᛫ᛋᛚᛁᛁᛈ, ᛏᚣ᛫ᛋᛠ᛫ᚹᛁ᛫ᛖᚾᛞ
ᚦᛖ᛫ᚻᚪᚱᛏ-ᛠᚳ, ᚫᚾᛞ᛫ᚦᛖ᛫ᚦᚪᚹᛋᚢᚾᛞ᛫ᚾᚫᚳᚻᚢᚱᚢᛚ᛫ᛋᚻᛟᚳᛋ
ᚦᚫᛏ᛫ᚠᛚᛖᛋᚻ᛫ᛁᛋ᛫ᛠᚱ᛫ᛏᚣ? 'ᛏᛁᛋ᛫ᚢ᛫ᚳᛟᚾᛋᚢᛗᛠᛋᚻᚢᚾ
ᛞᚢᚠᚪᚹᛏᛚᛁ᛫ᛏᚣ᛫ᛒᛁ᛫ᚹᛁᛋᚻᛞ. ᛏᚣ᛫ᛞᛡ, ᛏᚣ᛫ᛋᛚᛁᛁᛈ,
ᛏᚣ᛫ᛋᛚᛁᛁᛈ, ᛈᚢᚱᚳᚫᚾᛋ᛫ᛏᚣ᛫ᛞᚱᛁᛁᛗ; ᛡ, ᚦᛠᚱ'ᛋ᛫ᚦᛖ᛫ᚱᚢᛒ,
ᚠᚪᚱ᛫ᛁᚾ᛫ᚦᚫᛏ᛫ᛋᛚᛁᛁᛈ᛫ᛟᚠ᛫ᛞᛖᚦ᛫ᚹᚢᛏ᛫ᛞᚱᛁᛁᛗᛋ᛫ᛗᛠ᛫ᚳᚢᛗ,
ᚹᛖᚾ᛫ᚹᛁ᛫ᚻᚫᚠ᛫ᛋᚻᚢᚠᚢᛚᛞ᛫ᛟᚠᚠ᛫ᚦᛁᛋ᛫ᛗᚪᚱᛏᚢᛚ᛫ᚳᚩᛁᛚ,
ᛗᚢᛥ᛫ᚷᛁᚠ᛫ᚢᛋ᛫ᛈᛟᛋ.
*/
