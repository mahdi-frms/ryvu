use lex::{LexerError, lex};
use parse::{parse,ParserError};
use module::Module;
use translate::translate;

#[macro_use]
mod lex;
#[macro_use]
mod translate;
mod parse;

enum CompileResult {
    Ok(Module),
    Error(Vec<LexerError>,Vec<ParserError>)
}

fn compile(source:&str)-> CompileResult {
    let (tokens,lexer_error) = lex(source);
    let (connections,parser_error) = parse(tokens);
    if lexer_error.len() > 0 && parser_error.len() > 0 {
        CompileResult::Error(lexer_error,parser_error)
    }
    else{
        CompileResult::Ok(translate(connections))
    }
}