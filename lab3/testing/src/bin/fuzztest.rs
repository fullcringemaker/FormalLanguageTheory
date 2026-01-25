use std::collections::HashMap;
use std::collections::HashSet;

struct Symbol {
    terminal: bool,
    value: usize,
}

struct Grammar {
    start: usize,
    nt_names: Vec<String>,
    rules_lhs: Vec<usize>,
    rules_rhs: Vec<Vec<Symbol>>,
    rules_by_lhs: Vec<Vec<usize>>,
}

struct ParseState {
    chart: Vec<HashSet<u64>>,
    agenda: Vec<Vec<u64>>,
    temp: Vec<u64>,
}

fn rng_next(seed: &mut u64) -> u64 {
    let mut x: u64 = *seed;
    x = x ^ (x << 7);
    x = x ^ (x >> 9);
    x = x ^ (x << 8);
    *seed = x;
    x
}

fn make_item(rule_id: usize, dot: usize, origin: usize) -> u64 {
    let a: u64 = rule_id as u64;
    let b: u64 = (dot as u64) << 24;
    let c: u64 = (origin as u64) << 32;
    a | b | c
}

fn item_rule(x: u64) -> usize {
    (x & 0xFFFFFFu64) as usize
}

fn item_dot(x: u64) -> usize {
    ((x >> 24) & 0xFFu64) as usize
}

fn item_origin(x: u64) -> usize {
    ((x >> 32) & 0xFFu64) as usize
}

fn ws_new(max_len: usize) -> ParseState {
    let mut chart: Vec<HashSet<u64>> = Vec::new();
    let mut agenda: Vec<Vec<u64>> = Vec::new();
    let mut i: usize = 0;
    while i <= max_len {
        chart.push(HashSet::new());
        agenda.push(Vec::new());
        i = i + 1;
    }
    ParseState {
        chart: chart,
        agenda: agenda,
        temp: Vec::new(),
    }
}

fn ws_clear(ws: &mut ParseState, used_len: usize) {
    let mut i: usize = 0;
    while i <= used_len {
        ws.chart[i].clear();
        ws.agenda[i].clear();
        i = i + 1;
    }
    ws.temp.clear();
}

fn sym_term_from_token(tok: &str) -> Option<usize> {
    if tok == "a" {
        return Some(0usize);
    }
    if tok == "b" {
        return Some(1usize);
    }
    None
}

fn get_nt_index(map: &mut HashMap<String, usize>, names: &mut Vec<String>, name: &str) -> usize {
    let key: String = name.to_string();
    let found: Option<&usize> = map.get(&key);
    if found.is_some() {
        return *found.unwrap();
    }
    let idx: usize = names.len();
    names.push(key.clone());
    map.insert(key, idx);
    idx
}

fn add_rule_tokens(
    map: &mut HashMap<String, usize>,
    names: &mut Vec<String>,
    rules_lhs: &mut Vec<usize>,
    rules_rhs: &mut Vec<Vec<Symbol>>,
    lhs: &str,
    rhs_tokens: &[&str],
) {
    let lhs_idx: usize = get_nt_index(map, names, lhs);
    let mut rhs: Vec<Symbol> = Vec::new();
    let mut i: usize = 0;
    while i < rhs_tokens.len() {
        let tok: &str = rhs_tokens[i];
        let t: Option<usize> = sym_term_from_token(tok);
        if t.is_some() {
            rhs.push(Symbol {
                terminal: true,
                value: t.unwrap(),
            });
        } else {
            let nt_idx: usize = get_nt_index(map, names, tok);
            rhs.push(Symbol {
                terminal: false,
                value: nt_idx,
            });
        }
        i = i + 1;
    }
    rules_lhs.push(lhs_idx);
    rules_rhs.push(rhs);
}

