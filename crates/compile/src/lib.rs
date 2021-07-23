use lex::lex;
pub use lex::LexerError;
use parse::parse;
pub use parse::ParserError;
use module::Module;
use translate::translate;

#[macro_use]
mod lex;
#[macro_use]
mod translate;
mod parse;

type CompileResult = Result<Module,(Vec<LexerError>,Vec<ParserError>)>;

pub fn compile(source:&str)-> CompileResult {
    let (tokens,lexer_error) = lex(source);
    let (connections,parser_error) = parse(tokens);
    if lexer_error.len() > 0 && parser_error.len() > 0 {
        CompileResult::Err((lexer_error,parser_error))
    }
    else{
        CompileResult::Ok(translate(connections,false).module)
    }
}