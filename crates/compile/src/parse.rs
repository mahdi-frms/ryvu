use std::{collections::HashMap};

use crate::{lex::{SourcePosition, Token, TokenKind}, translate::{Connection, IdentKind, Identifier}};

#[derive(Default)]
struct Parser{
    tokens:Vec<Token>,
    token_index:usize,
    connections:Vec<Connection>,
    buffer:ConBuf,
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

#[derive(PartialEq, Eq,Clone, Copy)]
enum OperatorKind {
    Charge,
    Block,
    Comma
}

type IdMap = HashMap<String,IdentKind>;

#[derive(Debug,PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken(SourcePosition),
    UnexpectedEnd,
    IOMin,
    InconstIdKind(String,IdentKind,IdentKind)
}

pub fn parse(tokens:Vec<Token>,io_min:bool)->(Vec<Connection>,Vec<ParserError>) {
    Parser::default().parse(tokens,io_min)
}

impl Parser {

    fn parse(&mut self,tokens:Vec<Token>,io_min:bool) -> (Vec<Connection>,Vec<ParserError>) {
        self.tokens = tokens;
        while let Some(_) = self.peek_token() {
            if self.expect_source() == None {
                self.consume_end();
                self.clear_buffer();
            }
        }
        self.finalize(io_min)
    }

    fn expect_source(&mut self)->Option<()>{
        self.consume_all(&[TokenKind::Space,TokenKind::EndLine,TokenKind::Semicolon]);
        while let Some(token) = self.peek_token() {
            match token.kind() {
                TokenKind::Identifier | TokenKind::Port =>{
                    self.expect_statement()?;
                    self.consume_all(&[TokenKind::Space,TokenKind::EndLine]);
                },
                TokenKind::Semicolon=>{
                    self.consume_token();
                }
                _=>{
                    self.err_unexpected_token(&token);
                    return None;
                }
            }
        }
        Some(())
    }

    fn expect_statement(&mut self)->Option<()>{
        self.expect_batch(OperatorKind::default())?;
        self.consume_all(&[TokenKind::Space,TokenKind::EndLine]);
        self.expect_opbch()?;
        loop {
            let pr = self.peek_token();
            if pr == None {
                break;
            }
            let token = pr.unwrap();
            match token.kind() {
                TokenKind::Charge | TokenKind::Block => self.expect_opbch()?,
                TokenKind::EndLine =>{
                    self.consume_all(&[TokenKind::Space,TokenKind::EndLine]);
                    match self.peek_token() {
                        None => break,
                        Some(token) => match token.kind() {
                            TokenKind::Identifier | TokenKind::Port => break,
                            TokenKind::Charge | TokenKind::Block =>continue,
                            _=>{
                                self.err_unexpected_token(&token);
                                return None;
                            }
                        }
                    }
                },
                TokenKind::Semicolon =>{
                    self.consume_token();
                    break;
                }
                _=>{
                    self.err_unexpected_token(&token);
                    return None;
                }
            }
        }
        self.connect();
        self.clear_buffer();
        Some(())
    }

    fn expect_batch(&mut self,operator_kind:OperatorKind)->Option<()>{
        let mut id = self.expect_id()?;
        self.consume_all(&[TokenKind::Space]);
       
        self.new_ident(id.0.as_str(), id.1, operator_kind);
        while let Some(_) = self.peek(TokenKind::Comma) {
            self.consume_token();
            self.consume_all(&[TokenKind::Space,TokenKind::EndLine]);
            id = self.expect_id()?;
            self.consume_all(&[TokenKind::Space]);
            self.new_ident(id.0.as_str(), id.1, OperatorKind::Comma);
        }
        Some(())
    }

    fn expect_opbch(&mut self)->Option<()>{
        let op = self.expect(&[TokenKind::Charge,TokenKind::Block])?;
        self.consume_all(&[TokenKind::Space,TokenKind::EndLine]);
        self.expect_batch(match op.kind() {
            TokenKind::Charge => OperatorKind::Charge,
            TokenKind::Block => OperatorKind::Block,
            _ => OperatorKind::default(),
        })?;
        Some(())
    }

    fn expect_id(&mut self)->Option<IdPair> {
        let t1 = self.expect_token()?;
        match t1.kind() {
            TokenKind::Identifier=>{ 
                Some(IdPair(t1.text().to_owned(),false))
            },
            TokenKind::Port=>{
                let t2 = self.expect(&[TokenKind::Identifier])?;
                Some(IdPair(t2.text().to_owned(),true))
            },
            _=>None
        }
    }

