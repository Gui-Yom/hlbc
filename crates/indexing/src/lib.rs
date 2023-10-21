use std::cmp::Ordering;
use std::collections::BinaryHeap;

use hlbc::types::RefFun;
use hlbc::Bytecode;

#[cfg(feature = "tantivy")]
mod tantivy;

pub trait Searcher {
    fn search(&self, code: &Bytecode, needle: &str, limit: usize) -> Vec<RefFun>;
}

struct Comp<T>(T, f32);

impl<T> Eq for Comp<T> {}

impl<T> PartialEq<Self> for Comp<T> {
    fn eq(&self, other: &Comp<T>) -> bool {
        self.1.eq(&other.1)
    }
}

impl<T> PartialOrd<Self> for Comp<T> {
    fn partial_cmp(&self, other: &Comp<T>) -> Option<Ordering> {
        other.1.partial_cmp(&self.1)
    }
}

impl<T> Ord for Comp<T> {
    fn cmp(&self, other: &Comp<T>) -> Ordering {
        other.1.total_cmp(&self.1)
    }
}

pub fn top_candidates<'a, T>(n: usize, results: impl Iterator<Item = (T, f32)>) -> Vec<(T, f32)> {
    let mut top = BinaryHeap::with_capacity(n + 1);
    for (c, score) in results {
        if score > 0.0 {
            top.push(Comp(c, score));
            if top.len() > n {
                top.pop();
            }
        }
    }
    top.into_sorted_vec()
        .into_iter()
        .map(|c| (c.0, c.1))
        .collect()
}

pub struct Contains;

impl Searcher for Contains {
    fn search(&self, code: &Bytecode, needle: &str, limit: usize) -> Vec<RefFun> {
        let needle_len = needle.len() as f32;
        top_candidates(
            limit,
            code.functions().map(|f| {
                let name = f.name(code);
                let len = name.len() as f32;
                (
                    f.findex(),
                    if name.contains(needle) {
                        needle_len / len
                    } else if needle.contains(&*name) {
                        len / needle_len
                    } else {
                        0.0
                    },
                )
            }),
        )
        .into_iter()
        .map(|(c, s)| c)
        .collect()
    }
}

// pub struct Memchr;
//
// impl Searcher for Memchr {
//     fn with_needle<'a>(&self, needle: &'a str) -> Box<dyn Matcher + 'a> {
//         Box::new(MemchrMatcher(memchr::memmem::Finder::new(needle)))
//     }
// }
//
// pub struct MemchrMatcher<'a>(memchr::memmem::Finder<'a>);
//
// impl Matcher for MemchrMatcher<'_> {
//     fn eval(&self, candidate: &str) -> f32 {
//         if self.0.find(candidate.as_bytes()).is_some() {
//             self.0.needle().len() as f32 / candidate.len() as f32
//         } else if memchr::memmem::find(self.0.needle(), candidate.as_bytes()).is_some() {
//             candidate.len() as f32 / self.0.needle().len() as f32
//         } else {
//             0.0
//         }
//     }
// }

pub struct ClangdSearcher(fuzzy_matcher::clangd::ClangdMatcher);

impl ClangdSearcher {
    pub fn new() -> Self {
        Self(fuzzy_matcher::clangd::ClangdMatcher::default().ignore_case())
    }
}

impl Searcher for ClangdSearcher {
    fn search(&self, code: &Bytecode, needle: &str, limit: usize) -> Vec<RefFun> {
        top_candidates(
            limit,
            code.functions().map(|f| {
                (
                    f.findex(),
                    fuzzy_matcher::FuzzyMatcher::fuzzy_match(&self.0, &f.name(code), needle)
                        .map(|s| s as f32)
                        .unwrap_or(0.0),
                )
            }),
        )
        .into_iter()
        .map(|(c, s)| c)
        .collect()
    }
}

pub struct SkimSearcher(fuzzy_matcher::skim::SkimMatcherV2);

impl SkimSearcher {
    pub fn new() -> Self {
        Self(fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case())
    }
}

impl Searcher for SkimSearcher {
    fn search(&self, code: &Bytecode, needle: &str, limit: usize) -> Vec<RefFun> {
        top_candidates(
            limit,
            code.functions().map(|f| {
                (
                    f.findex(),
                    fuzzy_matcher::FuzzyMatcher::fuzzy_match(&self.0, &f.name(code), needle)
                        .map(|s| s as f32)
                        .unwrap_or(0.0),
                )
            }),
        )
        .into_iter()
        .map(|(c, s)| c)
        .collect()
    }
}
