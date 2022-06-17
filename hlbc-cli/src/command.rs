use std::ops::Range;

use chumsky::prelude::*;
use chumsky::text::*;
pub use chumsky::Parser;

pub type IndexRange = Range<usize>;

#[derive(Debug, Clone)]
pub enum FileOrIndex {
    File(String),
    Index(usize),
}

#[derive(Debug, Clone)]
pub enum ElementRef {
    String(usize),
    Global(usize),
    Fn(usize),
}

#[derive(Debug, Clone)]
pub enum Command {
    Exit,
    Help,
    Info,
    Entrypoint,
    Explain(String),
    Int(IndexRange),
    Float(IndexRange),
    String(IndexRange),
    SearchStr(String),
    Debugfile(IndexRange),
    SearchDebugfile(String),
    Type(IndexRange),
    Global(IndexRange),
    Native(IndexRange),
    Constant(IndexRange),
    FunctionHeader(IndexRange),
    Function(IndexRange),
    FunctionNamed(String),
    SearchFunction(String),
    InFile(FileOrIndex),
    FileOf(usize),
    SaveTo(String),
    Callgraph(usize, usize),
    RefTo(ElementRef),
    DecompType(usize),
    Decomp(usize),
}

// Used a default max values for index ranges
#[derive(Debug, Default)]
pub struct ParseContext {
    pub int_max: usize,
    pub float_max: usize,
    pub string_max: usize,
    pub debug_file_max: usize,
    pub type_max: usize,
    pub global_max: usize,
    pub native_max: usize,
    pub constant_max: usize,
    pub findex_max: usize,
}

/// Parse a command
pub fn parse_command(ctx: &ParseContext, line: &str) -> Result<Command, Vec<Simple<char>>> {
    command_parser(ctx).padded().parse(line)
}

/// Parse a list of command separated by ';'
pub fn parse_commands(ctx: &ParseContext, line: &str) -> Result<Vec<Command>, Vec<Simple<char>>> {
    commands_parser(ctx).parse(line)
}

pub fn commands_parser(
    ctx: &ParseContext,
) -> impl Parser<char, Vec<Command>, Error = Simple<char>> {
    command_parser(ctx).padded().separated_by(just(';'))
}

pub fn command_parser(ctx: &ParseContext) -> impl Parser<char, Command, Error = Simple<char>> {
    use Command::*;

    macro_rules! cmd {
        ($name:expr) => {
            just($name).padded().ignored()
        };
        ($name:expr, $name_min:expr) => {
            just($name).or(just($name_min)).padded().ignored()
        };
        ($name:expr; $then:expr) => {
            just($name).padded().ignore_then($then)
        };
        ($name:expr, $name_min:expr; $then:expr) => {
            just($name).or(just($name_min)).padded().ignore_then($then)
        };
    }

    let string = string();

    // The boxing is necessary because the items in an array must be of the same type.
    // We can't use a tuple because the impl are for tuples of length 25 at most.
    choice([
        cmd!("exit").map(|_| Exit).boxed(),
        cmd!("help").map(|_| Help).boxed(),
        cmd!("info").map(|_| Info).boxed(),
        cmd!("entrypoint").map(|_| Entrypoint).boxed(),
        cmd!("explain"; string.clone()).map(Explain).boxed(),
        cmd!("int", "i"; index_range(ctx.int_max)).map(Int).boxed(),
        cmd!("float", "f"; index_range(ctx.float_max))
            .map(Float)
            .boxed(),
        cmd!("string", "s"; index_range(ctx.string_max))
            .map(String)
            .boxed(),
        cmd!("sstr"; string.clone()).map(SearchStr).boxed(),
        cmd!("debugfile", "file"; index_range(ctx.debug_file_max))
            .map(Debugfile)
            .boxed(),
        cmd!("sfile"; string.clone()).map(SearchDebugfile).boxed(),
        cmd!("type", "t"; index_range(ctx.type_max))
            .map(Type)
            .boxed(),
        cmd!("global", "g"; index_range(ctx.global_max))
            .map(Global)
            .boxed(),
        cmd!("native", "n"; index_range(ctx.native_max))
            .map(Native)
            .boxed(),
        cmd!("constant", "c"; index_range(ctx.constant_max))
            .map(Constant)
            .boxed(),
        cmd!("fnh"; index_range(ctx.findex_max))
            .map(FunctionHeader)
            .boxed(),
        cmd!("fn"; index_range(ctx.findex_max))
            .map(Function)
            .boxed(),
        cmd!("fnamed", "fnn"; string.clone())
            .map(FunctionNamed)
            .boxed(),
        cmd!("sfn"; string.clone()).map(SearchFunction).boxed(),
        cmd!("infile")
            .ignore_then(choice((
                num().map(|n| InFile(FileOrIndex::Index(n))),
                filter(|c: &char| !c.is_whitespace())
                    .repeated()
                    .map(|v| InFile(FileOrIndex::File(v.into_iter().collect()))),
            )))
            .boxed(),
        cmd!("fileof"; num()).map(FileOf).boxed(),
        cmd!("saveto"; string.clone()).map(SaveTo).boxed(),
        cmd!("callgraph")
            .ignore_then(num())
            .then(num())
            .map(|(f, d)| Callgraph(f, d))
            .boxed(),
        cmd!("refto")
            .ignore_then(choice((
                just("string@").ignore_then(num()).map(ElementRef::String),
                just("global@").ignore_then(num()).map(ElementRef::Global),
                just("fn@").ignore_then(num()).map(ElementRef::Fn),
            )))
            .map(RefTo)
            .boxed(),
        cmd!("decomptype"; num()).map(DecompType).boxed(),
        cmd!("decomp"; num()).map(Decomp).boxed(),
    ])
}

