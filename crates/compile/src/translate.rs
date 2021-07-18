use module::{Module, ModuleBuilder};
use crate::lex::{Token,TokenKind};

fn translate(tokens:Vec<Token>)->Module {
    let mut builder = ModuleBuilder::default();
    for token in tokens.iter() {
        match *token.kind() {
            TokenKind::Identifier=>{
                builder.charge(0, 1);
                break;
            },
            _=>{
                
            }
        }
    }
    builder.build()
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
}