fn finalize_grammar(
    start_name: &str,
    map: HashMap<String, usize>,
    names: Vec<String>,
    rules_lhs: Vec<usize>,
    rules_rhs: Vec<Vec<Symbol>>,
) -> Grammar {
    let start_opt: Option<&usize> = map.get(&start_name.to_string());
    let start_idx: usize;
    if start_opt.is_some() {
        start_idx = *start_opt.unwrap();
    } else {
        start_idx = 0usize;
    }
    let mut rules_by_lhs: Vec<Vec<usize>> = Vec::new();
    let mut i: usize = 0;
    while i < names.len() {
        rules_by_lhs.push(Vec::new());
        i = i + 1;
    }
    let mut r: usize = 0;
    while r < rules_lhs.len() {
        let lhs_idx: usize = rules_lhs[r];
        rules_by_lhs[lhs_idx].push(r);
        r = r + 1;
    }

    Grammar {
        start: start_idx,
        nt_names: names,
        rules_lhs: rules_lhs,
        rules_rhs: rules_rhs,
        rules_by_lhs: rules_by_lhs,
    }
}

fn add_item(ws: &mut ParseState, col: usize, item: u64) {
    let inserted: bool = ws.chart[col].insert(item);
    if inserted {
        ws.agenda[col].push(item);
    }
}

fn parse_recognize(g: &Grammar, ws: &mut ParseState, word: &Vec<u8>) -> bool {
    let n: usize = word.len();
    ws_clear(ws, n);
    let start_rules: &Vec<usize> = &g.rules_by_lhs[g.start];
    let mut i: usize = 0;
    while i < start_rules.len() {
        let rid: usize = start_rules[i];
        let it: u64 = make_item(rid, 0usize, 0usize);
        add_item(ws, 0usize, it);
        i = i + 1;
    }
    let mut col: usize = 0;
    while col <= n {
        while !ws.agenda[col].is_empty() {
            let item: u64 = ws.agenda[col].pop().unwrap();
            let rid: usize = item_rule(item);
            let dot: usize = item_dot(item);
            let origin: usize = item_origin(item);
            let rhs: &Vec<Symbol> = &g.rules_rhs[rid];
            if dot < rhs.len() {
                let sym: &Symbol = &rhs[dot];
                if sym.terminal {
                    if col < n {
                        let ch: u8 = word[col];
                        if sym.value == 0usize && ch == b'a' {
                            let it2: u64 = make_item(rid, dot + 1usize, origin);
                            add_item(ws, col + 1usize, it2);
                        }
                        if sym.value == 1usize && ch == b'b' {
                            let it2: u64 = make_item(rid, dot + 1usize, origin);
                            add_item(ws, col + 1usize, it2);
                        }
                    }
                } else {
                    let nt: usize = sym.value;
                    let preds: &Vec<usize> = &g.rules_by_lhs[nt];
                    let mut p: usize = 0;
                    while p < preds.len() {
                        let pr: usize = preds[p];
                        let itp: u64 = make_item(pr, 0usize, col);
                        add_item(ws, col, itp);
                        p = p + 1;
                    }
                }
            } else {
                let completed_nt: usize = g.rules_lhs[rid];
                ws.temp.clear();

                let mut snap: Vec<u64> = Vec::new();
                for v in ws.chart[origin].iter() {
                    snap.push(*v);
                }
                let mut s: usize = 0;
                while s < snap.len() {
                    let witem: u64 = snap[s];
                    let wr: usize = item_rule(witem);
                    let wd: usize = item_dot(witem);
                    let wo: usize = item_origin(witem);
                    let wrhs: &Vec<Symbol> = &g.rules_rhs[wr];
                    if wd < wrhs.len() {
                        let ns: &Symbol = &wrhs[wd];
                        if !ns.terminal && ns.value == completed_nt {
                            let adv: u64 = make_item(wr, wd + 1usize, wo);
                            ws.temp.push(adv);
                        }
                    }

                    s = s + 1;
                }
                let mut t: usize = 0;
                while t < ws.temp.len() {
                    let adv_item: u64 = ws.temp[t];
                    add_item(ws, col, adv_item);
                    t = t + 1;
                }
            }
        }
        col = col + 1;
    }
    let mut j: usize = 0;
    while j < start_rules.len() {
        let rid: usize = start_rules[j];
        let need_dot: usize = g.rules_rhs[rid].len();
        let accept_item: u64 = make_item(rid, need_dot, 0usize);
        let ok: bool = ws.chart[n].contains(&accept_item);
        if ok {
            return true;
        }
        j = j + 1;
    }
    false
}

