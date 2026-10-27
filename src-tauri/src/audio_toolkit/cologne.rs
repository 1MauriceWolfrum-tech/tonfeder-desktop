//! German phonetic correction for custom words (Tonfeder).
//!
//! Handy's fallback matcher in `text.rs` scores with Soundex, an English
//! algorithm, and skips every term with non-ASCII letters, so German names such
//! as "Jäschke" never match. It also does not run at all when the model already
//! received the custom words as vocabulary. For German output Tonfeder
//! additionally compares how words sound using the Cologne phonetics (Kölner
//! Phonetik) and replaces them with the spelling the user entered.

use strsim::normalized_levenshtein;

use super::text::extract_punctuation;

/// Longest run of transcribed words that may form one custom term
/// ("luft wasser wärme pumpe" -> "Luft-Wasser-Wärmepumpe").
const MAX_WINDOW: usize = 4;

/// Custom terms with fewer letters are too ambiguous to match by sound.
const MIN_KEY_LEN: usize = 4;

/// Function words only belong to a custom term that contains them ("Müller und
/// Söhne"). This keeps a window from swallowing "die" or "mit" just because
/// the rest sounds similar.
const STOPWORDS: &[&str] = &[
    "der", "die", "das", "den", "dem", "des", "ein", "eine", "einen", "einem", "einer", "eines",
    "und", "oder", "aber", "doch", "denn", "weil", "dass", "wenn", "als", "wie", "so", "ist",
    "sind", "war", "waren", "bin", "bist", "sein", "hat", "habe", "haben", "hatte", "wird",
    "werden", "wurde", "kann", "muss", "soll", "will", "ich", "du", "er", "sie", "es", "wir",
    "ihr", "mich", "mir", "dich", "dir", "sich", "uns", "euch", "ihn", "ihm", "ihnen", "im", "in",
    "an", "am", "auf", "aus", "bei", "mit", "nach", "von", "vom", "zu", "zum", "zur", "für",
    "über", "unter", "vor", "durch", "gegen", "ohne", "um", "bis", "seit", "nicht", "kein",
    "keine", "noch", "schon", "auch", "nur", "sehr", "hier", "da", "dort", "ja", "nein", "bitte",
    "danke",
];

/// Lowercases and folds a word to the plain letters a-z: umlauts and common
/// accents lose their marks, "ß" becomes "ss", everything else is dropped.
fn fold(word: &str) -> Vec<char> {
    let mut out = Vec::with_capacity(word.len());
    for c in word.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' | 'á' | 'à' | 'â' => out.push('a'),
            'ö' | 'ó' | 'ò' | 'ô' => out.push('o'),
            'ü' | 'ú' | 'ù' | 'û' => out.push('u'),
            'é' | 'è' | 'ê' | 'ë' => out.push('e'),
            'í' | 'ì' | 'î' | 'ï' => out.push('i'),
            'ç' => out.push('c'),
            'ß' => out.extend(['s', 's']),
            c if c.is_ascii_lowercase() => out.push(c),
            _ => {}
        }
    }
    out
}

/// Cologne phonetics code of a word or phrase, e.g. "Meier", "Maier" and
/// "Meyer" all give "67", "Müller-Lüdenscheidt" gives "65752682".
pub fn cologne_code(word: &str) -> String {
    let s = fold(word);
    let mut raw = String::with_capacity(s.len() * 2);
    for (i, &c) in s.iter().enumerate() {
        let prev = if i > 0 { Some(s[i - 1]) } else { None };
        let next = s.get(i + 1).copied();
        let code = match c {
            'a' | 'e' | 'i' | 'j' | 'o' | 'u' | 'y' => "0",
            'b' => "1",
            'p' => {
                if next == Some('h') {
                    "3"
                } else {
                    "1"
                }
            }
            'd' | 't' => {
                if matches!(next, Some('c' | 's' | 'z')) {
                    "8"
                } else {
                    "2"
                }
            }
            'f' | 'v' | 'w' => "3",
            'g' | 'k' | 'q' => "4",
            'c' => {
                let hard = match prev {
                    None => matches!(
                        next,
                        Some('a' | 'h' | 'k' | 'l' | 'o' | 'q' | 'r' | 'u' | 'x')
                    ),
                    Some('s' | 'z') => false,
                    Some(_) => matches!(next, Some('a' | 'h' | 'k' | 'o' | 'q' | 'u' | 'x')),
                };
                if hard {
                    "4"
                } else {
                    "8"
                }
            }
            'x' => {
                if matches!(prev, Some('c' | 'k' | 'q')) {
                    "8"
                } else {
                    "48"
                }
            }
            'l' => "5",
            'm' | 'n' => "6",
            'r' => "7",
            's' | 'z' => "8",
            // 'h' and anything else carry no sound of their own.
            _ => "",
        };
        raw.push_str(code);
    }

    let mut collapsed: Vec<char> = Vec::with_capacity(raw.len());
    for ch in raw.chars() {
        if collapsed.last() != Some(&ch) {
            collapsed.push(ch);
        }
    }
    collapsed
        .iter()
        .enumerate()
        .filter(|&(i, &ch)| i == 0 || ch != '0')
        .map(|(_, &ch)| ch)
        .collect()
}

