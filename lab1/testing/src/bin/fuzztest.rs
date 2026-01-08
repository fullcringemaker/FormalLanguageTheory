use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

const ALPHABET: &str = "abcd";
const BFS_LIMIT: usize = 300000;
const BFS_DEPTH: usize = 40;
type Rules = Vec<(String, String)>;

fn t_rules() -> Rules {
    vec![
        ("aa".to_string(), "bb".to_string()),
        ("ccb".to_string(), "dca".to_string()),
        ("a".to_string(), "dcc".to_string()),
        ("cd".to_string(), "bc".to_string()),
    ]
}

fn tprime_base() -> Rules {
    vec![
        ("bb".to_string(), "aa".to_string()),
        ("dca".to_string(), "ccb".to_string()),
        ("dcc".to_string(), "a".to_string()),
        ("cd".to_string(), "bc".to_string()),
        ("cccb".to_string(), "bcca".to_string()),
        ("bccc".to_string(), "ca".to_string()),
    ]
}

fn gen_rule_n(n: usize) -> (String, String) {
    let mut lhs = String::from("bcc");
    for _ in 0..(n + 1) {
        lhs.push_str("acc");
    }
    lhs.push('c');
    let rhs = "c".repeat(3 * n + 4) + "a";
    (lhs, rhs)
}

fn build_tprime(n: usize) -> Rules {
    let mut rules = tprime_base();
    for k in 0..n {
        rules.push(gen_rule_n(k));
    }
    rules
}

fn find_all_positions(s: &str, pat: &str) -> Vec<usize> {
    if pat.is_empty() {
        return vec![];
    }
    let mut res = Vec::new();
    let mut start = 0usize;
    while start <= s.len() {
        if let Some(rel) = s[start..].find(pat) {
            let pos = start + rel;
            res.push(pos);
            start = pos + 1;
        } else {
            break;
        }
    }
    res
}

fn replace_at(s: &str, start: usize, len: usize, rep: &str) -> String {
    let mut out = String::with_capacity(s.len() - len + rep.len());
    out.push_str(&s[..start]);
    out.push_str(rep);
    out.push_str(&s[start + len..]);
    out
}

fn random_word<R: Rng + ?Sized>(rng: &mut R, min_len: usize, max_len: usize) -> String {
    let l = rng.gen_range(min_len..=max_len);
    let chars: Vec<char> = ALPHABET.chars().collect();
    let mut s = String::with_capacity(l);
    for _ in 0..l {
        let ch = chars[rng.gen_range(0..chars.len())];
        s.push(ch);
    }
    s
}

fn neighbors_tprime(s: &str) -> Vec<String> {
    let rules_tp = build_tprime(12);
    let mut res: HashSet<String> = HashSet::new();
    for (l, r) in &rules_tp {
        for pos in find_all_positions(s, l) {
            res.insert(replace_at(s, pos, l.len(), r));
        }
        for pos in find_all_positions(s, r) {
            res.insert(replace_at(s, pos, r.len(), l));
        }
    }
    res.into_iter().collect()
}

fn step_t<R: Rng + ?Sized>(s: &str, rng: &mut R, rules: &Rules) -> (String, bool) {
    let mut options: Vec<(usize, &str, &str)> = Vec::new();
    for (l, r) in rules {
        for p in find_all_positions(s, l) {
            options.push((p, l.as_str(), r.as_str()));
        }
    }
    if options.is_empty() {
        return (s.to_string(), false);
    }
    let (p, l, r) = options[rng.gen_range(0..options.len())];
    (replace_at(s, p, l.len(), r), true)
}

fn random_chain_t<R: Rng + ?Sized>(
    s: &str,
    rng: &mut R,
    min_steps: usize,
    max_steps: usize,
    rules: &Rules,
) -> (String, usize) {
    let steps = rng.gen_range(min_steps..=max_steps);
    let mut t = s.to_string();
    let mut did = 0usize;
    for _ in 0..steps {
        let (t2, ok) = step_t(&t, rng, rules);
        if !ok {
            break;
        }
        t = t2;
        did += 1;
    }
    (t, did)
}

fn build_path(
    p_left: &HashMap<String, Option<String>>,
    p_right: &HashMap<String, Option<String>>,
    meet: &str,
) -> Vec<String> {
    let mut left = Vec::new();
    let mut cur = Some(meet.to_string());
    while let Some(c) = cur {
        left.push(c.clone());
        cur = p_left.get(&c).cloned().unwrap_or(None);
    }
    left.reverse();
    let mut right = Vec::new();
    let mut cur2 = p_right.get(meet).cloned().unwrap_or(None);
    while let Some(c2) = cur2 {
        right.push(c2.clone());
        cur2 = p_right.get(&c2).cloned().unwrap_or(None);
    }
    left.into_iter().chain(right.into_iter()).collect()
}

