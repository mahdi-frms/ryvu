use std::collections::HashMap;

use module::{Module, ModuleBuilder};
use crate::lex::{Token,TokenKind};

#[derive(Default)]
struct Translator {
    indexer:Indexer,
    builder:ModuleBuilder,
    once:String,
    is_charge:bool
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

    fn handle_ident(&mut self,token:&Token){
        if self.once.len() > 0 {
            if self.is_charge {
                self.builder.charge(
                    self.indexer.index(self.once.clone()),
                    self.indexer.index(token.text().to_owned()),
                );
            }
            else {
                self.builder.block(
                    self.indexer.index(self.once.clone()),
                    self.indexer.index(token.text().to_owned())
                );
            }
        }
        self.once = token.text().to_owned();
    }
    fn translate(&mut self,tokens:Vec<Token>)->Module{
        for token in tokens.iter() {
            match *token.kind() {
                TokenKind::Identifier=>{
                    self.handle_ident(token);
                },
                TokenKind::Charge=>{
                    self.is_charge = true;
                },
                TokenKind::Block=>{
                    self.is_charge = false;
                },
                TokenKind::Semicolon=>{
                    self.once = String::new();
                },
                _=>{

                }
            }
        }
        self.builder.build()
    }
}

fn translate(tokens:Vec<Token>)->Module {
    let mut translator = Translator::default();
    translator.translate(tokens)
}

#[cfg(test)]
mod test {

    use module::ModuleBuilder;

    use crate::translate::{translate, Module, Token};

    fn test_case(tokens:Vec<Token>,module:Module){
        assert_eq!(translate(tokens),module);
    }

    #[test]
    fn no_tokens(){
        test_case(vec![], Module::default())
    }

    #[test]
    fn ignores_spaces(){
        test_case(vec![
            token!(Space,"   ",0,0),
            token!(EndLine,"\n",0,3),
            token!(Space,"    ",1,0)
        ], Module::default())
    }

    #[test]
    fn single_charge(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Identifier,"b",0,2)
        ], module.build())
    }

    #[test]
    fn single_charge_with_space(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        test_case(vec![
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
        test_case(vec![
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
        test_case(vec![
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
        test_case(vec![
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
        test_case(vec![
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
}