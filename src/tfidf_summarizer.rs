use regex::Regex;
use std::collections::{HashMap, HashSet};

pub fn summarize_tfidf(text: &str, num_sentences: usize) -> Vec<String> {
    let sentence_splitter = Regex::new(r"[.!?]\s+").unwrap();
    let sentences: Vec<&str> = sentence_splitter
        .split(text)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut term_doc_freq: HashMap<String, usize> = HashMap::new();
    let mut sentence_term_freqs: Vec<HashMap<String, usize>> = Vec::new();

    for sentence in &sentences {
        let tokens = tokenize(sentence);
        let mut freqs = HashMap::new();
        let mut seen = HashSet::new();

        for token in tokens {
            *freqs.entry(token.clone()).or_insert(0) += 1;
            if !seen.contains(&token) {
                *term_doc_freq.entry(token.clone()).or_insert(0) += 1;
                seen.insert(token);
            }
        }

        sentence_term_freqs.push(freqs);
    }

    let total_docs = sentences.len() as f64;
    let mut sentence_scores: Vec<(usize, f64)> = sentence_term_freqs
        .iter()
        .enumerate()
        .map(|(i, tf)| {
            let score = tf.iter().map(|(term, &freq)| {
                let tf_val = freq as f64;
                let df = *term_doc_freq.get(term).unwrap_or(&1) as f64;
                let idf = (total_docs / df).ln();
                tf_val * idf
            }).sum();

            (i, score)
        })
        .collect();

    sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    sentence_scores
        .into_iter()
        .take(num_sentences.min(sentences.len()))
        .map(|(i, _)| sentences[i].to_string())
        .collect()
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}