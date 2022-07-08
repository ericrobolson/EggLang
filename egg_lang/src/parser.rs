use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub ast: Ast,
    pub token: Token,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {}

pub struct Module {}

pub struct Parser {}
impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Module {
        todo!()
    }
}