/// A custom word prepared for matching. A trailing number is kept apart, so
/// "FrostLine 300" can match "frost lein" with or without the number spoken.
struct Entry<'a> {
    full: &'a str,
    base: &'a str,
    number: Option<&'a str>,
    /// Lowercased words of `base`, to tell which function words it contains.
    words: Vec<String>,
    key: String,
    code: String,
}

impl<'a> Entry<'a> {
    fn new(word: &'a str) -> Option<Self> {
        let full = word.trim();
        let (base, number) = match full.rsplit_once(char::is_whitespace) {
            Some((base, number))
                if number.starts_with(|c: char| c.is_ascii_digit()) && !base.trim().is_empty() =>
            {
                (base.trim(), Some(number))
            }
            _ => (full, None),
        };
        if base.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }
        let key: String = fold(base).into_iter().collect();
        if key.chars().count() < MIN_KEY_LEN {
            return None;
        }
        Some(Self {
            full,
            base,
            number,
            words: base
                .split_whitespace()
                .map(|w| core(w).to_lowercase())
                .collect(),
            code: cologne_code(base),
            key,
        })
    }
}

/// One whitespace-separated word of the transcript.
struct Token<'t> {
    text: &'t str,
    /// The whitespace before this token contains a line break. A term is never
    /// joined across it, so paragraphs and list items stay apart.
    after_line_break: bool,
}

/// The token without leading and trailing punctuation.
fn core(token: &str) -> &str {
    let (prefix, suffix) = extract_punctuation(token);
    &token[prefix.len()..token.len() - suffix.len()]
}

/// Tokens that may never be part of a matched term: pure punctuation and
/// numbers.
fn is_blocked(token: &str) -> bool {
    !token.chars().any(char::is_alphanumeric) || core(token).chars().any(|c| c.is_ascii_digit())
}

/// The lowercased token if it is a function word.
fn stopword(token: &str) -> Option<String> {
    let word = core(token).to_lowercase();
    STOPWORDS.contains(&word.as_str()).then_some(word)
}

