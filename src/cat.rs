#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub msg: String,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompilerState {
    tokens: Vec<Token>,
    locations: Vec<Location>,
}
impl CompilerState {
    pub fn new() -> CompilerState {
        CompilerState {
            tokens: Vec::new(),
            locations: Vec::new(),
        }
    }

    /// Parses the given input
    pub fn parse(&mut self, input: &str) -> Result<(), Error> {
        let mut line = 0;
        let mut column = 0;
        let mut working_buffer = String::new();

        let mut start_line = 0;
        let mut start_column = 0;

        for (idx, c) in input.chars().enumerate() {
            let making_string = !working_buffer.is_empty() && working_buffer.starts_with('"');
            let make_token = if idx == input.len() - 1 {
                true
            } else if making_string {
                c == '"' && !working_buffer.ends_with('\\')
            } else {
                c.is_whitespace()
            };

            if !c.is_whitespace() || making_string {
                if working_buffer.is_empty() {
                    start_line = line;
                    start_column = column;
                }

                working_buffer.push(c);
            }

            if make_token {
                let ty = match working_buffer.as_str() {
                    "true" | "false" => TokenType::Bool,
                    "+" => TokenType::Add,
                    _ => {
                        //
                        if working_buffer.starts_with('"') {
                            if !working_buffer.ends_with('"') {
                                return Err(Error {
                                    msg: "String not closed".to_string(),
                                    location: Location {
                                        line: start_line,
                                        column: start_column,
                                        file: None,
                                    },
                                });
                            }

                            // Clean up quotes and return string
                            working_buffer.remove(0);
                            working_buffer.pop();
                            working_buffer = working_buffer.replace("\\\"", "\"");

                            TokenType::String
                        } else {
                            // Check if it's a number or an identifier
                            working_buffer
                                .parse::<f64>()
                                .map_or(TokenType::Identifier, |_| TokenType::Number)
                        }
                    }
                };

                let token = Token {
                    ty_: ty,
                    value: working_buffer.clone(),
                };
                working_buffer.clear();

                if !token.value.is_empty() {
                    if token.ty_ == TokenType::Identifier {
                        // Ensure it doesn't start with a number
                        if token.value.chars().nth(0).unwrap().is_numeric() {
                            return Err(Error {
                                msg: format!(
                                    "Identifier cannot start with a number: '{}'",
                                    token.value
                                ),
                                location: Location {
                                    line: start_line,
                                    column: start_column,
                                    file: None,
                                },
                            });
                        }
                    }

                    self.tokens.push(token);

                    self.locations.push(Location {
                        line: start_line,
                        column: start_column,
                        file: None,
                    });
                }
            }

            if c == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        Ok(())
    }

    /// Returns the stack of the current state up to the given location
    pub fn get_stack_by_location(&self, location: Location) -> Result<Vec<TokenType>, Error> {
        let mut idx = None;

        for (i, l) in self.locations.iter().enumerate() {
            if l.line > location.line {
                break;
            }

            if l.line == location.line && l.column > location.column {
                break;
            }

            idx = Some(i);
        }

        if let None = idx {
            return self.get_stack();
        }

        let idx = idx.unwrap();
        let tokens = self.tokens.iter().take(idx + 1).cloned().collect();
        Self::calculate_stack(&tokens, &self.locations)
    }

    /// Calculates the stack from the given tokens
    fn calculate_stack(
        tokens: &Vec<Token>,
        locations: &Vec<Location>,
    ) -> Result<Vec<TokenType>, Error> {
        let mut stack = Vec::new();

        for (idx, token) in tokens.iter().enumerate() {
            let applications = token.ty_.application();
            for application in applications {
                match application {
                    TokenApplication::Value => {
                        stack.push(token.ty_.clone());
                    }
                    TokenApplication::Consume {
                        values,
                        return_type,
                    } => {
                        if stack.len() < values.len() {
                            return Err(Error {
                                msg: format!(
                                    "Operator '{}' requires a {:?}; got {:?}",
                                    token.value, values, stack
                                ),
                                location: locations[idx].clone(),
                            });
                        }

                        // Now compare values
                        let stack_values = stack[stack.len() - values.len()..].to_vec();
                        for (value_idx, value) in values.iter().enumerate() {
                            if stack_values[value_idx] != *value {
                                return Err(Error {
                                    msg: format!(
                                        "Operator '{}' requires a {:?}; got {:?}",
                                        token.value, values, stack_values
                                    ),
                                    location: locations[idx].clone(),
                                });
                            }
                        }

                        // Calculate new stack
                        // Remove the last N values from the stack
                        // and push the return type
                        for _ in 0..values.len() {
                            stack.pop();
                        }

                        stack.push(return_type);
                    }
                }
            }
        }

        Ok(stack)
    }

    /// Returns the entire stack of the current state
    pub fn get_stack(&self) -> Result<Vec<TokenType>, Error> {
        Self::calculate_stack(&self.tokens, &self.locations)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    ty_: TokenType,
    value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number,
    Bool,
    Add,
    String,
    Identifier,
}
impl TokenType {
    pub fn application(&self) -> Vec<TokenApplication> {
        match self {
            TokenType::Number | TokenType::Bool | TokenType::String => {
                vec![TokenApplication::Value]
            }
            TokenType::Add => vec![TokenApplication::Consume {
                values: vec![TokenType::Number, TokenType::Number],
                return_type: TokenType::Number,
            }],
            TokenType::Identifier => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenApplication {
    /// Returns self on the application
    Value,
    /// Consumes the given number of values and returns the given type
    Consume {
        values: Vec<TokenType>,
        return_type: TokenType,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> CompilerState {
        let mut state = CompilerState::new();
        state.parse(input).unwrap();
        state
    }

    #[test]
    fn parse_identifier_returns_identifier() {
        let input = "identifier";
        let token = Token {
            ty_: TokenType::Identifier,
            value: "identifier".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_identifier_starts_with_number_returns_error() {
        let input = "123identifier";
        let result = CompilerState::new().parse(input);
        let expected = Err(Error {
            msg: "Identifier cannot start with a number: '123identifier'".to_string(),
            location: Location {
                line: 0,
                column: 0,
                file: None,
            },
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_identifier_starts_with_symbol_returns_error() {
        let input = "123identifier";
        let result = CompilerState::new().parse(input);
        let expected = Err(Error {
            msg: "Identifier cannot start with a number: '123identifier'".to_string(),
            location: Location {
                line: 0,
                column: 0,
                file: None,
            },
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_empty_returns_nothing() {
        let input = "";
        let expected = CompilerState::new();
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_whitespace_returns_nothing() {
        let input = "\t\n\n\t";
        let expected = CompilerState::new();
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_1() {
        let input = "1";
        let token = Token {
            ty_: TokenType::Number,
            value: "1".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_math_symbols_are_split() {
        let symbols = ['-', '+', '*', '/', '%', '^', '!', '=', '<', '>'];
    }

    #[test]
    fn parse_string_single() {
        let input = "\"Hello\"";
        let token = Token {
            ty_: TokenType::String,
            value: "Hello".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_string_unclosed_returns_err() {
        let input = "\"Hello";
        let result = CompilerState::new().parse(input);
        let expected = Err(Error {
            msg: "String not closed".to_string(),
            location: Location {
                line: 0,
                column: 0,
                file: None,
            },
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_string_unclosed_in_nested_returns_err() {
        let input = r#""Hello, \"World!"#;
        let result = CompilerState::new().parse(input);
        let expected = Err(Error {
            msg: "String not closed".to_string(),
            location: Location {
                line: 0,
                column: 0,
                file: None,
            },
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_string_with_space() {
        let input = "\"Hello, World!\"";
        let token = Token {
            ty_: TokenType::String,
            value: "Hello, World!".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_string_with_newline() {
        let input = "\"Hello,\n World!\"";
        let token = Token {
            ty_: TokenType::String,
            value: "Hello,\n World!".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_string_nested() {
        let input = r#""Hello, \"World!""#;
        let token = Token {
            ty_: TokenType::String,
            value: "Hello, \"World!".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_2233() {
        let input = "2233";
        let token = Token {
            ty_: TokenType::Number,
            value: "2233".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_true() {
        let input = "true";
        let token = Token {
            ty_: TokenType::Bool,
            value: "true".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_false() {
        let input = "false";
        let token = Token {
            ty_: TokenType::Bool,
            value: "false".to_string(),
        };

        let expected = CompilerState {
            tokens: vec![token],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_add() {
        let input = "+";

        let expected = CompilerState {
            tokens: vec![Token {
                ty_: TokenType::Add,
                value: "+".to_string(),
            }],
            locations: vec![Location {
                line: 0,
                column: 0,
                file: None,
            }],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn get_stack_parse_1_and_2() {
        let input = "1 2";

        let expected = CompilerState {
            tokens: vec![
                Token {
                    ty_: TokenType::Number,
                    value: "1".to_string(),
                },
                Token {
                    ty_: TokenType::Number,
                    value: "2".to_string(),
                },
            ],
            locations: vec![
                Location {
                    line: 0,
                    column: 0,
                    file: None,
                },
                Location {
                    line: 0,
                    column: 2,
                    file: None,
                },
            ],
        };

        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn get_stack_add_returns_err() {
        let input = "+";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got []".to_string(),
            location: Location {
                line: 0,
                column: 0,
                file: None,
            },
        });
        assert_eq!(result.get_stack(), expected_stack);
    }

    #[test]
    fn get_stack_true_add_returns_err() {
        let input = "true +";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Bool]".to_string(),
            location: Location {
                line: 0,
                column: 5,
                file: None,
            },
        });
        assert_eq!(result.get_stack(), expected_stack);
    }

    #[test]
    fn get_stack_true_1_add_returns_err() {
        let input = "true 1 +";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Bool, Number]".to_string(),
            location: Location {
                line: 0,
                column: 7,
                file: None,
            },
        });
        assert_eq!(result.get_stack(), expected_stack);
    }

    #[test]
    fn get_stack_1_1_true_add_returns_err() {
        let input = "1 1 true +";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Number, Bool]".to_string(),
            location: Location {
                line: 0,
                column: 9,
                file: None,
            },
        });
        assert_eq!(result.get_stack(), expected_stack);
    }

    #[test]
    fn get_stack_1_add_returns_err() {
        let input = "1 +";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Number]".to_string(),
            location: Location {
                line: 0,
                column: 2,
                file: None,
            },
        });
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_parse_1_false_add_returns_error() {
        let input = "1 false +";

        let result = parse(input);
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Number, Bool]".to_string(),
            location: Location {
                line: 0,
                column: 8,
                file: None,
            },
        });
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_parse_1_and_2_add() {
        let input = "1 2 +";

        let result = parse(input);

        let expected_stack = Ok(vec![TokenType::Number]);
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_parse_true_false_string_1_2_add() {
        let input = "true false \"1\" 1 2 +";

        let result = parse(input);

        let expected_stack = Ok(vec![
            TokenType::Bool,
            TokenType::Bool,
            TokenType::String,
            TokenType::Number,
        ]);
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_parse_true_false_1_2_add() {
        let input = "true false 1 2 +";

        let result = parse(input);

        let expected_stack = Ok(vec![TokenType::Bool, TokenType::Bool, TokenType::Number]);
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_parse_1_and_2_add_and_3() {
        let input = "1 2 + 3";

        let result = parse(input);

        let expected_stack = Ok(vec![TokenType::Number, TokenType::Number]);
        let result_stack = result.get_stack();
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_first_location_returns_first() {
        let input = "1 2 + 3";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 0,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_after_first_location_before_second_returns_first() {
        let input = "1 2 + 3";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 1,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_second_returns_two() {
        let input = "1 2 + true";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 2,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number, TokenType::Number]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_third_returns_expected() {
        let input = "1 2 + true";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 4,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_past_end_column_returns_all() {
        let input = "1 2 + true";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 10,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number, TokenType::Bool]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_past_end_line_returns_all() {
        let input = "1 2 + true";

        let result = parse(input);
        let location = Location {
            line: 20,
            column: 0,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number, TokenType::Bool]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_before_err_returns_no_err() {
        let input = "1 +";

        let result = parse(input);
        let location = Location {
            line: 0,
            column: 0,
            file: None,
        };
        let expected_stack = Ok(vec![TokenType::Number]);
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }

    #[test]
    fn get_stack_by_location_with_err_returns_err() {
        let input = "1 +";

        let result = parse(input);
        let location = Location {
            line: 110,
            column: 0,
            file: None,
        };
        let expected_stack = Err(Error {
            msg: "Operator '+' requires a [Number, Number]; got [Number]".to_string(),
            location: Location {
                line: 0,
                column: 2,
                file: None,
            },
        });
        let result_stack = result.get_stack_by_location(location);
        assert_eq!(result_stack, expected_stack);
    }
}
