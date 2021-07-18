use module::{Module, ModuleBuilder};
use crate::lex::{Token,TokenKind};

#[derive(Default)]
struct Translator {
    builder:ModuleBuilder,
    once:String,
    is_charge:bool
}

impl Translator {

    fn handle_ident(&mut self,token:&Token){
        if self.once.len() > 0 {
            let next = if token.text().to_owned() == self.once {
                0
            }
            else {
                1
            };
            if self.is_charge {
                self.builder.charge(0, next);
            }
            else {
                self.builder.block(0, next);
            }
        }
        else{
            self.once = token.text().to_owned();
        }
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
    fn single_block(){
        let mut module = ModuleBuilder::default();
        module.block(0, 1);
        test_case(vec![
            token!(Space,"    ",0,0),
            token!(Identifier,"a",0,4),
            token!(Space,"   ",0,5),
            token!(Block,".",0,8),
            token!(Space,"  ",0,9),
            token!(Identifier,"c",0,11),
            token!(Space," ",0,12)
        ], module.build())
    }
}