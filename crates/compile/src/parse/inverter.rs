use crate::lex::{Token, TokenKind};

pub trait Inverter {
    fn new(tokens: Vec<Token>) -> Self;
    fn consume_end(&mut self);
    fn expect(&mut self) -> Option<Token>;
    fn peek(&mut self) -> Option<Token>;
}

#[derive(Default)]
pub struct DefaultInverter {
    tokens: Vec<Token>,
    index: usize,
    state: InverterState,
    stack: Vec<Token>,
}

#[derive(PartialEq, Eq)]
enum InverterState {
    Normal,
    WasPort,
    WasIdent,
    WasEndl(Token),
}

pub fn consume_end(tokens: &mut Vec<Token>, index: &mut usize) {
    loop {
        if let Some(token) = tokens.get(*index) {
            let t = token.kind();
            if t == TokenKind::Semicolon || t == TokenKind::EndLine {
                break;
            } else {
                *index += 1;
            }
        } else {
            break;
        }
    }
}

impl DefaultInverter {
    fn get(&mut self) {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        match token.kind() {
            TokenKind::Charge | TokenKind::Block | TokenKind::Comma | TokenKind::Semicolon => {
                self.state = InverterState::Normal;
                self.stack.push(token);
            }
            TokenKind::Space => {
                if self.state == InverterState::WasPort {
                    self.stack.push(token);
                    self.state = InverterState::Normal;
                }
            }
            TokenKind::Port => {
                self.state = InverterState::WasPort;
                self.stack.push(token);
            }
            TokenKind::Identifier => {
                self.stack.push(token);
                match &self.state {
                    InverterState::WasEndl(endl) => self.stack.push(endl.clone()),
                    _ => {}
                }
                self.state = InverterState::WasIdent;
            }
            TokenKind::EndLine => {
                if self.state == InverterState::WasIdent {
                    self.state = InverterState::WasEndl(token);
                }
            }
            _ => {}
        }
    }
}

impl Inverter for DefaultInverter {
    fn new(tokens: Vec<Token>) -> DefaultInverter {
        DefaultInverter {
            tokens,
            state: InverterState::Normal,
            index: 0,
            stack: vec![],
        }
    }
    fn consume_end(&mut self) {
        while let Some(token) = self.stack.pop() {
            if token.kind() == TokenKind::Semicolon || token.kind() == TokenKind::EndLine {
                return;
            }
        }
        consume_end(&mut self.tokens, &mut self.index);
        self.state = InverterState::Normal;
    }
    fn expect(&mut self) -> Option<Token> {
        let t = self.peek();
        self.stack.pop();
        t
    }
    fn peek(&mut self) -> Option<Token> {
        while self.index < self.tokens.len() && self.stack.is_empty() {
            self.get();
        }
        return self.stack.last().cloned();
    }
}

impl Default for InverterState {
    fn default() -> Self {
        InverterState::Normal
    }
}

#[cfg(test)]
mod test_inverter {
    use crate::{
        lex::Token,
        parse::inverter::{DefaultInverter, Inverter},
    };

    fn invertor_test_case(tokens: Vec<Token>, inverted: Vec<Token>) {
        let mut inv = DefaultInverter::new(tokens);
        let mut gen = vec![];
        while let Some(token) = inv.expect() {
            gen.push(token);
        }
        assert_eq!(gen, inverted);
    }

    #[test]
    fn no_token() {
        invertor_test_case(vec![], vec![]);
    }

    #[test]
    fn simple_tokens() {
        invertor_test_case(
            vec![
                token!(Charge, ">"),
                token!(Block, "."),
                token!(Comma, ","),
                token!(Semicolon, ";"),
            ],
            vec![
                token!(Charge, ">"),
                token!(Block, "."),
                token!(Comma, ","),
                token!(Semicolon, ";"),
            ],
        );
    }

    #[test]
    fn space() {
        invertor_test_case(
            vec![
                token!(Space, "   "),
                token!(Charge, ">"),
                token!(Block, "."),
                token!(Comma, ","),
                token!(Space, "     "),
                token!(Semicolon, ";"),
                token!(Space, "     "),
            ],
            vec![
                token!(Charge, ">"),
                token!(Block, "."),
                token!(Comma, ","),
                token!(Semicolon, ";"),
            ],
        );
    }

    #[test]
    fn identifiers() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn identifier_followed_by_endl() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(EndLine, "\n"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn endl_followed_by_identifier() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(EndLine, "\n"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn endl_surrounded_by_identifier() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(EndLine, "\n"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(EndLine, "\n"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn endl_surrounded_by_identifiers_and_spaces() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Space, " "),
                token!(Space, "    "),
                token!(EndLine, "\n"),
                token!(Space, "   "),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(EndLine, "\n"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn port() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Space, " "),
                token!(Port, "$"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Port, "$"),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }

    #[test]
    fn space_after_port() {
        invertor_test_case(
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Space, " "),
                token!(Port, "$"),
                token!(Space, "    "),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
            vec![
                token!(Block, "."),
                token!(Identifier, "s"),
                token!(Port, "$"),
                token!(Space, "    "),
                token!(Identifier, "s"),
                token!(Comma, ","),
            ],
        );
    }
}
