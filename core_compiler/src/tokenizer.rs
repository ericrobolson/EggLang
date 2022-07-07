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
impl TokenKind {
    pub fn token_type(&self) -> TokenType {
        match self {
            TokenKind::String(_) => TokenType::String,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    String,
}

/// An error that occured while tokenizing.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizerErr {
    StringNotStarted,
    UnclosedString,
    Type(TypeError),
    StackUnderflow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    WrongType { got: State, expected: TokenType },
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
                State::String(StringState { start, contents }) => {
                    let contents = contents.trim();

                    self.tokens.push(Token {
                        kind: TokenKind::String(contents.into()),
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

    fn pop_string_state(&mut self) -> Result<StringState, Err> {
        match self.pop_state() {
            Some(state) => {
                //
                match state {
                    State::String(state) => Ok(state),
                    state => Err(self.make_err(TokenizerErr::Type(TypeError::WrongType {
                        got: state,
                        expected: TokenType::String,
                    }))),
                }
            }
            None => Err(self.make_err(TokenizerErr::StackUnderflow)),
        }
    }

    /// Creates a new tokenizer.
    pub fn parse<'a>(contents: &'a str, path: PathBuf) -> Result<Success, Err> {
        Benchy::time("Tokenizer::parse");

        const ESCAPE_CHARACTER: char = '\\';
        const QUOTE: char = '\"';

        let mut tokenizer = Self::load(contents, path);

        // TODO: start strings.
        let mut prev_char = None;
        while let Some(c) = tokenizer.next_character() {
            let is_quote = c == QUOTE;
            let prev_char_is_escape = Some(ESCAPE_CHARACTER) == prev_char;

            if tokenizer.is_making_string() {
                if is_quote && !prev_char_is_escape {
                    todo!("End string")
                } else {
                    todo!("Add to string")
                }
            } else if is_quote {
                todo!("Make new string")
            } else {
                todo!("What to do after?");
                todo!("Make identifiers");
            }

            prev_char = Some(c);
        }

        tokenizer.finalize()
    }

    /// Convert to the final form.
    fn finalize(self) -> Result<Success, Err> {
        Ok(self.tokens)
    }

    /// Creates an error of the given kind.
    fn make_err(&self, kind: TokenizerErr) -> error::Error<TokenizerErr> {
        error::Error {
            kind,
            location: self.location.clone(),
        }
    }

    /// Pops the current state off the stack.
    fn pop_state(&mut self) -> Option<State> {
        self.state_stack.pop()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    String(StringState),
}
#[derive(Clone, Debug, PartialEq)]
pub struct StringState {
    start: Location,
    contents: String,
}

impl State {
    fn token_type(&self) -> TokenType {
        match self {
            State::String(_) => TokenType::String,
        }
    }

    pub fn is_making_string(&self) -> bool {
        match self {
            State::String(StringState { start, contents }) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn pop_string_state_underflow_returns_err() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(tokenizer.make_err(TokenizerErr::StackUnderflow));
        assert_eq!(expected, tokenizer.pop_string_state());
    }

    #[test]
    fn pop_string_state_returns_top_state() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let string_state = StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::String(string_state.clone());

        tokenizer.state_stack.push(state.clone());

        assert_eq!(string_state, tokenizer.pop_string_state().unwrap());
    }

    #[test]
    fn pop_state_nothing_returns_none() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = None;
        assert_eq!(expected, tokenizer.pop_state());
    }

    #[test]
    fn make_err_returns_error() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());
        let actual = tokenizer.make_err(TokenizerErr::StackUnderflow);

        assert_eq!(
            error::Error::<TokenizerErr> {
                kind: TokenizerErr::StackUnderflow,
                location: tokenizer.location.clone()
            },
            actual
        );
    }

    #[test]
    fn pop_state_something_returns_something() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::String(StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state.clone());

        let expected = Some(state.clone());

        assert_eq!(expected, tokenizer.pop_state());
    }

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

        let state = State::String(StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        assert_eq!(Ok(()), tokenizer.make_string());
        let expected = vec![Token {
            kind: TokenKind::String(" \"hello world\"    ".into()),
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

        let state = State::String(StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

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
        todo!();
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
