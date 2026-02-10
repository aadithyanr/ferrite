use crate::db::query::Identifier;

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
        table: Identifier
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
        // SELECT keyword
        match self.next_token() {
            Token::Keyword(kw) if kw.to_lowercase() == "select" => {},
            other => return Err(format!("Expected SELECT, got {:?}", other))
        }

        // parse column names (projection)
        let mut projection: Vec<Identifier> = Vec::new();
        loop {
            match self.next_token() {
                Token::Identifier(name) => {
                    projection.push(Identifier(name));
                }
                Token::Punctuation(',') => {
                    continue;  // continue to next column
                }
                Token::Keyword(kw) if kw.to_lowercase() == "from" => {
                    break;  // move to FROM part
                }
                other => return Err(format!("Unexpected token in projection: {:?}", other))
            }
        }

        let table = match self.next_token() {
            Token::Identifier(name) => Identifier(name),
            other => return Err(format!("Expected table name, got {:?}", other))
        };

        Ok(ASTNode::SelectStatement {
            projection,
            table: table.0,
        })
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
}