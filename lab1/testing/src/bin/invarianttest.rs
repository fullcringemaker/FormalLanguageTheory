use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

const ALPHABET: &str = "abcd";
const BFS_LIMIT: usize = 30_000;
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

fn count_letters(w: &str) -> (i32, i32, i32, i32) {
    let mut a = 0;
    let mut b = 0;
    let mut c = 0;
    let mut d = 0;
    for ch in w.chars() {
        match ch {
            'a' => a += 1,
            'b' => b += 1,
            'c' => c += 1,
            'd' => d += 1,
            _ => {}
        }
    }
    (a, b, c, d)
}

fn inv_i(w: &str) -> i32 {
    (w.len() as i32) % 2
}

fn inv_p(w: &str) -> i32 {
    let (a, b, c, d) = count_letters(w);
    (a + c - b - d).rem_euclid(2)
}

fn inv_m(w: &str) -> i32 {
    let (a, b, c, d) = count_letters(w);
    (a + c - b - d).rem_euclid(4)
}

fn inv_vec(w: &str) -> (i32, i32, i32) {
    (inv_i(w), inv_p(w), inv_m(w))
}

#[derive(Clone, Copy)]
struct RuleChecks {
    i: bool,
    p: bool,
    m: bool,
}

fn check_rules(rules: &Rules) -> RuleChecks {
    let mut ok_i = true;
    let mut ok_p = true;
    let mut ok_m = true;
    for (l, r) in rules {
        let (li, lp, lm) = inv_vec(l);
        let (ri, rp, rm) = inv_vec(r);
        if li != ri {
            ok_i = false;
        }
        if lp != rp {
            ok_p = false;
        }
        if lm != rm {
            ok_m = false;
        }
    }
    RuleChecks { i: ok_i, p: ok_p, m: ok_m }
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

fn step_t<R: Rng + ?Sized>(s: &str, rng: &mut R, rules: &Rules) -> (String, bool) {
    let mut opts: Vec<(usize, &str, &str)> = Vec::new();
    for (l, r) in rules {
        for p in find_all_positions(s, l) {
            opts.push((p, l.as_str(), r.as_str()));
        }
    }
    if opts.is_empty() {
        return (s.to_string(), false);
    }
    let (p, l, r) = opts[rng.gen_range(0..opts.len())];
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

fn step_tp<R: Rng + ?Sized>(s: &str, rng: &mut R, rules_tp: &Rules) -> (String, bool) {
    let mut opts: Vec<(usize, &str, &str)> = Vec::new();
    for (l, r) in rules_tp {
        for p in find_all_positions(s, l) {
            opts.push((p, l.as_str(), r.as_str()));
        }
    }
    if opts.is_empty() {
        return (s.to_string(), false);
    }
    let (p, l, r) = opts[rng.gen_range(0..opts.len())];
    (replace_at(s, p, l.len(), r), true)
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

fn random_chain_tp<R: Rng + ?Sized>(
    s: &str,
    rng: &mut R,
    rules_tp: &Rules,
    min_steps: usize,
    max_steps: usize,
) -> Vec<String> {
    let steps = rng.gen_range(min_steps..=max_steps);
    let mut t = s.to_string();
    let mut chain = vec![t.clone()];
    for _ in 0..steps {
        let (t2, ok) = step_tp(&t, rng, rules_tp);
        if !ok {
            break;
        }
        t = t2;
        chain.push(t.clone());
    }
    chain
}

fn neighbors_tprime(s: &str, rules_tp: &Rules) -> Vec<String> {
    let mut res: HashSet<String> = HashSet::new();
    for (l, r) in rules_tp {
        for pos in find_all_positions(s, l) {
            res.insert(replace_at(s, pos, l.len(), r));
        }
        for pos in find_all_positions(s, r) {
            res.insert(replace_at(s, pos, r.len(), l));
        }
    }
    res.into_iter().collect()
}

fn bfs_path_tprime(src: &str, dst: &str, rules_tp: &Rules) -> Option<Vec<String>> {
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
                for y in neighbors_tprime(&x, rules_tp) {
                    if p1.contains_key(&y) {
                        continue;
                    }
                    p1.insert(y.clone(), Some(x.clone()));
                    if p2.contains_key(&y) {
                        let meet = y.clone();
                        let mut left: Vec<String> = Vec::new();
                        let mut cur = Some(meet.clone());
                        while let Some(c) = cur {
                            left.push(c.clone());
                            cur = p1.get(&c).cloned().unwrap_or(None);
                        }
                        left.reverse();
                        let mut right: Vec<String> = Vec::new();
                        let mut cur2 = p2.get(&meet).cloned().unwrap_or(None);
                        while let Some(c2) = cur2 {
                            right.push(c2.clone());
                            cur2 = p2.get(&c2).cloned().unwrap_or(None);
                        }
                        left.extend(right);
                        return Some(left);
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
                for y in neighbors_tprime(&x, rules_tp) {
                    if p2.contains_key(&y) {
                        continue;
                    }
                    p2.insert(y.clone(), Some(x.clone()));
                    if p1.contains_key(&y) {
                        let meet = y.clone();
                        let mut left: Vec<String> = Vec::new();
                        let mut cur = Some(meet.clone());
                        while let Some(c) = cur {
                            left.push(c.clone());
                            cur = p1.get(&c).cloned().unwrap_or(None);
                        }
                        left.reverse();
                        let mut right: Vec<String> = Vec::new();
                        let mut cur2 = p2.get(&meet).cloned().unwrap_or(None);
                        while let Some(c2) = cur2 {
                            right.push(c2.clone());
                            cur2 = p2.get(&c2).cloned().unwrap_or(None);
                        }
                        left.extend(right);
                        return Some(left);
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

fn status(v: bool) -> &'static str {
    if v { "OK" } else { "FAIL" }
}

fn print_rules(title: &str, res: RuleChecks) {
    println!("{}", title);
    println!("I: {}", status(res.i));
    println!("P: {}", status(res.p));
    println!("M: {}", status(res.m));
    println!();
}

fn check_ends(chain: &[String]) -> (bool, bool, bool) {
    let w0 = &chain[0];
    let w_n = &chain[chain.len() - 1];
    let v0 = inv_vec(w0);
    let v_n = inv_vec(w_n); 
    (v0.0 == v_n.0, v0.1 == v_n.1, v0.2 == v_n.2)
}

fn join_chain(chain: &[String]) -> String {
    chain.join(" -> ")
}

fn fuzz_tp(tests: usize, steps_min: usize, steps_max: usize) {
    let mut rng = thread_rng();
    let rules_tp = build_tprime(12);
    let r_t = check_rules(&t_rules());
    let r_tp = check_rules(&rules_tp);
    print_rules("Invariant check for rules T:", r_t);
    print_rules("Invariant check for rules T′:", r_tp);
    let mut success = 0usize;
    let mut fail = 0usize;
    for i in 1..=tests {
        let mut tries = 0usize;
        let mut chain: Option<Vec<String>> = None;
        let mut w0 = String::new();
        let mut w2 = String::new();
        while tries < 200 {
            let pair = generate_pair(&mut rng, &t_rules());
            w0 = pair.0;
            w2 = pair.1;
            let c = random_chain_tp(&w0, &mut rng, &rules_tp, steps_min, steps_max);
            if c.len() >= 2 {
                chain = Some(c);
                break;
            }
            tries += 1;
        }
        println!("Test {}", i);
        if chain.is_none() {
            println!();
            continue; 
        }
        let ch = chain.unwrap();
        println!("Chain: {}", join_chain(&ch));
        let (ok_i, ok_p, ok_m) = check_ends(&ch);
        println!("Invariant I: {}", status(ok_i));
        println!("Invariant P: {}", status(ok_p));
        println!("Invariant M: {}", status(ok_m));
        if ok_i && ok_p && ok_m {
            println!("All invariants satisfied");
            success += 1;
        } else {
            println!("Invariant violation found");
            fail += 1;
        }
        let _path = bfs_path_tprime(&w0, &w2, &rules_tp);
        println!();
    }
    println!("Tests: {}", tests);
    println!("Successful: {}", success);
    println!("Failed: {}", fail);
}

fn main() {
    fuzz_tp(20, 5, 10);
}