    fn consume_end(&mut self){
        while let Some(token) = self.peek_token() {
            self.consume_token();
            if token.kind() == TokenKind::EndLine || token.kind() == TokenKind::Semicolon {
                break;
            }
        }
    }

    fn consume_all(&mut self,skips:&[TokenKind]){
        while let Some(token) = self.peek_token() {
            if skips.contains(&token.kind()) {
                self.consume_token();
            }
            else {
                break;
            }
        }
    }

    fn consume_token(&mut self){
        self.token_index += 1;
    }

    fn expect_token(&mut self) -> Option<Token>{
        let t = self.peek_token();
        if t == None {
            self.err_unexpected_end();
        }
        self.token_index += 1;
        t
    }

    fn peek_token(&mut self) -> Option<Token>{
        if self.token_index < self.tokens.len() {
            Some(self.tokens[self.token_index].clone())
        }
        else{
            None
        }
    }

    fn expect(&mut self,kinds:&[TokenKind]) -> Option<Token>{
        let t = self.expect_token()?;
        if kinds.contains(&t.kind()) {
            Some(t)   
        }
        else{
            self.err_unexpected_token(&t);
            None
        }
    }

    fn peek(&mut self,kind:TokenKind) -> Option<Token>{
        let t = self.peek_token()?;
        if t.kind() == kind {
            Some(t)
        }
        else{
            None
        }
    }

    fn finalize(&mut self,io_min:bool) -> (Vec<Connection>,Vec<ParserError>){
        if io_min && !self.check_io_min() {
            self.errors.push(ParserError::IOMin);
        }
        (
            std::mem::replace(&mut self.connections, vec![]),
            std::mem::replace(&mut self.errors, vec![])
        )
    }

    fn check_io_min(&self)->bool{
        let mut iflag = false;
        let mut oflag = false;
        for k in self.id_map.values() {
            if *k == IdentKind::InPort {
                iflag = true;
                if oflag {
                    return true;
                }
            }
            else if *k == IdentKind::OutPort {
                oflag = true;
                if iflag {
                    return true;
                }
            }
        }
        false
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

    fn clear_buffer(&mut self){
        self.buffer.from.clear();
        self.buffer.to.clear();
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
                    self.err_inconst_ident_kind(name.clone(),kind,act_kind);
                }
            },
            None => {
                self.id_map.insert(name.clone(), kind);
            }
        }
    }

    fn err_unexpected_token(&mut self,token:&Token){
        self.errors.push(ParserError::UnexpectedToken(token.position()))
    }

    fn err_unexpected_end(&mut self){
        self.errors.push(ParserError::UnexpectedEnd);
    }

    fn err_inconst_ident_kind(&mut self,name:String,kind:IdentKind,act_kind:IdentKind){
        self.errors.push(ParserError::InconstIdKind(name,kind,act_kind));
    }
}

impl Default for OperatorKind {
    fn default() -> Self {
        OperatorKind::Charge
    }
}

#[cfg(test)]
mod test {
    use crate::{lex::{SourcePosition,Token}, translate::{Connection,IdentKind}, parse::{parse,ParserError}};

    fn parser_test_case(tokens:Vec<Token>,connections:Vec<Connection>){
        let pr = parse(tokens,false);
        assert_eq!(pr.1,vec![]);
        assert_eq!(pr.0,connections);
    }

    fn parse_error_test_case(tokens:Vec<Token>,errors:Vec<ParserError>){
        let generated_errors = parse(tokens,false).1;
        assert_eq!(generated_errors,errors);
    }

    fn parse_test_case_force_output(tokens:Vec<Token>,connections:Vec<Connection>){
        let generated_errors = parse(tokens,false).0;
        assert_eq!(generated_errors,connections);
    }

    fn parse_error_test_case_io_min(tokens:Vec<Token>,errors:Vec<ParserError>){
        let generated_errors = parse(tokens,true).1;
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
    fn semicolon_statement_seperation(){
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
        parse_test_case_force_output(vec![
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

    #[test]
    fn error_io_min_violated(){
        parse_error_test_case_io_min(vec![
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Identifier,"b"),
            token!(EndLine,"\n"),
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Identifier,"c"),
        ], vec![
            ParserError::IOMin
        ])
    }

    #[test]
    fn operater_at_next_line(){
        parser_test_case(vec![
            token!(Identifier,"a"),
            token!(Charge,">"),
            token!(Identifier,"b"),
            token!(Comma,","),
            token!(Identifier,"c"),
            token!(EndLine,"\n"),
            token!(Charge,">"),
            token!(Identifier,"d"),
        ], vec![
            connection!(a > b),
            connection!(a > c),
            connection!(b > d),
            connection!(c > d),
        ])
    }
}