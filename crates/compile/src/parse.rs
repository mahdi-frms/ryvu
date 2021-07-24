use std::{collections::HashMap};

use crate::{lex::{SourcePosition, Token, TokenKind}, translate::{Connection, IdentKind, Identifier}};

#[derive(Default)]
struct Parser{
    state:ParserState,
    connections:Vec<Connection>,
    buffer:ConBuf,
    operator_kind:OperatorKind,
    errors:Vec<ParserError>,
    id_map:IdMap
}

#[derive(Default)]
struct ConBuf {
    from:Vec<IdPair>,
    to:Vec<IdPair>,
    is_charge:bool
}

#[derive(Clone)]
struct IdPair(String,bool);

impl ConBuf {
    fn clear(&mut self) {
        self.from.clear();
        self.to.clear();
    }
}

#[derive(PartialEq, Eq,Clone, Copy)]
enum OperatorKind {
    Charge,
    Block,
    Comma
}

type IdMap = HashMap<String,IdentKind>;

#[derive(PartialEq, Eq)]
enum ParserState {
    Statement(bool),
    Operator,
    Identifier(bool),
    Terminate,
    Error
}

#[derive(Debug,PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken(SourcePosition),
    UnexpectedEnd,
    InconstIdKind(String,IdentKind,IdentKind)
}

pub fn parse(tokens:Vec<Token>)->(Vec<Connection>,Vec<ParserError>) {
    Parser::default().parse(tokens)
}

impl Parser {

    fn parse(&mut self,tokens:Vec<Token>) -> (Vec<Connection>,Vec<ParserError>) {
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
            TokenKind::Charge | TokenKind::Block | TokenKind::Comma=>{
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

    fn finalize(&mut self) -> (Vec<Connection>,Vec<ParserError>){
        self.connect();
        if self.state == ParserState::Operator || 
        self.state == ParserState::Identifier(true) ||
        self.state == ParserState::Identifier(false)
        {
            self.unexpected_end();
        }
        self.state = ParserState::Statement(false);

        (
            std::mem::replace(&mut self.connections, vec![]),
            std::mem::replace(&mut self.errors, vec![])
        )
    }

    fn handle_ident(&mut self,token:&Token){
        match self.state {
            ParserState::Statement(is_port) => {
                self.new_ident(token.text(),is_port,self.operator_kind);
                self.state = ParserState::Operator;
            },
            ParserState::Identifier(is_port) => {
                self.new_ident(token.text(),is_port,self.operator_kind);
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
            ParserState::Identifier(true) | ParserState::Statement(true) => {
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
                self.connect();
                self.buffer.clear();
                self.state = ParserState::Statement(false);
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
                    self.operator_kind = OperatorKind::Charge;
                }
                else if token.kind() == TokenKind::Block{
                    self.operator_kind = OperatorKind::Block;
                }
                else{
                    self.operator_kind = OperatorKind::Comma
                }
                self.state = ParserState::Identifier(false);
            },
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }

    fn handle_port(&mut self,token:&Token){
        match self.state {
            ParserState::Identifier(false) => {
                self.state = ParserState::Identifier(true);
            },
            ParserState::Statement(false) => {
                self.state = ParserState::Statement(true);
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
                self.connect();
                self.buffer.clear();
                self.state = ParserState::Statement(false);
            },
            _ => {
                // nothing
            }
        }
    }

    fn new_ident(&mut self,token_text:&str,port:bool,operator_kind:OperatorKind){
        if operator_kind == OperatorKind::Comma {
            self.buffer.to.push(IdPair(token_text.to_owned(),port));
        }
        else{
            if self.buffer.from.len() > 0 {
                self.connect();
            }
            self.buffer.from = std::mem::take(&mut self.buffer.to);
            self.buffer.is_charge = operator_kind == OperatorKind::Charge;
            self.buffer.to.push(IdPair(token_text.to_owned(),port));
        }
    }

    fn connect(&mut self){
        for from in 0..self.buffer.from.len() {
            for to in 0..self.buffer.to.len() {
                self.connect_pair(self.buffer.from[from].clone(),self.buffer.to[to].clone());
            }    
        }
    }

    fn connect_pair(&mut self,from:IdPair,to:IdPair){
        let from_kind = self.get_ident_kind(from.1, true);
        let to_kind = self.get_ident_kind(to.1, false);
        self.check_ident_kind(&from.0, from_kind);
        self.check_ident_kind(&to.0, to_kind);
        let from = Identifier::new(from.0, from_kind);
        let to = Identifier::new(to.0, to_kind);
        self.connections.push(Connection::new(from, to, self.buffer.is_charge));
    }

    fn get_ident_kind(&self,is_port:bool,is_from:bool) -> IdentKind {
        if is_port {
            if is_from {
                IdentKind::InPort
            }
            else{
                IdentKind::OutPort
            }
        }
        else{
            IdentKind::Node
        }
    }

    fn check_ident_kind(&mut self,name:&String,kind:IdentKind){
        match self.id_map.get(name).copied() {
            Some(act_kind) => {
                if kind != act_kind {
                    self.inconst_ident_kind(name.clone(),kind,act_kind);
                }
            },
            None => {
                self.id_map.insert(name.clone(), kind);
            }
        }
    }

    fn unexpected_error(&mut self,token:&Token){
        self.errors.push(ParserError::UnexpectedToken(token.position()))
    }

    fn unexpected_end(&mut self){
        self.errors.push(ParserError::UnexpectedEnd);
    }

    fn inconst_ident_kind(&mut self,name:String,kind:IdentKind,act_kind:IdentKind){
        self.errors.push(ParserError::InconstIdKind(name,kind,act_kind));
    }
}

impl Default for OperatorKind {
    fn default() -> Self {
        OperatorKind::Charge
    }
}

impl Default for ParserState {
    fn default() -> Self {
        ParserState::Statement(false)
    }
}

#[cfg(test)]
mod test {
    use crate::{lex::{SourcePosition,Token}, translate::{Connection,IdentKind}, parse::{parse,ParserError}};

    fn parser_test_case(tokens:Vec<Token>,connections:Vec<Connection>){
        let generated_connections = parse(tokens).0;
        assert_eq!(generated_connections,connections);
    }

    fn parse_error_test_case(tokens:Vec<Token>,errors:Vec<ParserError>){
        let generated_errors = parse(tokens).1;
        assert_eq!(generated_errors,errors);
    }

    #[test]
    fn no_tokens(){
        parser_test_case(vec![], vec![])
    }

    #[test]
    fn ignores_spaces(){
        parser_test_case(vec![
            token!(Space,"   ",0,0),
            token!(EndLine,"\n",0,3),
            token!(Space,"    ",1,0)
        ], vec![])
    }

    #[test]
    fn single_charge(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Identifier,"b",0,2)
        ], vec![
            connection!(a > b)
        ])
    }