/// A custom word written all in lowercase ("abtauen") still starts with a
/// capital letter where the transcript has one, e.g. at the start of a
/// sentence. Mixed-case spellings such as "iPhone" are kept as entered.
fn capitalize_like(original: &str, replacement: &str) -> String {
    let starts_upper = original.chars().next().is_some_and(char::is_uppercase);
    if !starts_upper || replacement.chars().any(char::is_uppercase) {
        return replacement.to_string();
    }
    let mut chars = replacement.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Endings a German noun can take in plural or genitive.
const ENDINGS: &[&str] = &["e", "s", "es", "n", "en", "ns", "ens", "er", "ern"];

struct Match {
    consumed: usize,
    replacement: String,
    similarity: f64,
}

/// Finds the custom word that sounds most like a run of tokens starting at
/// `start`: same Cologne code and a spelling that is close enough.
fn best_match(tokens: &[Token], start: usize, entries: &[Entry]) -> Option<Match> {
    let mut best: Option<Match> = None;
    let max_window = MAX_WINDOW.min(tokens.len() - start);
    let mut stopwords: Vec<String> = Vec::new();

    for n in 1..=max_window {
        let window = &tokens[start..start + n];
        let last = window[n - 1].text;
        if is_blocked(last) {
            break;
        }
        // Do not join across punctuation or a line break: in "frost. lein"
        // the period closes the candidate after "frost".
        if n > 1
            && (window[n - 1].after_line_break
                || !extract_punctuation(window[n - 2].text).1.is_empty()
                || !extract_punctuation(last).0.is_empty())
        {
            break;
        }
        stopwords.extend(stopword(last));

        let phrase = window
            .iter()
            .map(|t| core(t.text))
            .collect::<Vec<_>>()
            .join(" ");
        let key: String = fold(&phrase).into_iter().collect();
        let code = cologne_code(&phrase);

        for entry in entries {
            if !stopwords.iter().all(|w| entry.words.contains(w)) {
                continue;
            }
            // "Europaletten" or "Lieferscheins" is the custom word with a
            // German ending. It must not be cut back to the dictionary form.
            // Split up ("euro paletten") it is joined and keeps its ending.
            if key.len() > entry.key.len() && key.starts_with(&entry.key) {
                let ending = &key[entry.key.len()..];
                if n > 1
                    && ENDINGS.contains(&ending)
                    && core(last).ends_with(ending)
                    && best.as_ref().is_none_or(|b| b.similarity < 1.0)
                {
                    best = Some(Match {
                        consumed: n,
                        replacement: format!("{}{}", entry.base, ending),
                        similarity: 1.0,
                    });
                }
                continue;
            }
            // The codes must agree exactly. Allowing one differing digit turned
            // "verdampft" and "verdampfen" into "Verdampfer": German word forms
            // of one stem often differ in just their last consonant.
            if code != entry.code {
                continue;
            }
            let score = normalized_levenshtein(&key, &entry.key);
            let min_score = if entry.code.len() < 4 { 0.75 } else { 0.6 };
            if score < min_score {
                continue;
            }
            if best.as_ref().is_some_and(|b| score <= b.similarity) {
                continue;
            }

            let spoken_number = entry.number.filter(|number| {
                extract_punctuation(last).1.is_empty()
                    && tokens
                        .get(start + n)
                        .is_some_and(|next| !next.after_line_break && core(next.text) == *number)
            });
            best = Some(match spoken_number {
                Some(_) => Match {
                    consumed: n + 1,
                    replacement: entry.full.to_string(),
                    similarity: score,
                },
                None => Match {
                    consumed: n,
                    replacement: entry.base.to_string(),
                    similarity: score,
                },
            });
        }
    }

    best
}

/// Byte ranges of the whitespace-separated tokens in `text`.
fn token_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                spans.push((s, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        spans.push((s, text.len()));
    }
    spans
}

/// Replaces words that sound like one of the custom words with the custom
/// spelling. Meant for German text only. Whitespace and line breaks between
/// unchanged words are kept exactly as transcribed.
pub fn apply_german_phonetic_words(text: &str, custom_words: &[String]) -> String {
    let entries: Vec<Entry> = custom_words.iter().filter_map(|w| Entry::new(w)).collect();
    if entries.is_empty() {
        return text.to_string();
    }

    let spans = token_spans(text);
    let tokens: Vec<Token> = spans
        .iter()
        .enumerate()
        .map(|(i, &(s, e))| Token {
            text: &text[s..e],
            after_line_break: i > 0 && text[spans[i - 1].1..s].contains('\n'),
        })
        .collect();
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    let mut i = 0;

    while i < tokens.len() {
        let Some(m) = best_match(&tokens, i, &entries) else {
            i += 1;
            continue;
        };
        let last = i + m.consumed - 1;
        let (prefix, _) = extract_punctuation(tokens[i].text);
        let (_, suffix) = extract_punctuation(tokens[last].text);
        out.push_str(&text[copied..spans[i].0]);
        out.push_str(prefix);
        out.push_str(&capitalize_like(core(tokens[i].text), &m.replacement));
        out.push_str(suffix);
        copied = spans[last].1;
        i = last + 1;
    }

    out.push_str(&text[copied..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn cologne_code_matches_reference_examples() {
        assert_eq!(cologne_code("Müller-Lüdenscheidt"), "65752682");
        assert_eq!(cologne_code("Wikipedia"), "3412");
        assert_eq!(cologne_code("Breschnew"), "17863");
    }

    #[test]
    fn cologne_code_groups_spelling_variants() {
        assert_eq!(cologne_code("Meier"), "67");
        assert_eq!(cologne_code("Maier"), "67");
        assert_eq!(cologne_code("Meyer"), "67");
        assert_eq!(cologne_code("Jäschke"), cologne_code("Jeschke"));
        assert_eq!(cologne_code("frost lein"), cologne_code("FrostLine"));
        assert_eq!(cologne_code("Straße"), cologne_code("Strasse"));
    }

    #[test]
    fn corrects_name_with_umlaut() {
        let result = apply_german_phonetic_words("Frau jeschke ruft zurück.", &words(&["Jäschke"]));
        assert_eq!(result, "Frau Jäschke ruft zurück.");
    }

    #[test]
    fn joins_split_words() {
        let custom = words(&["Filtertrockner", "Luft-Wasser-Wärmepumpe"]);
        assert_eq!(
            apply_german_phonetic_words("filter trockner getauscht", &custom),
            "Filtertrockner getauscht"
        );
        assert_eq!(
            apply_german_phonetic_words("Eine neue luft wasser wärme pumpe.", &custom),
            "Eine neue Luft-Wasser-Wärmepumpe."
        );
    }

    #[test]
    fn keeps_trailing_number_of_product_name() {
        let custom = words(&["FrostLine 300"]);
        assert_eq!(
            apply_german_phonetic_words("Die frost lein 300 läuft wieder.", &custom),
            "Die FrostLine 300 läuft wieder."
        );
        assert_eq!(
            apply_german_phonetic_words("Die frost lein läuft wieder.", &custom),
            "Die FrostLine läuft wieder."
        );
    }

    #[test]
    fn keeps_punctuation_around_the_term() {
        let result = apply_german_phonetic_words("(jeschke), bitte", &words(&["Jäschke"]));
        assert_eq!(result, "(Jäschke), bitte");
    }

    #[test]
    fn does_not_join_across_punctuation() {
        let custom = words(&["FrostLine"]);
        assert_eq!(
            apply_german_phonetic_words("Es war frost. Lein ist neu.", &custom),
            "Es war frost. Lein ist neu."
        );
    }

    #[test]
    fn keeps_line_breaks_and_spacing() {
        let custom = words(&["Jäschke"]);
        assert_eq!(
            apply_german_phonetic_words("Hallo  Frau jeschke,\nbis morgen.", &custom),
            "Hallo  Frau Jäschke,\nbis morgen."
        );
    }

    #[test]
    fn leaves_similar_but_different_words_alone() {
        let custom = words(&["Meier", "Halbach", "Kommissionierzone"]);
        let text = "Die Mauer steht, um halb acht läuft die Kommissionierung.";
        assert_eq!(apply_german_phonetic_words(text, &custom), text);
    }

    #[test]
    fn keeps_inflected_forms() {
        let custom = words(&["Europalette", "Lieferschein", "Verdampfer"]);
        let text = "Drei Europaletten, zwei Lieferscheine und der Lüfter des Verdampfers.";
        assert_eq!(apply_german_phonetic_words(text, &custom), text);
    }

    #[test]
    fn joins_split_words_with_german_ending() {
        let custom = words(&["Europalette", "Kältekreis"]);
        assert_eq!(
            apply_german_phonetic_words(
                "zwei euro paletten und die Dichtung des kälte kreises",
                &custom
            ),
            "zwei Europaletten und die Dichtung des Kältekreises"
        );
    }

    #[test]
    fn keeps_related_words_of_the_same_stem() {
        let custom = words(&["Verdampfer"]);
        let text = "Das Kältemittel verdampft, beim Verdampfen sinkt die Temperatur.";
        assert_eq!(apply_german_phonetic_words(text, &custom), text);
    }

    #[test]
    fn accepts_close_spelling_of_short_codes() {
        let custom = words(&["Meier"]);
        assert_eq!(
            apply_german_phonetic_words("Herr Maier kommt.", &custom),
            "Herr Meier kommt."
        );
    }

    #[test]
    fn never_swallows_function_words() {
        let custom = words(&["Andersen"]);
        assert_eq!(
            apply_german_phonetic_words("an der Ecke", &custom),
            "an der Ecke"
        );
    }

    #[test]
    fn skips_short_and_numeric_terms() {
        let custom = words(&["VDI 2035", "R-290", "Öl"]);
        let text = "nach vdi zwanzig fünfunddreißig, öl und r 290";
        assert_eq!(apply_german_phonetic_words(text, &custom), text);
    }

    #[test]
    fn keeps_capital_letter_for_lowercase_entries() {
        let custom = words(&["abtauen", "tonfeder", "eBay-Shop"]);
        assert_eq!(
            apply_german_phonetic_words("Abtauen dauert lange.", &custom),
            "Abtauen dauert lange."
        );
        assert_eq!(
            apply_german_phonetic_words("Ton feder läuft. Im Ebay shop gekauft.", &custom),
            "Tonfeder läuft. Im eBay-Shop gekauft."
        );
    }

    #[test]
    fn does_not_join_across_line_breaks() {
        let custom = words(&["Filtertrockner", "FrostLine 300"]);
        let text = "Ersatzteile: filter\ntrockner und Kabel\nfrost lein\n300 Stück";
        assert_eq!(
            apply_german_phonetic_words(text, &custom),
            "Ersatzteile: filter\ntrockner und Kabel\nFrostLine\n300 Stück"
        );
    }

    #[test]
    fn matches_entries_that_contain_function_words() {
        let custom = words(&["Müller und Söhne", "Andersen"]);
        assert_eq!(
            apply_german_phonetic_words("Rechnung an Möller und Söhne.", &custom),
            "Rechnung an Müller und Söhne."
        );
        // A function word still never joins a term that does not contain it.
        assert_eq!(
            apply_german_phonetic_words("an der Ecke", &custom),
            "an der Ecke"
        );
    }

    #[test]
    fn unchanged_without_custom_words() {
        let text = "Alles bleibt, wie es ist.";
        assert_eq!(apply_german_phonetic_words(text, &[]), text);
    }

    #[test]
    fn already_correct_text_stays_the_same() {
        let text = "Die FrostLine 300 von Frau Jäschke.";
        let custom = words(&["FrostLine 300", "Jäschke"]);
        assert_eq!(apply_german_phonetic_words(text, &custom), text);
    }
}