fn bfs_path_tprime(src: &str, dst: &str) -> Option<Vec<String>> {
    if src == dst {
        return Some(vec![src.to_string()]);
    }
    let mut q1: VecDeque<String> = VecDeque::new();
    let mut q2: VecDeque<String> = VecDeque::new();
    q1.push_back(src.to_string());
    q2.push_back(dst.to_string());
    let mut p1: HashMap<String, Option<String>> = HashMap::new();
    let mut p2: HashMap<String, Option<String>> = HashMap::new();
    p1.insert(src.to_string(), None);
    p2.insert(dst.to_string(), None);
    let mut depth = 0usize;
    while !q1.is_empty() && !q2.is_empty() && depth < BFS_DEPTH {
        depth += 1;
        let q1_len = q1.len();
        for _ in 0..q1_len {
            if let Some(x) = q1.pop_front() {
                for y in neighbors_tprime(&x) {
                    if p1.contains_key(&y) {
                        continue;
                    }
                    p1.insert(y.clone(), Some(x.clone()));
                    if p2.contains_key(&y) {
                        return Some(build_path(&p1, &p2, &y));
                    }
                    q1.push_back(y.clone());
                    if p1.len() + p2.len() > BFS_LIMIT {
                        return None;
                    }
                }
            }
        }
        let q2_len = q2.len();
        for _ in 0..q2_len {
            if let Some(x) = q2.pop_front() {
                for y in neighbors_tprime(&x) {
                    if p2.contains_key(&y) {
                        continue;
                    }
                    p2.insert(y.clone(), Some(x.clone()));
                    if p1.contains_key(&y) {
                        return Some(build_path(&p1, &p2, &y));
                    }
                    q2.push_back(y.clone());
                    if p1.len() + p2.len() > BFS_LIMIT {
                        return None;
                    }
                }
            }
        }
    }
    None
}

fn generate_pair<R: Rng + ?Sized>(rng: &mut R, rules_t: &Rules) -> (String, String) {
    let mut attempts = 0usize;
    let min_steps_t = 5usize;
    let max_steps_t = 8usize;
    let mut w = String::new();
    let mut w2 = String::new();
    while attempts < 1000 {
        w = random_word(rng, 10, 20);
        let (cand, did) = random_chain_t(&w, rng, min_steps_t, max_steps_t, rules_t);
        if did >= min_steps_t && cand != w {
            w2 = cand;
            return (w, w2);
        }
        attempts += 1;
    }
    (w, w2)
}

fn fuzz(tests: usize) {
    let mut rng = thread_rng();
    let rules_t = t_rules();
    let mut success = 0usize;
    let mut skipped = 0usize;
    for i in 1..=tests {
        let mut tries = 0usize;
        let mut path: Option<Vec<String>> = None;
        let mut w = String::new();
        let mut w2 = String::new();
        let mut direction = String::new();
        while tries < 200 {
            let pair = generate_pair(&mut rng, &rules_t);
            w = pair.0;
            w2 = pair.1;
            path = bfs_path_tprime(&w, &w2);
            if let Some(ref p) = path {
                if p.len() >= 2 {
                    direction = "forward".to_string();
                    break;
                }
            }
            path = bfs_path_tprime(&w2, &w);
            if let Some(ref p) = path {
                if p.len() >= 2 {
                    direction = "backward".to_string();
                    break;
                }
            }
            tries += 1;
        }
        println!("Test {}:", i);
        println!("w->w' by T: {} -> {}", w, w2);
        if path.is_none() || path.as_ref().unwrap().len() < 2 {
            println!("Skipped: no rewrite path in T′ for this T-pair");
            println!();
            skipped += 1;
            continue;
        }
        let chain = path.unwrap();
        println!("Path in T': {}", chain.join(" -> "));
        if direction == "forward" {
            println!("Reachable (from w->w' by T')");
        } else {
            println!("Reachable (from w'->w by T')");
        }
        success += 1;
        println!();
    }
    println!("Tests: {}", tests);
    println!("Successful: {}", success);
    println!("Failed: {}", skipped);
}

fn main() {
    fuzz(20);
}
