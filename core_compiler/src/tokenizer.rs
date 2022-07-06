use crate::{error, location::Location};
use benchy::Benchy;
use std::path::PathBuf;

pub type Err = error::Error<TokenizerErr>;
pub type Success = Vec<Token>;

/// Represents a single token.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
}

/// Represents the particular kind of token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    String(String),
}

/// An error that occured while tokenizing.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerErr {
    StringNotStarted,
}

/// State for tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer {
    tokens: Success,
    location: Location,
    next_char_index: usize,
    original_contents: String,
    state_stack: Vec<State>,
}

impl Tokenizer {
    /// Returns whether the tokenizer is making a string or not.
    pub fn is_making_string(&self) -> bool {
        if self.state_stack.is_empty() {
            false
        } else {
            self.state_stack[self.state_stack.len() - 1].is_making_string()
        }
    }

    /// Loads the given contents into the tokenizer.
    pub fn load<'a>(contents: &'a str, path: PathBuf) -> Self {
        let contents = contents.replace("\r\n", "\n").replace("\r", "\n");

        Self {
            tokens: vec![],
            location: Location::new(path),
            next_char_index: 0,
            original_contents: contents,
            state_stack: vec![],
        }
    }

    // TODO: test
    pub fn make_string(&mut self) -> Result<(), Err> {
        match self.state_stack.pop() {
            Some(state) => match state {
                State::String { start, contents } => {
                    self.tokens.push(Token {
                        kind: TokenKind::String(contents),
                        location: start,
                    });

                    Ok(())
                }
            },
            None => Err(error::Error {
                kind: TokenizerErr::StringNotStarted,
                location: self.location.clone(),
            }),
        }
    }

    /// Returns the next character in the contents.
    pub fn next_character(&mut self) -> Option<char> {
        let next_index = self.next_char_index;

        let c = self.original_contents.chars().nth(next_index);

        // Increment if next character exists
        if let Some(c) = c {
            self.next_char_index += 1;

            // Move location if it's a new line.
            if c == '\n' {
                self.location.column = 0;
                self.location.line += 1;
            } else {
                self.location.column += 1;
            }
        }

        c
    }

    /// Creates a new tokenizer.
    pub fn parse<'a>(contents: &'a str, path: PathBuf) -> Result<Success, Err> {
        Benchy::time("Tokenizer::parse");

        let mut tokenizer = Self::load(contents, path);

        // TODO: start strings.

        while let Some(c) = tokenizer.next_character() {
            if c == '\"' {
                if tokenizer.is_making_string() {}
            }
        }

        tokenizer.finalize()
    }

    /// Convert to the final form.
    fn finalize(self) -> Result<Success, Err> {
        Ok(self.tokens)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum State {
    String { start: Location, contents: String },
}

impl State {
    pub fn is_making_string(&self) -> bool {
        match self {
            State::String { start, contents } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn make_string_returns_err_when_no_string() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(error::Error {
            kind: TokenizerErr::StringNotStarted,
            location: tokenizer.location.clone(),
        });
        assert_eq!(expected, tokenizer.make_string());
    }

    #[test]
    fn make_string_creates_string() {
        let contents = " \"hello world\"    ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        todo!("Need to start string");
        todo!("Maybe convert to FSM where you can easily pop things?");

        assert_eq!(Ok(()), tokenizer.make_string());
        let expected = vec![Token {
            kind: TokenKind::String("hello world".into()),
            location: Location {
                line: 0,
                column: 0,
                path: path,
            },
        }];

        assert_eq!(expected, tokenizer.tokens)
    }

    #[test]
    fn is_making_string_returns_true() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());
        tokenizer.state_stack.push(State::String {
            start: tokenizer.location.clone(),
            contents: String::new(),
        });

        assert_eq!(true, tokenizer.is_making_string());
    }

    #[test]
    fn is_making_string_returns_false() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(false, tokenizer.is_making_string());
    }

    #[test]
    fn tokenize_empty_returns_empty() {
        let contents = "     ";

        let actual = Tokenizer::parse(contents, PathBuf::default());

        let expected = Ok(vec![]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn load_replaces_r() {
        let contents = "\r\n \r \n \r \n \r\n";
        let path = PathBuf::from("WUT");
        let actual = Tokenizer::load(contents, path.clone());
        let expected = Tokenizer {
            location: Location::new(path),
            state_stack: vec![],
            tokens: vec![],
            original_contents: "\n \n \n \n \n \n".into(),
            next_char_index: 0,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn load_replaces_rn() {
        let contents = "\r\n \n \n \r\n";
        let path = PathBuf::from("WUT");
        let actual = Tokenizer::load(contents, path.clone());
        let expected = Tokenizer {
            location: Location::new(path),
            state_stack: vec![],
            tokens: vec![],
            original_contents: "\n \n \n \n".into(),
            next_char_index: 0,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn next_character_nothing_returns_none() {
        let contents = "";
        let path = PathBuf::from("WUT");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(None, tokenizer.next_character());
        assert_eq!(0, tokenizer.next_char_index);
        assert_eq!(0, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);
    }

    #[test]
    fn next_character_existant_returns_character() {
        let contents = "a";
        let path = PathBuf::from("WUT");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(Some('a'), tokenizer.next_character());
        assert_eq!(1, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);
    }

    #[test]
    fn next_character_adds_newlines() {
        let contents = "a\nb";
        let path = PathBuf::from("WUT");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(Some('a'), tokenizer.next_character());
        assert_eq!(1, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);

        assert_eq!(Some('\n'), tokenizer.next_character());
        assert_eq!(2, tokenizer.next_char_index);
        assert_eq!(0, tokenizer.location.column);
        assert_eq!(1, tokenizer.location.line);

        assert_eq!(Some('b'), tokenizer.next_character());
        assert_eq!(3, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(1, tokenizer.location.line);
    }

    #[test]
    fn tokenize_single_string() {
        let contents = "\"hello world!\"";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![Token {
            kind: TokenKind::String("hello world!".into()),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        }]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_nested_string() {}

    #[test]
    fn tokenize_unclosed_string_returns_err() {}
}