fn string() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    filter(|c: &char| c != &';')
        .repeated()
        .map(|v| v.into_iter().collect())
}

fn num() -> impl Parser<char, usize, Error = Simple<char>> {
    int::<_, Simple<char>>(10)
        .map(|s: String| s.parse::<usize>().unwrap())
        .labelled("positive integer")
}

/// Parse any range, constrained between min and max. Can also parse a single index.
/// e.g. .., ..3, 4..5, 2,..=9, 14
fn index_range(max: usize) -> impl Parser<char, IndexRange, Error = Simple<char>> {
    choice((
        // Range
        num()
            .or_not()
            .then(just("..=").or(just("..")))
            .then(num().or_not())
            .map(move |((a, range), b)| {
                let a = a.unwrap_or(0).max(0);
                if range == ".." {
                    let b = b.unwrap_or(max).min(max);
                    a..b
                } else {
                    let b = (b.unwrap_or(max - 1) + 1).min(max);
                    a..b
                }
            }),
        // Single index
        num().map(|i| i..(i + 1)),
    ))
    .labelled("index range")
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::command::{
        index_range, parse_command, parse_commands, Command, FileOrIndex, ParseContext,
    };
    use crate::parse_commands;

    #[test]
    fn test_index_range() {
        assert_eq!(
            Ok((0..10).sum::<usize>()),
            index_range(10).parse("..").map(Iterator::sum)
        );
        assert_eq!(
            Ok((0..=9).sum::<usize>()),
            index_range(10).parse("..=").map(Iterator::sum)
        );
        assert_eq!(
            Ok((0..4).sum::<usize>()),
            index_range(10).parse("..4").map(Iterator::sum)
        );
        assert_eq!(
            Ok((2..10).sum::<usize>()),
            index_range(10).parse("2..").map(Iterator::sum)
        );
        assert_eq!(
            Ok((1..5).sum::<usize>()),
            index_range(10).parse("1..5").map(Iterator::sum)
        );
        assert_eq!(
            Ok((0..=8).sum::<usize>()),
            index_range(10).parse("..=8").map(Iterator::sum)
        );
    }

    #[test]
    fn test_index_single() {
        assert_eq!(
            (4..5).sum::<usize>(),
            index_range(10).parse("4").unwrap().sum()
        );
    }

    #[test]
    fn test_command_simple() {
        let parsed = parse_command(&ParseContext::default(), "exit");
        assert!(matches!(parsed, Ok(Command::Exit)));
    }

    #[test]
    fn test_command_index() {
        let parsed = parse_command(
            &ParseContext {
                string_max: 10,
                ..Default::default()
            },
            "s ..",
        );
        assert!(matches!(parsed, Ok(Command::String(_))));
    }

    #[test]
    fn test_command_str() {
        let parsed = parse_command(&ParseContext::default(), "sstr hello world");
        assert!(match parsed {
            Ok(Command::SearchStr(s)) => {
                s == "hello world"
            }
            _ => false,
        });
    }

    #[test]
    fn test_file_or_index() {
        let parsed = parse_command(&ParseContext::default(), "infile 226");
        assert!(match parsed {
            Ok(Command::InFile(FileOrIndex::Index(n))) => {
                n == 226
            }
            _ => false,
        });
        let parsed = parse_command(&ParseContext::default(), "infile Array.hx");
        assert!(match parsed {
            Ok(Command::InFile(FileOrIndex::File(s))) => {
                s == "Array.hx"
            }
            _ => false,
        });
        // Should not take the trailing whitespaces
        let parsed = parse_command(&ParseContext::default(), "infile       Array.hx        ");
        assert!(match parsed {
            Ok(Command::InFile(FileOrIndex::File(s))) => {
                s == "Array.hx"
            }
            _ => false,
        });
    }

    #[test]
    fn test_command_list() {
        let parsed = parse_commands(
            &ParseContext::default(),
            "sstr hello world; exit    ; fnamed main",
        )
        .unwrap();
        assert!(match &parsed[0] {
            Command::SearchStr(s) => {
                s == "hello world"
            }
            _ => false,
        });
        assert!(match &parsed[1] {
            Command::Exit => true,
            _ => false,
        });
        assert!(match &parsed[2] {
            Command::FunctionNamed(s) => {
                s == "main"
            }
            _ => false,
        });
    }
}
