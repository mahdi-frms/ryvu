use std::collections::HashMap;
use module::{Module, ModuleBuilder};
use crate::lex::{SourcePosition, Token, TokenKind};

#[derive(Default)]
struct Translator {
    state:TranslatorState,
    indexer:Indexer,
    builder:ModuleBuilder,
    once:String,
    is_charge:bool,
    errors:Vec<TranslatorError>
}

#[derive(Debug,PartialEq, Eq)]
enum TranslatorError {
    unexpected_token(SourcePosition),
    unexpected_end
}

#[derive(PartialEq, Eq)]
enum TranslatorState {
    Statement,
    Operator,
    Identifier,
    Terminate,
    Error
}

impl Default for TranslatorState {
    fn default() -> Self {
        TranslatorState::Statement
    }
}

#[derive(Default)]
struct Indexer{
    map:HashMap<String,usize>
}

impl Indexer {
    fn index(&mut self,ident:String)->usize {
        match self.map.get(&ident) {
            Some(index)=> *index,
            None=>{
                self.map.insert(ident, self.map.len());
                self.map.len()-1
            }
        }
    }
}

impl Translator {

    fn unexpected_error(&mut self,token:&Token){
        self.errors.push(TranslatorError::unexpected_token(token.position()))
    }
    fn unexpected_end(&mut self){
        self.errors.push(TranslatorError::unexpected_end);
    }

    fn handle_ident(&mut self,token:&Token){
        match self.state {
            TranslatorState::Statement => {
                self.once = token.text().to_owned();
                self.state = TranslatorState::Operator;
            },
            TranslatorState::Identifier => {
                self.connect(token.text());
                self.once = token.text().to_owned();
                self.state = TranslatorState::Terminate;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = TranslatorState::Error;
            }
        }
    }
    fn connect(&mut self,token_text:&str){
        if self.is_charge {
            self.builder.charge(
                self.indexer.index(self.once.clone()),
                self.indexer.index(token_text.to_owned())
            );
        }
        else {
            self.builder.block(
                self.indexer.index(self.once.clone()),
                self.indexer.index(token_text.to_owned())
            );
        }
    }
    fn handle_semicolon(&mut self,token:&Token){
        match self.state {
            TranslatorState::Terminate | TranslatorState::Error => {
                self.once = String::new();
                self.state = TranslatorState::Statement;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = TranslatorState::Error;
            }
        }
    }
    fn handle_operator(&mut self,token:&Token){
        match self.state {
            TranslatorState::Terminate | TranslatorState::Operator=> {
                if token.kind() == TokenKind::Charge {
                    self.is_charge = true;
                }
                else{
                    self.is_charge = false;
                }
                self.state = TranslatorState::Identifier;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = TranslatorState::Error;
            }
        }
    }
    fn handle_endline(&mut self,token:&Token){
        match self.state {
            TranslatorState::Terminate | TranslatorState::Error => {
                self.once = String::new();
                self.state = TranslatorState::Statement;
            },
            _ => {
                // nothing    
            }
        }
    }
    fn translate(&mut self,tokens:Vec<Token>)->(Module,Vec<TranslatorError>){
        for token in tokens.iter() {
            if  self.state == TranslatorState::Error && 
                token.kind() != TokenKind::Semicolon && 
                token.kind() != TokenKind::EndLine 
            
            {
                continue;
            }
            match token.kind() {
                TokenKind::Identifier=>{
                    self.handle_ident(token);
                },
                TokenKind::Charge | TokenKind::Block=>{
                    self.handle_operator(token);
                },
                TokenKind::Semicolon=>{
                    self.handle_semicolon(token);
                },
                TokenKind::EndLine=>{
                    self.handle_endline(token);
                },
                _=>{

                }
            }
        }
        if self.state == TranslatorState::Operator || self.state == TranslatorState::Identifier {
            self.unexpected_end();
        }
        (self.builder.build(),std::mem::replace(&mut self.errors, vec![]))
    }
}

fn translate(tokens:Vec<Token>)->(Module,Vec<TranslatorError>){
    let mut translator = Translator::default();
    translator.translate(tokens)
}

#[cfg(test)]
mod test {