    #[test]
    fn single_charge_with_space(){
        parser_test_case(vec![
            token!(Space,"    ",0,0),
            token!(Identifier,"a",0,4),
            token!(Space,"   ",0,5),
            token!(Charge,">",0,8),
            token!(Space,"  ",0,9),
            token!(Identifier,"b",0,11),
            token!(Space," ",0,12)
        ], vec![
            connection!(a > b)
        ])
    }

    #[test]
    fn single_charge_same_node(){
        parser_test_case(vec![
            token!(Space,"    ",0,0),
            token!(Identifier,"a",0,4),
            token!(Space,"   ",0,5),
            token!(Charge,">",0,8),
            token!(Space,"  ",0,9),
            token!(Identifier,"a",0,11),
            token!(Space," ",0,12)
        ], vec![
            connection!(a > a)
        ])
    }

    #[test]
    fn chained_statements(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Charge,">",0,8),
            token!(Identifier,"c",0,9),
        ], vec![
            connection!(a . b),
            connection!(b > c)
        ])
    }

    #[test]
    fn chained_statements_reoccurring_idents(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Space,"   ",0,1),
            token!(Block,".",0,4),
            token!(Space,"  ",0,5),
            token!(Identifier,"b",0,7),
            token!(Charge,">",0,8),
            token!(Identifier,"a",0,9),
        ], vec![
            connection!(a . b),
            connection!(b > a)
        ])
    }

    #[test]
    fn semincolon_statement_seperation(){
        parser_test_case(vec![
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
        ], vec![
            connection!(a . b),
            connection!(b > c),
            connection!(a > d)
        ])
    }

    #[test]
    fn passes_on_sequential_identifiers(){
        parser_test_case(vec![
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
            connection!(a . b),
            connection!(a > a),
        ])
    }

    #[test]
    fn error_on_sequential_identifiers(){
        parse_error_test_case(vec![
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
            ParserError::UnexpectedToken(SourcePosition::new(0,12))
        ])
    }

    #[test]
    fn ignores_endline_in_statements(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(EndLine,"\n",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"b",0,4)
        ], vec![
            connection!(a . b)
        ])
    }

    #[test]
    fn endline_terminates_statement(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(EndLine,"\n",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"b",0,4),
            token!(EndLine,"\n",0,5),
            token!(Identifier,"a",1,0),
            token!(Charge,">",1,1),
            token!(Identifier,"c",1,2),
        ], vec![
            connection!(a . b),
            connection!(a > c)
        ])
    }

    #[test]
    fn endline_recovers_after_error(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Block,".",0,1),
            token!(Block,".",0,2),
            token!(EndLine,"\n",0,3),
            token!(Identifier,"a",1,0),
            token!(Charge,">",1,1),
            token!(Identifier,"c",1,2),
        ], vec![
            connection!(a > c)
        ])
    }

    #[test]
    fn error_on_unexpected_end(){
        parse_error_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Block,".",0,1)
        ], vec![
            ParserError::UnexpectedEnd
        ])
    }

    #[test]
    fn input_ports(){
        parser_test_case(vec![
            token!(Port,"$",0,0),
            token!(Identifier,"a",0,1),
            token!(Charge,">",0,2),
            token!(Space,"  ",0,3),
            token!(Identifier,"b",0,5)
        ], vec![
            connection!(!a > b)
        ])
    }

    #[test]
    fn error_port_notfollewedby_ident(){
        parse_error_test_case(vec![
            token!(Port,"$",0,0),
            token!(Space," ",0,1),
            token!(Identifier,"a",0,2),
            token!(Charge,">",0,3),
            token!(Space,"  ",0,4),
            token!(Identifier,"b",0,6)
        ], vec![
            ParserError::UnexpectedToken(SourcePosition::new(0,1))
        ])
    }

    #[test]
    fn output_ports(){
        parser_test_case(vec![
            token!(Identifier,"a",0,0),
            token!(Charge,">",0,1),
            token!(Port,"$",0,2),
            token!(Identifier,"b",0,3)
        ], vec![
            connection!(a > !b)
        ])
    }


    #[test]
    fn error_inconsistant_ident_type(){
        parse_error_test_case(vec![
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
            ParserError::InconstIdKind("b".to_owned(),IdentKind::InPort,IdentKind::OutPort),
            ParserError::InconstIdKind("a".to_owned(),IdentKind::OutPort,IdentKind::Node),
            ParserError::InconstIdKind("a".to_owned(),IdentKind::InPort,IdentKind::Node)
        ])
    }

    #[test]
    fn single_connect_node_batching(){
        parser_test_case(vec![
            token!(Identifier,"a"),
            token!(Comma,","),
            token!(Identifier,"b"),
            token!(Comma,","),
            token!(Identifier,"c"),
            token!(Charge,">"),
            token!(Identifier,"d")
        ], vec![
            connection!(a > d),
            connection!(b > d),
            connection!(c > d)
        ])
    }
    
    #[test]
    fn multi_connect_node_batching(){
        parser_test_case(vec![
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Identifier,"b1"),
            token!(Comma,","),
            token!(Identifier,"b2"),
            token!(Charge,">"),
            token!(Identifier,"c1"),
            token!(Comma,","),
            token!(Identifier,"c2"),
            token!(Block,"."),
            token!(Identifier,"d")
        ], vec![
            connection!(a > b1),
            connection!(a > b2),
            connection!(b1 > c1),
            connection!(b1 > c2),
            connection!(b2 > c1),
            connection!(b2 > c2),
            connection!(c1 . d),
            connection!(c2 . d)
        ])
    }

    #[test]
    fn port_node_batching(){
        parser_test_case(vec![
            token!(Port,"$"),
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Identifier,"b"),
            token!(Comma,","),
            token!(Identifier,"c"),
            token!(Charge,">"),
            token!(Port,"$"),
            token!(Identifier,"d")
        ], vec![
            connection!(!a > b),
            connection!(!a > c),
            connection!(b > !d),
            connection!(c > !d),
        ])
    }

    #[test]
    fn error_inconsistant_ident_type_node_batching(){
        parse_error_test_case(vec![
            token!(Port,"$"),
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Port,"$"),
            token!(Identifier,"b"),
            token!(Comma,","),
            token!(Identifier,"c"),
            token!(Charge,">"),
            token!(Port,"$"),
            token!(Identifier,"d")
        ], vec![
            ParserError::InconstIdKind("b".to_owned(),IdentKind::InPort,IdentKind::OutPort)
        ])
    }
}