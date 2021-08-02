mod inverter;
#[cfg(test)]
mod test;
use crate::{
    lex::{SourcePosition, Token, TokenKind},
    translate::{ConVec, Connection, IdentKind, Identifier},
};
use inverter::{DefaultInverter, Inverter};
use std::collections::HashMap;

#[derive(Default)]
struct Parser<I>
where
    I: Inverter,
{
    inverter: I,
    connections: ConVec,
    buffer: ConBuf,
    errors: Vec<ParserError>,
    id_map: IdMap,
}

#[derive(Default)]
struct ConBuf {
    from: Vec<IdPair>,
    to: Vec<IdPair>,
    is_charge: bool,
}

#[derive(Clone)]
struct IdPair(String, bool);

#[derive(PartialEq, Eq, Clone, Copy)]
enum OperatorKind {
    Charge,
    Block,
    Comma,
}

type IdMap = HashMap<String, IdentKind>;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken(SourcePosition),
    UnexpectedEnd,
    IOMin,
    OutPortBlock(String),
    InconstIdKind(String, IdentKind, IdentKind),
}

pub fn parse(tokens: Vec<Token>, io_min: bool) -> (ConVec, Vec<ParserError>) {
    Parser::<DefaultInverter>::default().parse(tokens, io_min)
}