    use module::ModuleBuilder;

    use crate::{lex::SourcePosition, translate::{translate, Module, Token, TranslatorError}};

    fn module_test_case(tokens:Vec<Token>,module:Module){
        assert_eq!(translate(tokens).0,module);
    }
    fn error_test_case(tokens:Vec<Token>,errors:Vec<TranslatorError>){
        assert_eq!(translate(tokens).1,errors);
    }

    #[test]
    fn no_tokens(){
        module_test_case(vec![], Module::default())
    }

    #[test]
    fn ignores_spaces(){
        module_test_case(vec![
            token!(Space,"   ",0,0),
            token!(EndLine,"\n",0,3),
            token!(Space,"    ",1,0)
        ], Module::default())
    }

    #[test]
    fn single_charge(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Identifier,"b",0,2)
        ], module.build())
    }

    #[test]
    fn single_charge_with_space(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        module_test_case(vec![
            token!(Space,"    ",0,0),
            token!(Identifier,"a",0,4),
            token!(Space,"   ",0,5),
            token!(Charge,">",0,8),
            token!(Space,"  ",0,9),
            token!(Identifier,"b",0,11),
            token!(Space," ",0,12)
        ], module.build())
    }

    #[test]
    fn single_charge_same_node(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 0);
        module_test_case(vec![
            token!(Space,"    ",0,0),
            token!(Identifier,"a",0,4),
            token!(Space,"   ",0,5),
            token!(Charge,">",0,8),
            token!(Space,"  ",0,9),
            token!(Identifier,"a",0,11),
            token!(Space," ",0,12)
        ], module.build())
    }


    #[test]
    fn chained_statements(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module.charge(1, 2);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Charge,">",0,8),
            token!(Identifier,"c",0,9),
        ], module.build())
    }

    #[test]
    fn chained_statements_reoccurring_idents(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module.charge(1, 0);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Charge,">",0,8),
            token!(Identifier,"a",0,9),
        ], module.build())
    }


    #[test]
    fn semincolon_statement_seperation(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module.charge(1, 2);
        module.charge(0, 3);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Charge,">",0,8),
            token!(Identifier,"c",0,9),
            token!(Semicolon,";",0,10),
            token!(Space," ",0,11),
            token!(Identifier,"a",0,12),
            token!(Charge,">",0,13),
            token!(Identifier,"d",0,14),
            token!(Semicolon,";",0,15),
        ], module.build())
    }

    #[test]
    fn passes_on_sequential_identifiers(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module.charge(0, 0);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Semicolon,";",0,8),
            token!(Identifier,"c",0,9),
            token!(Space,"  ",0,10),
            token!(Identifier,"a",0,12),
            token!(Semicolon,";",0,13),
            token!(Identifier,"a",0,14),
            token!(Charge,">",0,15),
            token!(Identifier,"a",0,16),
        ], module.build())
    }

    #[test]
    fn error_on_sequential_identifiers(){
        error_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Semicolon,";",0,8),
            token!(Identifier,"c",0,9),
            token!(Space,"  ",0,10),
            token!(Identifier,"a",0,12),
            token!(Semicolon,";",0,13),
            token!(Identifier,"a",0,14),
            token!(Charge,">",0,15),
            token!(Identifier,"a",0,16),
        ], vec![
            TranslatorError::unexpected_token(SourcePosition::new(0,12))
        ])
    }

    #[test]
    fn ignores_endline_in_statements(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(EndLine,"\n",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"b",0,4)
        ], module.build())
    }

    #[test]
    fn endline_terminates_statement(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        module.charge(0, 2);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(EndLine,"\n",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"b",0,4),
            token!(EndLine,"\n",0,5),
            token!(Identifier,"a",1,0),
            token!(Charge,">",1,1),
            token!(Identifier,"c",1,2),
        ], module.build())
    }

    #[test]
    fn endline_recovers_after_error(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Block,".",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"a",1,0),
            token!(Charge,">",1,1),
            token!(Identifier,"c",1,2),
        ], module.build())
    }

    #[test]
    fn error_on_unexpected_end(){
        error_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Block,".",0,1)
        ], vec![
            TranslatorError::unexpected_end
        ])
    }
}