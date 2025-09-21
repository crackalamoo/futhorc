pub mod futhorc;

use smallvec::SmallVec;
use std::collections::HashMap;

pub type CollapsedKey = smallvec::SmallVec<[u16; 32]>;

pub type Ambiguities = SmallVec<[u16; 4]>;

#[derive(Default, Debug)]
struct BuildState {
    seen_f: u32,
    seen_v: u32,
    seen_s: u32,
    seen_z: u32,
}

pub type AmbiguityMap = HashMap<CollapsedKey, Ambiguities>;

fn remove_stress_markers(string: &str) -> String {
    let mut output = String::new();

    for ch in string.chars() {
        match ch {
            'ˈ' | 'ˌ' => {}
            ch => output.push(ch),
        }
    }

    output
}

fn translate_to_runic(string: &str) -> String {
    let mut output = String::new();

    for char in string.chars() {
        let runes = match char {
            'X' => " ",
            ' ' => "᛫",
            'a' => "ᚪ",             // f_a_r
            'ɑ' | 'ɔ' => "ᛟ",       // h_o_t (American), h_o_t
            'æ' => "ᚫ",             // h_a_t
            'ɛ' => "ᛖ",             // s_e_nd
            'ɪ' | 'I' => "ᛁ",       // s_i_t, w_e_'ll, an_y_
            'i' => "ᛁᛁ",            // s_ee_d
            'ʊ' | 'u' => "ᚣ",       // b_oo_k, f_oo_d
            'ə' | 'ʌ' | 'ɜ' => "ᚢ", // _a_bout, f_u_n, t_u_rn
            'p' | 'P' => "ᛈ",       // _p_ot
            'b' => "ᛒ",             // _b_oy
            't' | 'T' => "ᛏ",       // _t_ime
            'd' | 'D' => "ᛞ",       // _d_og
            'k' | 'K' => "ᚳ",       // _k_ite
            'g' => "ᚷ",             // _g_ame
            'f' | 'F' => "ᚠᚠ",      // _f_ear
            'v' => "ᚠ",             // _v_ine
            'θ' | 'ð' => "ᚦ",       // _th_ing, _th_is
            's' => "ᛋᛋ",            // _s_ee, lot_s_
            'z' => "ᛋ",             // _z_ebra, song_s_
            'ʃ' | 'ʒ' => "ᛋᚻ",      // _sh_are, mea_s_ure
            'h' => "ᚻ",             // _h_ole
            'm' | 'M' => "ᛗ",       // _m_outh
            'n' | 'N' => "ᚾ",       // _n_ow
            'ŋ' => "ᛝ",             // ri_ng_
            'j' => "ᛄ",             // _y_ou
            'w' => "ᚹ",             // _w_ind
            'ɹ' | 'R' => "ᚱ",       // _r_ain
            'l' | 'L' => "ᛚ",       // _l_ine
            'ˣ' => "ᛉ",             // letter x represented as ks
            // Added below this line.
            'ʤ' => "ᚷᚻ", // _j_og
            'ʧ' => "ᚳᚻ", // _ch_eese
            'ɚ' => "ᚢᚱ", // runn_er_
            c => &c.to_string(),
        };

        output.push_str(runes);
    }

    output = output.replace("ᛋᛏ", "ᛥ");
    output = output.replace("ᚳᚹ", "ᛢ");

    output
}

fn translate_to_runic_2(string: &str) -> String {
    let vec: Vec<_> = string.chars().collect();
    let mut string = String::new();

    let mut skip = false;

    for two in vec.windows(2) {
        if skip {
            skip = false;
            continue;
        }

        let output = match two {
            // 2nd 2nd added.
            ['e', 'ɪ' | 'j'] => {
                skip = true;
                "ᛠ" // st_ay_
            }
            ['a', 'ɪ' | 'j'] => {
                skip = true;
                "ᛡ" // l_ie_
            }
            // 2nd 2nd added.
            ['a', 'ʊ' | 'w'] => {
                skip = true;
                "ᚪᚹ" // f_ou_nd
            }
            ['ɑ', 'ɹ'] => {
                skip = true;
                "ᚪᚱ" // f_ar_
            }
            ['ɛ', 'ɹ'] => {
                skip = true;
                "ᛠᚱ" // ai_r
            }
            ['ɪ' | 'i', 'ɹ'] => {
                skip = true;
                "ᛁᛁᚱ" // f_ear_
            }
            // 2nd 2nd added.
            ['o', 'ʊ' | 'w'] => {
                skip = true;
                "ᚩ" // n_o_
            }
            ['ɔ', 'ɪ' | 'j'] => {
                skip = true;
                "ᚩᛁ" // p_oi_nt
            }
            ['ɔ', 'ɹ'] => {
                skip = true;
                "ᚩᚱ" // d_oo_r
            }
            ['t', 'ʃ'] => {
                skip = true;
                "ᚳᚻ" // _ch_eese
            }
            ['d', 'ʒ'] => {
                skip = true;
                "ᚷᚻ" // _j_og
            }
            ['ŋ', 'g'] => {
                skip = true;
                "ᛝ" // ri_ng_
            }
            // Added.
            ['s', 'S'] => {
                skip = true;
                "ᛋᛋᛋ" // mi_ss_tate
            }
            [one] | [one, _] => &one.to_string(),
            [] | [..] => "",
        };

        string.push_str(output);
    }

    if !skip {
        string.push(*vec.last().unwrap());
    }

    string
}

