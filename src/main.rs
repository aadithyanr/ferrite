mod db;

use db::storage_engine::FileSystem;
use db::schema::Row;
use db::parser::{Parser, Token};
use db::query::QueryPlan;
use db::executor::ExecutionEngine;

fn main() -> Result<(), String> {
    let mut filesystem = FileSystem::new("database.db")?;
    
    filesystem.create_table("users", vec!["id".to_string(), "name".to_string(), "email".to_string()])?;

    // create index on id column
    filesystem.create_index("users", "id")?;
    println!("created btree index on users.id");

    let mut row1 = Row::new();
    row1.set("id".to_string(), "1".to_string());
    row1.set("name".to_string(), "aadithyan nair".to_string());
    row1.set("email".to_string(), "aadithyan@olostep.com".to_string());
    filesystem.insert_row("users", row1)?;

    let mut row2 = Row::new();
    row2.set("id".to_string(), "2".to_string());
    row2.set("name".to_string(), "test user".to_string());
    row2.set("email".to_string(), "test@somethingidk.com".to_string());
    filesystem.insert_row("users", row2)?;

    let mut row3 = Row::new();
    row3.set("id".to_string(), "3".to_string());
    row3.set("name".to_string(), "jane doe".to_string());
    row3.set("email".to_string(), "jane@example.com".to_string());
    filesystem.insert_row("users", row3)?;

    println!("db created and rows inserted (wal active)");

    // query without filter
    let input1 = "SELECT * FROM users";
    println!("\nquery: {}", input1);
    let tokens1 = tokenize(input1);
    let mut parser1 = Parser::new(tokens1);
    let ast1 = parser1.parse()?;

    let query_plan1 = match ast1 {
        db::parser::ASTNode::SelectStatement { projection, table, filter } => {
            QueryPlan {
                table,
                projection,
                filter,
            }
        }
    };

    let execution_engine = ExecutionEngine::new(filesystem.storage_engine.clone());
    println!("{}", execution_engine.explain(&query_plan1));
    let result1 = execution_engine.execute(query_plan1)?;
    println!("results: {:?}\n", result1);

    // query with WHERE clause using index
    let input2 = "SELECT name email FROM users WHERE id = 2";
    println!("query: {}", input2);
    let tokens2 = tokenize(input2);
    let mut parser2 = Parser::new(tokens2);
    let ast2 = parser2.parse()?;

    let query_plan2 = match ast2 {
        db::parser::ASTNode::SelectStatement { projection, table, filter } => {
            QueryPlan {
                table,
                projection,
                filter,
            }
        }
    };

    println!("{}", execution_engine.explain(&query_plan2));
    let result2 = execution_engine.execute(query_plan2)?;
    println!("results: {:?}", result2);

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
                if word == "=" {
                    tokens.push(Token::Operator('='));
                } else if word.contains('=') {
                    let parts: Vec<&str> = word.splitn(2, '=').collect();
                    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                        tokens.push(Token::Identifier(parts[0].to_string()));
                        tokens.push(Token::Operator('='));
                        tokens.push(Token::Identifier(parts[1].to_string()));
                    } else {
                        tokens.push(Token::Identifier(word.to_string()));
                    }
                } else {
                    tokens.push(Token::Identifier(word.to_string()));
                }
            }
        }
    }
    
    tokens.push(Token::EOF);
    tokens
}