impl<I> Parser<I>
where
    I: Inverter,
{
    fn parse(&mut self, tokens: Vec<Token>, io_min: bool) -> (ConVec, Vec<ParserError>) {
        self.inverter = I::new(tokens);
        while let Some(_) = self.peek_token() {
            if self.expect_source() == None {
                self.inverter.consume_end();
                self.clear_buffer();
            }
        }
        self.finalize(io_min)
    }

    fn expect_source(&mut self) -> Option<()> {
        self.expect_statement()?;
        while let Some(_) = self.peek(&[TokenKind::Semicolon, TokenKind::EndLine]) {
            self.consume_token();
            self.expect_statement()?;
        }
        Some(())
    }

    fn expect_statement(&mut self) -> Option<()> {
        while let Some(_) = self.peek(&[TokenKind::Semicolon, TokenKind::EndLine]) {
            self.consume_token();
        }
        if let Some(_) = self.peek(&[TokenKind::Identifier, TokenKind::Port]) {
            self.expect_batch(OperatorKind::default())?;
            self.expect_operation()?;
            while let Some(_) = self.peek(&[TokenKind::Charge, TokenKind::Block]) {
                self.expect_operation()?;
            }
            self.connect();
            self.clear_buffer();
        }
        Some(())
    }

    fn expect_operation(&mut self) -> Option<()> {
        let opr = self.expect(&[TokenKind::Charge, TokenKind::Block])?;
        self.expect_batch(match opr.kind() {
            TokenKind::Charge => OperatorKind::Charge,
            TokenKind::Block => OperatorKind::Block,
            _ => OperatorKind::default(),
        })
    }

    fn expect_batch(&mut self, operator_kind: OperatorKind) -> Option<()> {
        let mut id = self.expect_id()?;
        self.new_ident(id.0.as_str(), id.1, operator_kind);
        while let Some(_) = self.peek(&[TokenKind::Comma]) {
            self.consume_token();
            id = self.expect_id()?;
            self.new_ident(id.0.as_str(), id.1, OperatorKind::Comma);
        }
        Some(())
    }

    fn expect_id(&mut self) -> Option<IdPair> {
        let t1 = self.expect_token()?;
        match t1.kind() {
            TokenKind::Identifier => Some(IdPair(t1.text().to_owned(), false)),
            TokenKind::Port => {
                let t2 = self.expect(&[TokenKind::Identifier])?;
                Some(IdPair(t2.text().to_owned(), true))
            }
            _ => {
                self.err_unexpected_token(&t1);
                None
            }
        }
    }

    fn consume_token(&mut self) {
        self.inverter.expect();
    }

    fn expect_token(&mut self) -> Option<Token> {
        match self.inverter.expect() {
            None => {
                self.err_unexpected_end();
                None
            }
            Some(token) => Some(token),
        }
    }

    fn peek_token(&mut self) -> Option<Token> {
        self.inverter.peek()
    }

    fn expect(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        let t = self.expect_token()?;
        if kinds.contains(&t.kind()) {
            Some(t)
        } else {
            self.err_unexpected_token(&t);
            None
        }
    }

    fn peek(&mut self, kinds: &[TokenKind]) -> Option<Token> {
        let token = self.peek_token()?;
        if kinds.contains(&token.kind()) {
            Some(token)
        } else {
            None
        }
    }

    fn finalize(&mut self, io_min: bool) -> (ConVec, Vec<ParserError>) {
        if self.errors.len() == 0 {
            if io_min && !self.check_io_min() {
                self.errors.push(ParserError::IOMin);
            }
            self.check_output_block();
        }

        (
            std::mem::take(&mut self.connections),
            std::mem::take(&mut self.errors),
        )
    }

    fn check_output_block(&mut self) {
        for i in 0..self.connections.0.len() {
            let con = self.connections.0[i].clone();
            if con.to.kind == IdentKind::OutPort && !con.is_charge {
                self.err_output_block(con.to.name);
            }
        }
    }

    fn check_io_min(&self) -> bool {
        let mut iflag = false;
        let mut oflag = false;
        for k in self.id_map.values() {
            if *k == IdentKind::InPort {
                iflag = true;
                if oflag {
                    return true;
                }
            } else if *k == IdentKind::OutPort {
                oflag = true;
                if iflag {
                    return true;
                }
            }
        }
        false
    }

    fn new_ident(&mut self, token_text: &str, port: bool, operator_kind: OperatorKind) {
        if operator_kind == OperatorKind::Comma {
            self.buffer.to.push(IdPair(token_text.to_owned(), port));
        } else {
            if self.buffer.from.len() > 0 {
                self.connect();
            }
            self.buffer.from = std::mem::take(&mut self.buffer.to);
            self.buffer.is_charge = operator_kind == OperatorKind::Charge;
            self.buffer.to.push(IdPair(token_text.to_owned(), port));
        }
    }

    fn connect(&mut self) {
        for from in 0..self.buffer.from.len() {
            for to in 0..self.buffer.to.len() {
                self.connect_pair(self.buffer.from[from].clone(), self.buffer.to[to].clone());
            }
        }
    }

    fn clear_buffer(&mut self) {
        self.buffer.from.clear();
        self.buffer.to.clear();
    }

    fn connect_pair(&mut self, from: IdPair, to: IdPair) {
        let from_kind = self.get_ident_kind(from.1, true);
        let to_kind = self.get_ident_kind(to.1, false);
        self.check_ident_kind(&from.0, from_kind);
        self.check_ident_kind(&to.0, to_kind);
        let from = Identifier::new(from.0, from_kind);
        let to = Identifier::new(to.0, to_kind);
        self.connections
            .0
            .push(Connection::new(from, to, self.buffer.is_charge));
    }

    fn get_ident_kind(&self, is_port: bool, is_from: bool) -> IdentKind {
        if is_port {
            if is_from {
                IdentKind::InPort
            } else {
                IdentKind::OutPort
            }
        } else {
            IdentKind::Node
        }
    }

    fn check_ident_kind(&mut self, name: &String, kind: IdentKind) {
        match self.id_map.get(name).copied() {
            Some(act_kind) => {
                if kind != act_kind {
                    self.err_inconst_ident_kind(name.clone(), kind, act_kind);
                }
            }
            None => {
                self.id_map.insert(name.clone(), kind);
            }
        }
    }

    fn err_unexpected_token(&mut self, token: &Token) {
        self.errors
            .push(ParserError::UnexpectedToken(token.position()))
    }

    fn err_output_block(&mut self, ident: String) {
        self.errors.push(ParserError::OutPortBlock(ident))
    }

    fn err_unexpected_end(&mut self) {
        self.errors.push(ParserError::UnexpectedEnd);
    }

    fn err_inconst_ident_kind(&mut self, name: String, kind: IdentKind, act_kind: IdentKind) {
        self.errors
            .push(ParserError::InconstIdKind(name, kind, act_kind));
    }
}

impl Default for OperatorKind {
    fn default() -> Self {
        OperatorKind::Charge
    }
}
