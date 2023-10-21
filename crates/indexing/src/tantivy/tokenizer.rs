use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

/// Correctly tokenizes snake case and pascal case
#[derive(Clone)]
pub(crate) struct FunctionTokenizer;

struct FunctionTokenStream<'a> {
    /// Remaining text to be tokenized
    text: &'a str,
    /// Global offset at the start of tokenization
    offset: usize,
    /// Current token
    token: Token,
}

impl Tokenizer for FunctionTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        BoxTokenStream::from(FunctionTokenStream {
            text,
            offset: 0,
            token: Default::default(),
        })
    }
}

impl<'a> TokenStream for FunctionTokenStream<'a> {
    fn advance(&mut self) -> bool {
        // Reset current token
        self.token.text.clear();
        self.token.position = self.token.position.wrapping_add(1);
        // Local offset
        let mut from = 0;

        // Emit a token
        macro_rules! tok {
            ($i:expr) => {
                self.token.offset_from = self.offset + from;
                self.offset += $i;
                self.token.offset_to = self.offset;
                let (before, after) = self.text.split_at($i);
                self.text = after;
                self.token.text.push_str(&before[from..]);
                return true;
            };
        }

        let mut state = false;
        for (i, c) in self.text.char_indices() {
            if c.is_alphanumeric() {
                if state {
                    if c.is_uppercase() {
                        tok!(i);
                    }
                } else {
                    // Start new token
                    from = i;
                    state = true;
                }
            } else {
                if state {
                    tok!(i);
                }
            }
        }
        // Emit remaining token
        if state {
            tok!(self.text.len());
        }
        false
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn function_tokenizer_simple() {
        let text = "my_function otherFunction";
        let tokenizer = FunctionTokenizer;
        let mut stream = tokenizer.token_stream(text);
        let tokens: Vec<_> = (0..4)
            .filter_map(|i| stream.next().map(|t| t.text.clone()))
            .collect();
        assert_eq!(tokens.len(), 4);
        assert!(!stream.advance());
        assert_eq!(
            tokens
                .iter()
                .zip(["my", "function", "other", "Function"])
                .filter(|(a, b)| a != b)
                .count(),
            0
        );
    }
}