pub fn disambiguate(ipa: &str, ambiguities: &AmbiguityMap) -> String {
    // 1. Replace all F, V, S, Z with their lowercase equivalents.
    let ipa_no_stress = remove_stress_markers(ipa);
    let mut ipa_chars: Vec<char> = ipa_no_stress
        .chars()
        .map(|c| match c {
            'F' => 'f',
            'V' => 'v',
            'S' => 's',
            'Z' => 'z',
            _ => c,
        })
        .collect();

    let collapsed = collapse_key(&ipa_chars.iter().collect::<String>());
    if let Some(ambs) = ambiguities.get(&collapsed) {
        // 2. Set all ambiguous /f/ to F, all ambiguous /s/ to S.
        for &idx in ambs.iter() {
            if (idx as usize) < ipa_chars.len() {
                match ipa_chars[idx as usize] {
                    'f' => ipa_chars[idx as usize] = 'F',
                    's' => ipa_chars[idx as usize] = 'S',
                    _ => {}
                }
            }
        }
    }
    // 3. Replace all remaining /f/ with /v/, all remaining /s/ with /z/.
    for c in ipa_chars.iter_mut() {
        match *c {
            'f' => *c = 'v',
            's' => *c = 'z',
            _ => {}
        }
    }
    // 4. Replace F and S with their lowercase equivalents.
    for c in ipa_chars.iter_mut() {
        match *c {
            'F' => *c = 'f',
            'S' => *c = 's',
            _ => {}
        }
    }
    ipa_chars.iter().collect()
}

pub fn collapse_key(seq: &String) -> CollapsedKey {
    let mut key = CollapsedKey::new();
    key.reserve(seq.len());
    // let mut key: CollapsedKey = SmallVec::new();
    for ch in seq.chars() {
        key.push(match ch {
            'f' | 'v' => 0,
            's' | 'S' | 'z' => 1,
            _ => ch as u16,
        });
    }
    key
}

fn ingest_word(word: String) -> BuildState {
    let mut st = BuildState::default();
    for (i, ch) in word.chars().enumerate() {
        if i >= 32 {
            break;
        }
        let bit = 1u32 << i;
        match ch {
            'f' => st.seen_f |= bit,
            'v' => st.seen_v |= bit,
            's' | 'S' => st.seen_s |= bit,
            'z' => st.seen_z |= bit,
            _ => {}
        }
    }
    st
}

pub fn detect_ambiguities(english_to_ipa: &HashMap<String, String>) -> AmbiguityMap {
    let mut full_dict: HashMap<CollapsedKey, BuildState> =
        HashMap::with_capacity(english_to_ipa.len());
    for (_eng, ipa) in english_to_ipa.iter() {
        let ipa_no_stress = remove_stress_markers(ipa);
        let collapsed = collapse_key(&ipa_no_stress);
        let st = ingest_word(ipa_no_stress);
        let entry = full_dict.entry(collapsed).or_default();
        entry.seen_f |= st.seen_f;
        entry.seen_v |= st.seen_v;
        entry.seen_s |= st.seen_s;
        entry.seen_z |= st.seen_z;
    }
    let mut amb_map: AmbiguityMap = HashMap::new();
    for (key, st) in full_dict {
        let mut amb = (st.seen_f & st.seen_v) | (st.seen_s & st.seen_z);
        if amb != 0 {
            let mut idxs: SmallVec<[u16; 4]> = SmallVec::new();
            while amb != 0 {
                let tz = amb.trailing_zeros() as u16;
                idxs.push(tz);
                // Clear the lowest set bit.
                amb &= amb - 1;
            }
            amb_map.insert(key, idxs);
        }
    }
    amb_map
}
