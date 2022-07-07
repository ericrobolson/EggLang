use crate::{error, location::Location};
use benchy::Benchy;
use std::path::PathBuf;

pub type Err = error::Error<TokenErr>;
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
    Comment(String),
    Identifier(String),
    Number(f64),
    String(String),
    Symbol(char),
}
impl TokenKind {
    pub fn token_type(&self) -> TokenType {
        match self {
            TokenKind::Identifier(_) => TokenType::Identifier,
            TokenKind::Number(_) => TokenType::Number,
            TokenKind::String(_) => TokenType::String,
            TokenKind::Comment(_) => TokenType::Comment,
            TokenKind::Symbol(_) => TokenType::Symbol,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Comment,
    Identifier,
    Number,
    String,
    Symbol,
}

/// An error that occured while tokenizing.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenErr {
    Comment(CommentErr),
    String(StringErr),
    Type(TypeErr),
    Identifier(IdentifierErr),
    StackUnderflow,
}

/// An error that occured for a comment.
#[derive(Debug, Clone, PartialEq)]
pub enum CommentErr {
    NotStarted,
}

/// An error that occured for a string.
#[derive(Debug, Clone, PartialEq)]
pub enum StringErr {
    NotStarted,
    Unclosed(StringState),
}

/// An error that occured for a string.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierErr {
    NotStarted,
    BeginsWithNumber { got: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeErr {
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

pub const ESCAPE_CHARACTER: char = '\\';
pub const QUOTE: char = '\"';
pub const COMMENT: char = ';';
pub const NEW_LINE: char = '\n';

fn is_symbol(c: char) -> bool {
    match c {
        '(' | ')' | '[' | ']' | '{' | '}' => {
            //
            true
        }
        _ => false,
    }
}

impl Tokenizer {
    /// Parse the given contents into a series of tokens.
    pub fn parse<'a>(contents: &'a str, path: PathBuf) -> Result<Success, Err> {
        Benchy::time("Tokenizer::parse");

        let mut tokenizer = Self::load(contents, path);

        let mut prev_char = None;
        while let Some(c) = tokenizer.next_character() {
            let is_comment = c == COMMENT;
            let is_quote = c == QUOTE;
            let is_whitespace = c.is_whitespace();
            let prev_char_is_escape = Some(ESCAPE_CHARACTER) == prev_char;
            let is_newline = c == NEW_LINE;
            let is_symbol = is_symbol(c);
            let is_terminal_character = is_symbol | is_whitespace || is_comment || is_newline;

            // Handle making a string
            if tokenizer.is_making_string() {
                if is_quote && !prev_char_is_escape {
                    tokenizer.make_string()?;
                } else {
                    let mut state = tokenizer.pop_string_state()?;
                    state.contents.push(c);
                    tokenizer.state_stack.push(State::String(state));
                }
            }
            // End the string
            else if is_quote {
                if tokenizer.is_making_identifier() {
                    tokenizer.make_identifier()?;
                }

                tokenizer.state_stack.push(State::String(StringState {
                    start: tokenizer.location.clone(),
                    contents: String::new(),
                }));
            } else if is_terminal_character {
                if is_whitespace && tokenizer.state_stack.is_empty() {
                    // do nothing
                } else {
                    if tokenizer.is_making_identifier() {
                        tokenizer.make_identifier()?;
                    }

                    if is_comment {
                        tokenizer.state_stack.push(State::Comment(CommentState {
                            start: tokenizer.location.clone(),
                            contents: String::new(),
                        }));
                    }

                    if is_newline && tokenizer.is_making_comment() {
                        tokenizer.make_comment()?;
                    }

                    if is_symbol {
                        tokenizer.tokens.push(Token {
                            kind: TokenKind::Symbol(c),
                            location: tokenizer.location.clone(),
                        });
                    }
                }
            } else if tokenizer.is_making_identifier() {
                let mut state = tokenizer.pop_identifier_state()?;
                state.contents.push(c);
                tokenizer.state_stack.push(State::Identifier(state));
            } else if tokenizer.is_making_comment() {
                let mut state = tokenizer.pop_comment_state()?;
                state.contents.push(c);
                tokenizer.state_stack.push(State::Comment(state));
            } else {
                // Start identifier
                tokenizer
                    .state_stack
                    .push(State::Identifier(IdentifierState {
                        start: tokenizer.location.clone(),
                        contents: c.to_string(),
                    }));
            }

            // TODO: terminations of special characters

            prev_char = Some(c);
            tokenizer.increment_location(c);
        }

        tokenizer.finalize()
    }

    /// Returns whether the tokenizer is making a string or not.
    fn is_making_comment(&self) -> bool {
        if self.state_stack.is_empty() {
            false
        } else {
            self.state_stack[self.state_stack.len() - 1].token_type() == TokenType::Comment
        }
    }

    /// Returns whether the tokenizer is making a string or not.
    fn is_making_identifier(&self) -> bool {
        if self.state_stack.is_empty() {
            false
        } else {
            self.state_stack[self.state_stack.len() - 1].token_type() == TokenType::Identifier
        }
    }

    /// Returns whether the tokenizer is making a string or not.
    fn is_making_string(&self) -> bool {
        if self.state_stack.is_empty() {
            false
        } else {
            self.state_stack[self.state_stack.len() - 1].token_type() == TokenType::String
        }
    }

    /// Loads the given contents into the tokenizer.
    fn load<'a>(contents: &'a str, path: PathBuf) -> Self {
        let contents = contents.replace("\r\n", "\n").replace("\r", "\n");

        Self {
            tokens: vec![],
            location: Location::new(path),
            next_char_index: 0,
            original_contents: contents,
            state_stack: vec![],
        }
    }

    /// Attempts to make a string.
    fn make_comment(&mut self) -> Result<(), Err> {
        match self.state_stack.pop() {
            Some(state) => match state {
                State::Comment(CommentState { start, contents }) => {
                    let contents = contents.trim();

                    self.tokens.push(Token {
                        kind: TokenKind::Comment(contents.into()),
                        location: start,
                    });

                    Ok(())
                }

                state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                    got: { state },
                    expected: TokenType::Comment,
                }))),
            },
            None => Err(error::Error {
                kind: TokenErr::Comment(CommentErr::NotStarted),
                location: self.location.clone(),
            }),
        }
    }

    /// Attempts to make a string.
    fn make_identifier(&mut self) -> Result<(), Err> {
        match self.state_stack.pop() {
            Some(state) => match state {
                State::Identifier(IdentifierState { start, contents }) => {
                    let contents = contents.trim();

                    let contents = contents.replace("\\\"", "\"");

                    // Try to parse number
                    match contents.parse::<f64>() {
                        Ok(n) => {
                            self.tokens.push(Token {
                                kind: TokenKind::Number(n),
                                location: start,
                            });

                            return Ok(());
                        }
                        _ => {
                            // Ensure that the identifier doesn't start with a number
                            if let Some(c) = contents.chars().nth(0) {
                                if c.is_numeric() {
                                    return Err(error::Error {
                                        location: start,
                                        kind: TokenErr::Identifier(
                                            IdentifierErr::BeginsWithNumber { got: contents },
                                        ),
                                    });
                                }
                            }
                        }
                    }

                    self.tokens.push(Token {
                        kind: TokenKind::Identifier(contents.into()),
                        location: start,
                    });

                    Ok(())
                }

                state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                    got: { state },
                    expected: TokenType::Identifier,
                }))),
            },
            None => Err(error::Error {
                kind: TokenErr::Identifier(IdentifierErr::NotStarted),
                location: self.location.clone(),
            }),
        }
    }

    /// Attempts to make a string.
    fn make_string(&mut self) -> Result<(), Err> {
        match self.state_stack.pop() {
            Some(state) => match state {
                State::String(StringState { start, contents }) => {
                    let contents = contents.trim();

                    let contents = contents.replace("\\\"", "\"");

                    self.tokens.push(Token {
                        kind: TokenKind::String(contents.into()),
                        location: start,
                    });

                    Ok(())
                }

                state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                    got: { state },
                    expected: TokenType::String,
                }))),
            },
            None => Err(error::Error {
                kind: TokenErr::String(StringErr::NotStarted),
                location: self.location.clone(),
            }),
        }
    }

    /// Returns the next character in the contents.
    fn next_character(&mut self) -> Option<char> {
        self.original_contents.chars().nth(self.next_char_index)
    }

    /// Convert to the final form.
    fn finalize(mut self) -> Result<Success, Err> {
        while let Ok(state) = self.pop_state() {
            match state {
                State::String(state) => {
                    return Err(self.make_err(TokenErr::String(StringErr::Unclosed(state))));
                }
                State::Identifier(state) => {
                    self.state_stack.push(State::Identifier(state));
                    self.make_identifier()?;
                }
                State::Comment(state) => {
                    self.state_stack.push(State::Comment(state));
                    self.make_comment()?;
                }
            }
        }

        Ok(self.tokens)
    }

    /// Increments the location for the given character.
    fn increment_location(&mut self, c: char) {
        // Increment if next character exists
        self.next_char_index += 1;

        // Move location if it's a new line.
        if c == '\n' {
            self.location.column = 0;
            self.location.line += 1;
        } else {
            self.location.column += 1;
        }
    }

    /// Creates an error of the given kind.
    fn make_err(&self, kind: TokenErr) -> error::Error<TokenErr> {
        error::Error {
            kind,
            location: self.location.clone(),
        }
    }

    /// Attempts to pop off a string state.
    fn pop_comment_state(&mut self) -> Result<CommentState, Err> {
        match self.pop_state()? {
            State::Comment(state) => Ok(state),
            state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                got: state,
                expected: TokenType::Comment,
            }))),
        }
    }

    /// Attempts to pop off a string state.
    fn pop_identifier_state(&mut self) -> Result<IdentifierState, Err> {
        match self.pop_state()? {
            State::Identifier(state) => Ok(state),
            state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                got: state,
                expected: TokenType::Identifier,
            }))),
        }
    }

    /// Pops the current state off the stack.
    fn pop_state(&mut self) -> Result<State, Err> {
        match self.state_stack.pop() {
            Some(s) => Ok(s),
            None => Err(self.make_err(TokenErr::StackUnderflow)),
        }
    }

    /// Attempts to pop off a string state.
    fn pop_string_state(&mut self) -> Result<StringState, Err> {
        match self.pop_state()? {
            State::String(state) => Ok(state),
            state => Err(self.make_err(TokenErr::Type(TypeErr::WrongType {
                got: state,
                expected: TokenType::String,
            }))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Comment(CommentState),
    String(StringState),
    Identifier(IdentifierState),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommentState {
    start: Location,
    contents: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IdentifierState {
    start: Location,
    contents: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StringState {
    start: Location,
    contents: String,
}

impl State {
    fn token_type(&self) -> TokenType {
        match self {
            State::Comment(_) => TokenType::Comment,
            State::Identifier(_) => TokenType::Identifier,
            State::String(_) => TokenType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn increment_location_does_not_increment_line() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        tokenizer.increment_location('a');
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);

        tokenizer.increment_location('\n');
        assert_eq!(0, tokenizer.location.column);
        assert_eq!(1, tokenizer.location.line);
    }

    #[test]
    fn is_making_comment_returns_false() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(false, tokenizer.is_making_comment());
    }

    #[test]
    fn is_making_comment_returns_true() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::Comment(CommentState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

        assert_eq!(true, tokenizer.is_making_comment());
    }

    #[test]
    fn is_making_identifier_returns_false() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(false, tokenizer.is_making_identifier());
    }

    #[test]
    fn is_making_identifier_returns_true() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::Identifier(IdentifierState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

        assert_eq!(true, tokenizer.is_making_identifier());
    }

    #[test]
    fn is_making_string_returns_false() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(false, tokenizer.is_making_string());
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
    fn make_err_returns_error() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let tokenizer = Tokenizer::load(contents, path.clone());
        let actual = tokenizer.make_err(TokenErr::StackUnderflow);

        assert_eq!(
            error::Error::<TokenErr> {
                kind: TokenErr::StackUnderflow,
                location: tokenizer.location.clone()
            },
            actual
        );
    }

    #[test]
    fn make_comment_creates_string() {
        let contents = " \"jajajaja\"    ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::Comment(CommentState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

        assert_eq!(Ok(()), tokenizer.make_comment());
        let expected = vec![Token {
            kind: TokenKind::Comment("jajajaja".into()),
            location: Location {
                line: 0,
                column: 0,
                path: path,
            },
        }];

        assert_eq!(expected, tokenizer.tokens)
    }

    #[test]
    fn make_comment_returns_err_when_no_comment() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(error::Error {
            kind: TokenErr::Comment(CommentErr::NotStarted),
            location: tokenizer.location.clone(),
        });
        assert_eq!(expected, tokenizer.make_comment());
    }

    #[test]
    fn make_identifier_creates_identifier() {
        let contents = " \"jajajaja\"    ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::Identifier(IdentifierState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

        assert_eq!(Ok(()), tokenizer.make_identifier());
        let expected = vec![Token {
            kind: TokenKind::Identifier("jajajaja".into()),
            location: Location {
                line: 0,
                column: 0,
                path: path,
            },
        }];

        assert_eq!(expected, tokenizer.tokens)
    }

    #[test]
    fn make_identifier_returns_err_when_no_identifier() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(error::Error {
            kind: TokenErr::Identifier(IdentifierErr::NotStarted),
            location: tokenizer.location.clone(),
        });
        assert_eq!(expected, tokenizer.make_identifier());
    }

    #[test]
    fn make_string_creates_string() {
        let contents = " \"jajajaja\"    ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let state = State::String(StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        });

        tokenizer.state_stack.push(state);

        assert_eq!(Ok(()), tokenizer.make_string());
        let expected = vec![Token {
            kind: TokenKind::String("jajajaja".into()),
            location: Location {
                line: 0,
                column: 0,
                path: path,
            },
        }];

        assert_eq!(expected, tokenizer.tokens)
    }

    #[test]
    fn make_string_returns_err_when_no_string() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(error::Error {
            kind: TokenErr::String(StringErr::NotStarted),
            location: tokenizer.location.clone(),
        });
        assert_eq!(expected, tokenizer.make_string());
    }

    #[test]
    fn next_character_adds_newlines() {
        let contents = "a\nb";
        let path = PathBuf::from("WUT");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(Some('a'), tokenizer.next_character());
        tokenizer.increment_location('a');

        assert_eq!(1, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);

        assert_eq!(Some('\n'), tokenizer.next_character());
        tokenizer.increment_location('\n');

        assert_eq!(2, tokenizer.next_char_index);
        assert_eq!(0, tokenizer.location.column);
        assert_eq!(1, tokenizer.location.line);

        assert_eq!(Some('b'), tokenizer.next_character());
        tokenizer.increment_location('b');

        assert_eq!(3, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(1, tokenizer.location.line);
    }

    #[test]
    fn next_character_existant_returns_character() {
        let contents = "a";
        let path = PathBuf::from("WUT");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(Some('a'), tokenizer.next_character());

        tokenizer.increment_location('a');

        assert_eq!(1, tokenizer.next_char_index);
        assert_eq!(1, tokenizer.location.column);
        assert_eq!(0, tokenizer.location.line);
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
    fn parse_comment_ends_with_end_of_file() {
        let contents = ";foo";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![Token {
            kind: TokenKind::Comment("foo".into()),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        }]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_comment_ends_with_newline() {
        let contents = ";foo\nident_test";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Comment("foo".into()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Identifier("ident_test".into()),
                location: Location {
                    line: 1,
                    column: 0,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_empty_returns_empty() {
        let contents = "     ";

        let actual = Tokenizer::parse(contents, PathBuf::default());

        let expected = Ok(vec![]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_symbol_ends_when_no_trailing_chars() {
        let c = '(';
        let contents = c.to_string();
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(&contents, path.clone());
        let expected = Ok(vec![Token {
            kind: TokenKind::Symbol(c),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        }]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_symbol_ends_identifier() {
        let contents = "h(";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(&contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Identifier('h'.to_string()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Symbol('('),
                location: Location {
                    line: 0,
                    column: 1,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_symbol_ends_identifiers() {
        let contents = "h(()asd)fff";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(&contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Identifier("h".into()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Symbol('('),
                location: Location {
                    line: 0,
                    column: 1,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Symbol('('),
                location: Location {
                    line: 0,
                    column: 2,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Symbol(')'),
                location: Location {
                    line: 0,
                    column: 3,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Identifier("asd".into()),
                location: Location {
                    line: 0,
                    column: 4,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Symbol(')'),
                location: Location {
                    line: 0,
                    column: 7,
                    path: "HelloPath".into(),
                },
            },
            Token {
                kind: TokenKind::Identifier("fff".to_string()),
                location: Location {
                    line: 0,
                    column: 8,
                    path: "HelloPath".into(),
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_identifier_ends_when_no_trailing_chars() {
        let contents = "foo";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![Token {
            kind: TokenKind::Identifier("foo".into()),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        }]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_identifier_ends_with_comment() {
        let contents = "test_ident;foo";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Identifier("test_ident".into()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Comment("foo".into()),
                location: Location {
                    line: 0,
                    column: 10,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_identifier_ends_with_space() {
        let contents = "foo bar";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Identifier("foo".into()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Identifier("bar".into()),
                location: Location {
                    line: 0,
                    column: 4,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_identifier_ends_with_string() {
        let contents = "foo\"bar\"";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Identifier("foo".into()),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::String("bar".into()),
                location: Location {
                    line: 0,
                    column: 3,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_identifier_returns_err_if_begins_with_number() {
        let contents = "12345FooBar";
        let path = PathBuf::from("1234HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Err(error::Error {
            kind: TokenErr::Identifier(IdentifierErr::BeginsWithNumber {
                got: "12345FooBar".into(),
            }),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        });
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_nested_string() {
        let contents = r#""\"hello \ world!\"""#;
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![Token {
            kind: TokenKind::String("\"hello \\ world!\"".into()),
            location: Location {
                line: 0,
                column: 0,
                path,
            },
        }]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_single_string() {
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
    fn parse_number_returns_number_from_int() {
        let contents = "12345 6780";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Number(12345.0),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Number(6780.0),
                location: Location {
                    line: 0,
                    column: 6,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_number_returns_number_from_float() {
        let contents = "12345.033 -6.780";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Ok(vec![
            Token {
                kind: TokenKind::Number(12345.033),
                location: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
            },
            Token {
                kind: TokenKind::Number(-6.78),
                location: Location {
                    line: 0,
                    column: 10,
                    path,
                },
            },
        ]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_unclosed_string_returns_err() {
        let contents = "\"hello \n world!";
        let path = PathBuf::from("HelloPath");

        let actual = Tokenizer::parse(contents, path.clone());
        let expected = Err(error::Error {
            kind: TokenErr::String(StringErr::Unclosed(StringState {
                start: Location {
                    line: 0,
                    column: 0,
                    path: path.clone(),
                },
                contents: "hello \n world!".into(),
            })),
            location: Location {
                line: 1,
                column: 7,
                path: path.clone(),
            },
        });
        assert_eq!(expected, actual);
    }

    #[test]
    fn pop_comment_state_returns_top_state() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let string_state = CommentState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::Comment(string_state.clone());

        tokenizer.state_stack.push(state.clone());

        assert_eq!(string_state, tokenizer.pop_comment_state().unwrap());
    }

    #[test]
    fn pop_comment_state_returns_wrong_type() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let string_state = StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::String(string_state.clone());

        tokenizer.state_stack.push(state.clone());

        let expected = Err(tokenizer.make_err(TokenErr::Type(TypeErr::WrongType {
            got: state,
            expected: TokenType::Comment,
        })));
        assert_eq!(expected, tokenizer.pop_comment_state());
    }

    #[test]
    fn pop_comment_state_underflow_returns_err() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(tokenizer.make_err(TokenErr::StackUnderflow));
        assert_eq!(expected, tokenizer.pop_comment_state());
    }

    #[test]
    fn pop_identifier_state_returns_top_state() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let string_state = IdentifierState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::Identifier(string_state.clone());

        tokenizer.state_stack.push(state.clone());

        assert_eq!(string_state, tokenizer.pop_identifier_state().unwrap());
    }

    #[test]
    fn pop_identifier_state_returns_wrong_type() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let string_state = StringState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::String(string_state.clone());

        tokenizer.state_stack.push(state.clone());

        let expected = Err(tokenizer.make_err(TokenErr::Type(TypeErr::WrongType {
            got: state,
            expected: TokenType::Identifier,
        })));
        assert_eq!(expected, tokenizer.pop_identifier_state());
    }

    #[test]
    fn pop_identifier_state_underflow_returns_err() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(tokenizer.make_err(TokenErr::StackUnderflow));
        assert_eq!(expected, tokenizer.pop_identifier_state());
    }

    #[test]
    fn pop_state_nothing_returns_none() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        assert_eq!(
            tokenizer.make_err(TokenErr::StackUnderflow),
            tokenizer.pop_state().unwrap_err()
        );
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
    fn pop_string_state_returns_wrong_type() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let id_state = IdentifierState {
            start: tokenizer.location.clone(),
            contents: "jajajaja".into(),
        };
        let state = State::Identifier(id_state.clone());

        tokenizer.state_stack.push(state.clone());

        let expected = Err(tokenizer.make_err(TokenErr::Type(TypeErr::WrongType {
            got: state,
            expected: TokenType::String,
        })));
        assert_eq!(expected, tokenizer.pop_string_state());
    }

    #[test]
    fn pop_string_state_underflow_returns_err() {
        let contents = "     ";
        let path = PathBuf::from("wutup");
        let mut tokenizer = Tokenizer::load(contents, path.clone());

        let expected = Err(tokenizer.make_err(TokenErr::StackUnderflow));
        assert_eq!(expected, tokenizer.pop_string_state());
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

        let expected = Ok(state.clone());

        assert_eq!(expected, tokenizer.pop_state());
    }
}
