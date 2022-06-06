use std::ops::Range;

use chumsky::prelude::*;
use chumsky::text::*;

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
    DumpType(usize),
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

    choice((
        cmd!("exit").map(|_| Exit),
        cmd!("help").map(|_| Help),
        cmd!("info").map(|_| Info),
        cmd!("entrypoint").map(|_| Entrypoint),
        cmd!("explain"; any().repeated()).map(|v| Explain(v.into_iter().collect())),
        cmd!("int", "i"; index_range(ctx.int_max)).map(Int),
        cmd!("float", "f"; index_range(ctx.float_max)).map(Float),
        cmd!("string", "s"; index_range(ctx.string_max)).map(String),
        cmd!("sstr"; any().repeated()).map(|v| SearchStr(v.into_iter().collect())),
        cmd!("debugfile", "file"; index_range(ctx.debug_file_max)).map(Debugfile),
        cmd!("sfile"; any().repeated()).map(|v| SearchDebugfile(v.into_iter().collect())),
        cmd!("type", "t"; index_range(ctx.type_max)).map(Type),
        cmd!("global", "g"; index_range(ctx.global_max)).map(Global),
        cmd!("native", "n"; index_range(ctx.native_max)).map(Native),
        cmd!("constant", "c"; index_range(ctx.constant_max)).map(Constant),
        cmd!("fnh"; index_range(ctx.findex_max)).map(FunctionHeader),
        cmd!("fn"; index_range(ctx.findex_max)).map(Function),
        cmd!("fnamed", "fnn"; any().repeated()).map(|v| FunctionNamed(v.into_iter().collect())),
        cmd!("sfn"; any().repeated()).map(|v| SearchFunction(v.into_iter().collect())),
        cmd!("infile").ignore_then(choice((
            num().map(|n| InFile(FileOrIndex::Index(n))),
            filter(|c: &char| !c.is_whitespace())
                .repeated()
                .map(|v| InFile(FileOrIndex::File(v.into_iter().collect()))),
        ))),
        cmd!("fileof"; num()).map(FileOf),
        cmd!("saveto"; any().repeated()).map(|v| SaveTo(v.into_iter().collect())),
        cmd!("callgraph")
            .ignore_then(num())
            .then(num())
            .map(|(f, d)| Callgraph(f, d)),
        cmd!("refto")
            .ignore_then(choice((
                just("string@").ignore_then(num()).map(ElementRef::String),
                just("global@").ignore_then(num()).map(ElementRef::Global),
                just("fn@").ignore_then(num()).map(ElementRef::Fn),
            )))
            .map(RefTo),
        cmd!("dumptype"; num()).map(DumpType),
    ))
    .labelled("command")
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

    use crate::command::{index_range, parse_command, Command, FileOrIndex, ParseContext};

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
}
