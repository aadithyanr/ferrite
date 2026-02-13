mod db;

use db::storage_engine::FileSystem;
use db::schema::Row;
use db::parser::{Parser, Token};
use db::query::{QueryPlan, Identifier};
use db::executor::ExecutionEngine;

fn main() -> Result<(), String> {
    let mut filesystem = FileSystem::new("database.db")?;
    
    filesystem.create_table("users", vec!["id".to_string(), "name".to_string(), "email".to_string()])?;
    
    let mut row1 = Row::new();
    row1.set("id".to_string(), "1".to_string());
    row1.set("name".to_string(), "aadithyan nair".to_string());
    row1.set("email".to_string(), "aadithyan@olostep.com".to_string());
    filesystem.insert_row("users", row1)?;
    
    let mut row2 = Row::new();
    row2.set("id".to_string(), "2".to_string());
    row2.set("name".to_string(), "test user".to_string());
    row2.set("email".to_string(), "test@example.com".to_string());
    filesystem.insert_row("users", row2)?;
    
    println!("db created and rows inserted");
    
    let input = "SELECT * FROM users";
    let tokens = tokenize(input);
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    println!("parsed: {:?}", ast);
    
    let query_plan = match ast {
        db::parser::ASTNode::SelectStatement { projection, table } => {
            QueryPlan {
                table,
                projection,
            }
        }
    };
    
    let execution_engine = ExecutionEngine::new(filesystem.storage_engine.clone());
    let result = execution_engine.execute(query_plan)?;
    
    println!("result: {:?}", result);
    
    Ok(())
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let words: Vec<&str> = input.split_whitespace().collect();
    
    for word in words {
        match word.to_uppercase().as_str() {
            "SELECT" | "FROM" | "WHERE" => {
                tokens.push(Token::Keyword(word.to_string()));
            }
            "*" => {
                tokens.push(Token::Operator('*'));
            }
            _ => {
                tokens.push(Token::Identifier(word.to_string()));
            }
        }
    }
    
    tokens.push(Token::EOF);
    tokens
}