fn grammar_to_string(g: &Grammar) -> String {
    let mut out: String = String::new();
    out.push_str("Start symbol: ");
    out.push_str(g.nt_names[g.start].as_str());
    out.push('\n');
    let mut r: usize = 0;
    while r < g.rules_lhs.len() {
        let lhs: &String = &g.nt_names[g.rules_lhs[r]];
        out.push_str(lhs.as_str());
        out.push_str(" -> ");
        let rhs: &Vec<Symbol> = &g.rules_rhs[r];
        let mut i: usize = 0;
        while i < rhs.len() {
            if i > 0usize {
                out.push(' ');
            }
            let s: &Symbol = &rhs[i];
            if s.terminal {
                if s.value == 0usize {
                    out.push('a');
                } else {
                    out.push('b');
                }
            } else {
                out.push_str(g.nt_names[s.value].as_str());
            }
            i = i + 1;
        }
        out.push('\n');

        r = r + 1;
    }
    out
}

fn build_original_grammar() -> Grammar {
    let mut map: HashMap<String, usize> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut rules_lhs: Vec<usize> = Vec::new();
    let mut rules_rhs: Vec<Vec<Symbol>> = Vec::new();

    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S", &["a", "T", "T", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "T", &["a", "T", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "T", &["S", "b", "b", "S"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "T", &["b", "a", "b"]);
    finalize_grammar("S", map, names, rules_lhs, rules_rhs)
}

