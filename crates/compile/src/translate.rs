use std::collections::HashMap;
use module::{Module, ModuleBuilder};
use crate::lex::{SourcePosition, Token, TokenKind};

#[derive(Default)]
struct Parser{
    state:ParserState,
    connections:Vec<Connection>,
    buffer:String,
    is_charge:bool,
    is_port:bool,
    errors:Vec<TranslatorError>
}

type IndexMap = HashMap<String,(usize,IdentKind)>;

#[derive(Debug,PartialEq, Eq)]
enum TranslatorError {
    UnexpectedToken(SourcePosition),
    UnexpectedEnd,
    InconstIdent(String,IdentKind,IdentKind)
}

#[derive(PartialEq, Eq)]
enum ParserState {
    Statement,
    Operator,
    Identifier,
    PortIdent,
    PortStmt,
    Terminate,
    Error
}

struct Connection {
    from: Identifier,
    to: Identifier,
    is_charge:bool
}

struct Identifier {
    name:String,
    kind:IdentKind,
}

#[derive(Debug,PartialEq,Eq,Clone, Copy)]
enum IdentKind {
    Node,
    InPort,
    OutPort
}

#[derive(PartialEq, Eq)]
enum PortIndexResult {
    Error,
    Normal,
    NewPort
}

fn translate(tokens:Vec<Token>)->(Module,Vec<TranslatorError>) {
    let errors = vec![];
    let (cons,errors) = Parser::default().parse(tokens,errors);
    let (module,errors) = Translator::default().build(cons,errors);
    (module,errors)
}

#[derive(Default)]
struct Translator {
    errors:Vec<TranslatorError>,
    indexes:IndexMap
}

impl Translator {

    fn build(&mut self,connections:Vec<Connection>,errors:Vec<TranslatorError>)->(Module,Vec<TranslatorError>) {
        self.errors = errors;
        let mut builder = ModuleBuilder::default();
        for con in connections.iter() {

            let from = self.index(&con.from);
            let to = self.index(&con.to);

            if from.1 != PortIndexResult::Error && to.1 != PortIndexResult::Error {
                builder.connect(from.0, to.0, con.is_charge);
                if from.1 == PortIndexResult::NewPort {
                    builder.input(from.0);
                }
                if to.1 == PortIndexResult::NewPort {
                    builder.output(to.0);
                }
            }
        }
        (
            builder.build(),
            std::mem::replace(&mut self.errors, vec![])
        )
    }

    fn index(&mut self,ident:&Identifier)->(usize,PortIndexResult) {
        match self.indexes.get(&ident.name).copied() {
            Some((index,orig_kind))=> {
                if orig_kind == ident.kind {
                    (index,PortIndexResult::Normal)
                }
                else{  
                    self.inconst_ident(ident,orig_kind);
                    (index,PortIndexResult::Error)
                }
            },
            None=>{
                self.indexes.insert(ident.name.clone(), (self.indexes.len(),ident.kind));
                (self.indexes.len()-1,if ident.kind != IdentKind::Node {
                    PortIndexResult::NewPort
                }
                else{
                    PortIndexResult::Normal
                })
            }
        }
    }

    fn inconst_ident(&mut self,ident:&Identifier,orig_kind:IdentKind){
        self.errors.push(TranslatorError::InconstIdent(ident.name.clone(),ident.kind,orig_kind));
    }
}

impl Parser {

