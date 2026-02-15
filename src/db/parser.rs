use crate::db::query::{Identifier, Filter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Punctuation(char),
    Operator(char),
    Comment(String),
    Whitespace,
    Error(String),
    EOF,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    SelectStatement {
        projection: Vec<Identifier>,
        table: String,
        filter: Option<Filter>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current_index: 0 }
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        self.parse_select_statement()
    }

    fn parse_select_statement(&mut self) -> Result<ASTNode, String> {
        match self.next_token() {
            Token::Keyword(kw) if kw.to_lowercase() == "select" => {},
            other => return Err(format!("expected select, got {:?}", other))
        }

        let mut projection: Vec<Identifier> = Vec::new();
        
        // handle SELECT *
        match self.peek_token() {
            Token::Operator('*') => {
                self.next_token();
                // we'll handle * in executor by getting all columns
                projection.push(Identifier("*".to_string()));
            }
            _ => {
                loop {
                    match self.next_token() {
                        Token::Identifier(name) => {
                            projection.push(Identifier(name));
                        }
                        Token::Punctuation(',') => {
                            continue;
                        }
                        Token::Keyword(kw) if kw.to_lowercase() == "from" => {
                            break;
                        }
                        other => return Err(format!("unexpected token in projection: {:?}", other))
                    }
                }
            }
        }

        // if we didn't break on FROM, consume it now
        if !matches!(self.tokens.get(self.current_index - 1), Some(Token::Keyword(kw)) if kw.to_lowercase() == "from") {
            match self.next_token() {
                Token::Keyword(kw) if kw.to_lowercase() == "from" => {},
                other => return Err(format!("expected from, got {:?}", other))
            }
        }

        let table = match self.next_token() {
            Token::Identifier(name) => name,
            other => return Err(format!("expected table name, got {:?}", other))
        };

        // check for WHERE clause
        let filter = match self.peek_token() {
            Token::Keyword(kw) if kw.to_lowercase() == "where" => {
                self.next_token(); // consume WHERE
                Some(self.parse_where_clause()?)
            }
            Token::EOF => None,
            _ => None,
        };

        Ok(ASTNode::SelectStatement {
            projection,
            table,
            filter,
        })
    }

    fn parse_where_clause(&mut self) -> Result<Filter, String> {
        let column = match self.next_token() {
            Token::Identifier(name) => name,
            other => return Err(format!("expected column name, got {:?}", other))
        };

        match self.next_token() {
            Token::Operator('=') => {},
            other => return Err(format!("expected =, got {:?}", other))
        }

        let value = match self.next_token() {
            Token::Identifier(val) => val,
            Token::Literal(val) => val,
            other => return Err(format!("expected value, got {:?}", other))
        };

        Ok(Filter { column, value })
    }

    fn next_token(&mut self) -> Token {
        if self.current_index < self.tokens.len() {
            let token = self.tokens[self.current_index].clone();
            self.current_index += 1;
            token
        } else {
            Token::EOF
        }
    }

    fn peek_token(&self) -> Token {
        if self.current_index < self.tokens.len() {
            self.tokens[self.current_index].clone()
        } else {
            Token::EOF
        }
    }
}