fn build_ll1_intersection_grammar() -> Grammar {
    let mut map: HashMap<String, usize> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut rules_lhs: Vec<usize> = Vec::new();
    let mut rules_rhs: Vec<Vec<Symbol>> = Vec::new();

    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S'", &["⟨0,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S'", &["⟨0,S,10⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,8⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,8⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,8⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,8⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,10⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,10⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["a", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,1⟩", &["a", "⟨3,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,1⟩", &["a", "⟨3,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,1⟩", &["a", "⟨3,T,10⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["a", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["a", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["⟨1,S,1⟩", "b", "b", "⟨7,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,1⟩", &["⟨3,S,1⟩", "b", "b", "⟨7,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,1⟩", &["⟨3,S,10⟩", "b", "b", "⟨0,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,T,10⟩", &["⟨3,S,10⟩", "b", "b", "⟨0,S,10⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["⟨8,S,1⟩", "b", "b", "⟨7,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["⟨10,S,1⟩", "b", "b", "⟨7,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,8⟩", "⟨8,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,10⟩", "⟨10,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨3,T,10⟩", "⟨10,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,8⟩", "⟨8,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,10⟩", "⟨10,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨3,S,1⟩", &["a", "⟨3,T,10⟩", "⟨10,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨1,T,8⟩", "⟨8,T,8⟩", "a"]);
    finalize_grammar("S'", map, names, rules_lhs, rules_rhs)
}

fn build_lr0_intersection_grammar() -> Grammar {
    let mut map: HashMap<String, usize> = HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut rules_lhs: Vec<usize> = Vec::new();
    let mut rules_rhs: Vec<Vec<Symbol>> = Vec::new();

    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S'", &["⟨0,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "S'", &["⟨0,S,8⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,8⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨6,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,8⟩", &["a", "b", "b", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,7⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,T,7⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,7⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,7⟩", &["b", "a", "b"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["a", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,T,1⟩", &["a", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["a", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["a", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["a", "⟨10,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["a", "⟨10,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["a", "⟨10,T,8⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,T,1⟩", &["⟨1,S,1⟩", "b", "b", "⟨6,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,T,1⟩", &["⟨7,S,1⟩", "b", "b", "⟨6,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,T,1⟩", &["⟨8,S,1⟩", "b", "b", "⟨6,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["⟨10,S,1⟩", "b", "b", "⟨6,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,1⟩", &["⟨10,S,8⟩", "b", "b", "⟨0,S,1⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,T,8⟩", &["⟨10,S,8⟩", "b", "b", "⟨0,S,8⟩"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨0,S,1⟩", &["a", "⟨10,T,8⟩", "⟨8,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨1,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨6,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨6,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨6,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨6,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨7,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨8,S,1⟩", &["a", "⟨1,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,1⟩", "⟨1,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,1⟩", "⟨1,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,7⟩", "⟨7,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,7⟩", "⟨7,T,7⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,8⟩", "⟨8,T,1⟩", "a"]);
    add_rule_tokens(&mut map, &mut names, &mut rules_lhs, &mut rules_rhs, "⟨10,S,1⟩", &["a", "⟨10,T,8⟩", "⟨8,T,7⟩", "a"]);
    finalize_grammar("S'", map, names, rules_lhs, rules_rhs)
}

fn bytes_to_string(word: &Vec<u8>) -> String {
    let mut s: String = String::new();
    let mut i: usize = 0;
    while i < word.len() {
        let ch: u8 = word[i];
        if ch == b'a' {
            s.push('a');
        } else {
            s.push('b');
        }
        i = i + 1;
    }
    s
}

fn run_tests(tests: usize) {
    let g0: Grammar = build_original_grammar();
    let gll: Grammar = build_ll1_intersection_grammar();
    let glr: Grammar = build_lr0_intersection_grammar();
    let mut ws0: ParseState = ws_new(30usize);
    let mut wsll: ParseState = ws_new(30usize);
    let mut wslr: ParseState = ws_new(30usize);
    let mut rng: u64 = 88172645463393265u64;
    let mut passed: usize = 0usize;
    for _t in 0..tests {
        let rlen: u64 = rng_next(&mut rng);
        let mut len: usize = (rlen % 25u64) as usize;
        len = len + 6usize;
        let mut word: Vec<u8> = Vec::new();
        for _i in 0..len {
            let rbit: u64 = rng_next(&mut rng);
            let bit: u64 = rbit % 2u64;
            if bit == 0u64 {
                word.push(b'a');
            } else {
                word.push(b'b');
            }
        }
        let r0: bool = parse_recognize(&g0, &mut ws0, &word);
        let rll: bool = parse_recognize(&gll, &mut wsll, &word);
        let rlr: bool = parse_recognize(&glr, &mut wslr, &word);
        let ok: bool;
        if r0 == rll && rll == rlr {
            ok = true;
        } else {
            ok = false;
        }
        if !ok {
            let wstr: String = bytes_to_string(&word);
            println!("Mismatch found.");
            println!("Word: {}", wstr);
            println!("Original CFG result: {}", r0);
            println!("LL(1)-intersection result: {}", rll);
            println!("LR(0)-intersection result: {}", rlr);
            println!();
            println!("Original CFG:");
            print!("{}", grammar_to_string(&g0));
            println!();
            println!("LL(1)-intersection CFG (LL(1) automaton: start=0, finals={{1,10}}):");
            print!("{}", grammar_to_string(&gll));
            println!();
            println!("LR(0)-intersection CFG (LR(0) automaton: start=0, finals={{1,8}}):");
            print!("{}", grammar_to_string(&glr));
            return;
        }
        passed = passed + 1usize;
    }
    println!("They are equivalent: {}/{} tests passed", passed, tests);
}

fn main() {
    let tests: usize = 100000usize;
    run_tests(tests);
}