    fn parse(&mut self,tokens:Vec<Token>,errors:Vec<TranslatorError>) -> (Vec<Connection>,Vec<TranslatorError>) {
        for token in tokens.iter() {
            if  self.state == ParserState::Error && 
                token.kind() != TokenKind::Semicolon && 
                token.kind() != TokenKind::EndLine 
            
            {
                continue;
            }

            self.handle_token(token);
        }
        self.finalize()
    }
    fn handle_token(&mut self,token:&Token){
        
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
                self.handle_endline();
            },
            TokenKind::Port=>{
                self.handle_port(token);
            },
            TokenKind::Space=>{
                self.handle_space(token);
            }
        }
    }
    fn finalize(&mut self) -> (Vec<Connection>,Vec<TranslatorError>){
        if self.state == ParserState::Operator || self.state == ParserState::Identifier {
            self.unexpected_end();
        }
        self.state = ParserState::Statement;

        (
            std::mem::replace(&mut self.connections, vec![]),
            std::mem::replace(&mut self.errors, vec![])
        )
    }
    fn handle_ident(&mut self,token:&Token){
        match self.state {
            ParserState::Statement => {
                self.buffer = token.text().to_owned();
                self.state = ParserState::Operator;
                self.is_port = false;
            },
            ParserState::PortStmt => {
                self.buffer = token.text().to_owned();
                self.state = ParserState::Operator;
                self.is_port = true;
            },
            ParserState::Identifier => {
                self.connect(token.text(),false);
                self.buffer = token.text().to_owned();
                self.is_port = false;
                self.state = ParserState::Terminate;
            },
            ParserState::PortIdent => {
                self.connect(token.text(),true);
                self.buffer = token.text().to_owned();
                self.is_port = true;
                self.state = ParserState::Terminate;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }
    fn handle_space(&mut self,token:&Token){
        match self.state  {
            ParserState::PortIdent | ParserState::PortStmt => {
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
            _=>{
                // nothing
            }
        }
    }
    fn handle_semicolon(&mut self,token:&Token){
        match self.state {
            ParserState::Terminate | ParserState::Error => {
                self.buffer = String::new();
                self.state = ParserState::Statement;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }
    fn handle_operator(&mut self,token:&Token){
        match self.state {
            ParserState::Terminate | ParserState::Operator=> {
                if token.kind() == TokenKind::Charge {
                    self.is_charge = true;
                }
                else{
                    self.is_charge = false;
                }
                self.state = ParserState::Identifier;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }
    fn handle_port(&mut self,token:&Token){
        match self.state {
            ParserState::Identifier => {
                self.state = ParserState::PortIdent;
            },
            ParserState::Statement => {
                self.state = ParserState::PortStmt;
            }
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }
    fn handle_endline(&mut self){
        match self.state {
            ParserState::Terminate | ParserState::Error => {
                self.buffer = String::new();
                self.state = ParserState::Statement;
            },
            _ => {
                // nothing
            }
        }
    }
    fn connect(&mut self,token_text:&str,port:bool){
        let from = Identifier::new(self.buffer.clone(),if self.is_port {
            IdentKind::InPort
        }
        else{
            IdentKind::Node
        });
        let to = Identifier::new(token_text.to_owned(),if port {
            IdentKind::OutPort
        }
        else{
            IdentKind::Node
        });
        self.connections.push(Connection::new(from, to, self.is_charge));
    }
    fn unexpected_error(&mut self,token:&Token){
        self.errors.push(TranslatorError::UnexpectedToken(token.position()))
    }
    fn unexpected_end(&mut self){
        self.errors.push(TranslatorError::UnexpectedEnd);
    }
}

impl Default for ParserState {
    fn default() -> Self {
        ParserState::Statement
    }
}
impl Connection {
    fn new(from: Identifier, to: Identifier, is_charge: bool) -> Connection {
        Connection { from, to, is_charge }
    }
}
impl Identifier {
    fn new(name: String, kind: IdentKind) -> Self { Self { name, kind } }
}

#[cfg(test)]
mod test {

    use module::ModuleBuilder;
    use crate::{lex::SourcePosition, translate::{IdentKind, Module, Token, TranslatorError, translate}};

    fn module_test_case(tokens:Vec<Token>,module:Module){
        let compiled_module = translate(tokens).0;
        assert_eq!(compiled_module,module);
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
            TranslatorError::UnexpectedToken(SourcePosition::new(0,12))
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
            TranslatorError::UnexpectedEnd
        ])
    }

    #[test]
    fn input_ports(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        module.input(0);
        module_test_case(vec![
            token!(Port,"$",0,0),
            token!(Identifier,"a",0,1),
            token!(Charge,">",0,2),
            token!(Space,"  ",0,3),
            token!(Identifier,"b",0,5)
        ], module.build())
    }
    #[test]
    fn error_port_notfollewedby_ident(){
        error_test_case(vec![
            token!(Port,"$",0,0),
            token!(Space," ",0,1),
            token!(Identifier,"a",0,2),
            token!(Charge,">",0,3),
            token!(Space,"  ",0,4),
            token!(Identifier,"b",0,6)
        ], vec![
            TranslatorError::UnexpectedToken(SourcePosition::new(0,1))
        ])
    }

    #[test]
    fn output_ports(){
        let mut module = ModuleBuilder::default();
        module.charge(0, 1);
        module.output(1);
        module_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Port,"$",0,2),
            token!(Identifier,"b",0,3)
        ], module.build())
    }

    #[test]
    fn error_inconsistant_ident_type(){
        error_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Port,"$",0,2),
            token!(Identifier,"b",0,3),
            token!(Semicolon,";",0,4),

            token!(Port,"$",0,5),
            token!(Identifier,"b",0,6),
            token!(Charge,">",0,7),
            token!(Port,"$",0,8),
            token!(Identifier,"a",0,9),
            token!(Semicolon,";",0,10),

            token!(Port,"$",0,11),
            token!(Identifier,"a",0,12),
            token!(Charge,">",0,13),
            token!(Identifier,"c",0,14),
        ], vec![
            TranslatorError::InconstIdent("b".to_owned(),IdentKind::InPort,IdentKind::OutPort),
            TranslatorError::InconstIdent("a".to_owned(),IdentKind::OutPort,IdentKind::Node),
            TranslatorError::InconstIdent("a".to_owned(),IdentKind::InPort,IdentKind::Node)
        ])
    }
}