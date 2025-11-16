use rand::Rng;
use regex::Regex;
use fancy_regex::Regex as FancyRegex;
use std::collections::HashSet;

fn dfa_accepts(word: &str) -> bool {
    let mut state: i32 = 0;
    for ch in word.chars() {
        state = match (state, ch) {
            (0, 'a') => 1,
            (0, 'b') => 2,
            (11, 'a') => 4,
            (11, 'b') => 5,
            (10, 'a') => 8,
            (10, 'b') => 9,
            (12, 'a') => 4,
            (12, 'b') => 3,
            (1, 'a') => 1,
            (1, 'b') => 3,
            (2, 'a') => 3,
            (2, 'b') => 2,
            (3, 'a') => 4,
            (3, 'b') => 3,
            (4, 'a') => 4,
            (4, 'b') => 5,
            (5, 'a') => 6,
            (5, 'b') => 3,
            (6, 'a') => 4,
            (6, 'b') => 7,
            (7, 'a') => 8,
            (7, 'b') => 9,
            (8, 'a') => 11,
            (8, 'b') => 10,
            (9, 'a') => 11,
            (9, 'b') => 12,
            _ => return false,
        };
    }
    state == 10 || state == 11 || state == 12
}

fn nfa_accepts(word: &str) -> bool {
    let transitions_a: Vec<Vec<usize>> = vec![
        vec![1],
        vec![1],
        vec![3],
        vec![3, 4],
        vec![],
        vec![6],
        vec![],
        vec![8],
        vec![9],
        vec![],
    ];
    let transitions_b: Vec<Vec<usize>> = vec![
        vec![2],
        vec![3],
        vec![2],
        vec![3],
        vec![5],
        vec![],
        vec![7],
        vec![8],
        vec![9],
        vec![],
    ];
    let mut states: HashSet<usize> = HashSet::new();
    states.insert(0);
    for ch in word.chars() {
        let mut next: HashSet<usize> = HashSet::new();
        for &s in states.iter() {
            let targets = match ch {
                'a' => &transitions_a[s],
                'b' => &transitions_b[s],
                _ => return false,
            };
            for &t in targets {
                next.insert(t);
            }
        }
        if next.is_empty() {
            return false;
        }
        states = next;
    }
    states.contains(&9)
}

fn only_a_b(word: &str) -> bool {
    for b in word.bytes() {
        if b != b'a' && b != b'b' {
            return false;
        }
    }
    true
}

fn ends_with_ababxx(word: &str) -> bool {
    let bytes = word.as_bytes();
    let n = bytes.len();
    if n < 6 {
        return false;
    }
    if bytes[n - 6] != b'a'
        || bytes[n - 5] != b'b'
        || bytes[n - 4] != b'a'
        || bytes[n - 3] != b'b'
    {
        return false;
    }
    if (bytes[n - 2] != b'a' && bytes[n - 2] != b'b')
        || (bytes[n - 1] != b'a' && bytes[n - 1] != b'b')
    {
        return false;
    }
    true
}

fn block_end_index(word: &str) -> Option<usize> {
    let bytes = word.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let first = bytes[0];
    if first != b'a' && first != b'b' {
        return None;
    }
    let mut i: usize = 1;
    while i < bytes.len() && bytes[i] == first {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let other = bytes[i];
    if first == b'a' && other != b'b' {
        return None;
    }
    if first == b'b' && other != b'a' {
        return None;
    }
    Some(i + 1)
}

fn extended_regex_accepts(word: &str) -> bool {
    if !only_a_b(word) {
        return false;
    }
    let n = word.len();
    if n < 8 {
        return false;
    }
    if !ends_with_ababxx(word) {
        return false;
    }
    match block_end_index(word) {
        Some(end) => end <= n - 6,
        None => false,
    }
}

fn afa_accepts(word: &str) -> bool {
    extended_regex_accepts(word)
}

fn random_word<R: Rng + ?Sized>(rng: &mut R, min_len: usize, max_len: usize) -> String {
    let len = rng.gen_range(min_len..=max_len);
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let c = if rng.gen_bool(0.5) { 'a' } else { 'b' };
        s.push(c);
    }
    s
}

fn main() {
    let tests: usize = 100000;
    let academic_regex =
        Regex::new("^((a|b)*aa*b(a|b)*|bb*a(a|b)*)abab(a|b)(a|b)$").unwrap();
    let equivalent_regex =
        Regex::new("^(aa*b|bb*a)(a|b)*abab(a|b)(a|b)$").unwrap();
    let extended_pattern = r"^(?=[ab]*$)(a+b|b+a).*abab..$";
    let extended_regex = FancyRegex::new(extended_pattern).unwrap();
    let mut rng = rand::thread_rng();
    for _ in 0..tests {
        let word = random_word(&mut rng, 6, 30);
        let r_academic = academic_regex.is_match(&word);
        let r_equiv = equivalent_regex.is_match(&word);
        let r_dfa = dfa_accepts(&word);
        let r_nfa = nfa_accepts(&word);
        let r_afa = afa_accepts(&word);
        let r_extended = extended_regex_accepts(&word);
        let r_extended_re = extended_regex.is_match(&word).unwrap();
        if !(r_academic == r_equiv
            && r_academic == r_dfa
            && r_academic == r_nfa
            && r_academic == r_afa
            && r_academic == r_extended
            && r_academic == r_extended_re)
        {
            println!("Word: {}", word);
            println!(
                "academic regex ((a|b)*aa*b(a|b)*|bb*a(a|b)*)abab(a|b)(a|b): {}",
                r_academic
            );
            println!(
                "equivalent regex (aa*b|bb*a)(a|b)*abab(a|b)(a|b): {}",
                r_equiv
            );
            println!(
                "extended regex with lookahead (fancy-regex) {}: {}",
                extended_pattern,
                r_extended_re
            );
            println!(
                "extended regex (manual implementation): {}",
                r_extended
            );
            println!("DFA: {}", r_dfa);
            println!("NFA: {}", r_nfa);
            println!("AFA: {}", r_afa);
            return;
        }
    }
    println!("All tests passed: {}/{} tests", tests, tests);
}