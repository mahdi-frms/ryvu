use crate::{
    lex::{SourcePosition, Token},
    parse::{
        inverter::{consume_end, Inverter},
        Parser, ParserError,
    },
    translate::{ConVec, Connection, IdentKind},
};

#[derive(Default)]
struct MockInverter {
    tokens: Vec<Token>,
    index: usize,
}

impl Inverter for MockInverter {
    fn new(tokens: Vec<Token>) -> Self {
        MockInverter { tokens, index: 0 }
    }
    fn peek(&mut self) -> Option<Token> {
        self.tokens.get(self.index).cloned()
    }
    fn expect(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.index).cloned()?;
        self.index += 1;
        Some(t)
    }
    fn consume_end(&mut self) {
        consume_end(&mut self.tokens, &mut self.index)
    }
}

fn parse(tokens: Vec<Token>, io_min: bool) -> (Vec<Connection>, Vec<ParserError>) {
    let mut parser = Parser::<MockInverter>::default();
    let pr = parser.parse(tokens, io_min);
    (pr.0, pr.1)
}

fn parser_test_case(tokens: Vec<Token>, connections: Vec<Connection>) {
    let pr = parse(tokens, false);
    assert_eq!(pr.1, vec![]);
    assert_eq!(ConVec(pr.0), ConVec(connections));
}

fn parse_error_test_case(tokens: Vec<Token>, errors: Vec<ParserError>) {
    let generated_errors = parse(tokens, false).1;
    assert_eq!(generated_errors, errors);
}

fn parse_test_case_force_output(tokens: Vec<Token>, connections: Vec<Connection>) {
    let generated_connections = parse(tokens, false).0;
    assert_eq!(ConVec(generated_connections), ConVec(connections));
}

fn parse_error_test_case_io_min(tokens: Vec<Token>, errors: Vec<ParserError>) {
    let generated_errors = parse(tokens, true).1;
    assert_eq!(generated_errors, errors);
}

#[test]
fn no_tokens() {
    parser_test_case(vec![], vec![])
}

#[test]
fn single_charge() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
        ],
        vec![connection!(a > b)],
    )
}

#[test]
fn single_charge_same_node() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "a"),
        ],
        vec![connection!(a > a)],
    )
}

#[test]
fn chained_statements() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
        ],
        vec![connection!(a.b), connection!(b > c)],
    )
}

#[test]
fn chained_statements_reoccurring_idents() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(Charge, ">"),
            token!(Identifier, "a"),
        ],
        vec![connection!(a.b), connection!(b > a)],
    )
}

#[test]
fn semicolon_statement_seperation() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
            token!(Semicolon, ";"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "d"),
            token!(Semicolon, ";"),
        ],
        vec![connection!(a.b), connection!(b > c), connection!(a > d)],
    )
}

#[test]
fn passes_on_sequential_identifiers() {
    parse_test_case_force_output(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(Semicolon, ";"),
            token!(Identifier, "c"),
            token!(Identifier, "a"),
            token!(Semicolon, ";"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "a"),
        ],
        vec![connection!(a.b), connection!(a > a)],
    )
}

#[test]
fn error_on_sequential_identifiers() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(Semicolon, ";"),
            token!(Identifier, "c"),
            token!(Identifier, "a", 0, 1),
            token!(Semicolon, ";"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "a"),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn ignores_endline_in_statements() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
        ],
        vec![connection!(a.b)],
    )
}

#[test]
fn endline_terminates_statement() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Identifier, "b"),
            token!(EndLine, "\n"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
        ],
        vec![connection!(a.b), connection!(a > c)],
    )
}

#[test]
fn endline_recovers_after_error() {
    parse_test_case_force_output(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Block, "."),
            token!(EndLine, "\n"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
        ],
        vec![connection!(a > c)],
    )
}

#[test]
fn error_on_unexpected_end() {
    parse_error_test_case(
        vec![token!(Identifier, "a"), token!(Block, ".")],
        vec![ParserError::UnexpectedEnd],
    )
}

#[test]
fn input_ports() {
    parser_test_case(
        vec![
            token!(Port, "$"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
        ],
        vec![connection!(!a > b)],
    )
}

#[test]
fn error_port_notfollewedby_ident() {
    parse_error_test_case(
        vec![
            token!(Port, "$"),
            token!(Space, " ", 0, 1),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn output_ports() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "b"),
        ],
        vec![connection!(a > !b)],
    )
}

#[test]
fn error_inconsistant_ident_type() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "b"),
            token!(Semicolon, ";"),
            token!(Port, "$"),
            token!(Identifier, "b"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "a"),
            token!(Semicolon, ";"),
            token!(Port, "$"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
        ],
        vec![
            ParserError::InconstIdKind("b".to_owned(), IdentKind::InPort, IdentKind::OutPort),
            ParserError::InconstIdKind("a".to_owned(), IdentKind::OutPort, IdentKind::Node),
            ParserError::InconstIdKind("a".to_owned(), IdentKind::InPort, IdentKind::Node),
        ],
    )
}

#[test]
fn single_connect_node_batching() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Comma, ","),
            token!(Identifier, "b"),
            token!(Comma, ","),
            token!(Identifier, "c"),
            token!(Charge, ">"),
            token!(Identifier, "d"),
        ],
        vec![connection!(a > d), connection!(b > d), connection!(c > d)],
    )
}

#[test]
fn multi_connect_node_batching() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b1"),
            token!(Comma, ","),
            token!(Identifier, "b2"),
            token!(Charge, ">"),
            token!(Identifier, "c1"),
            token!(Comma, ","),
            token!(Identifier, "c2"),
            token!(Block, "."),
            token!(Identifier, "d"),
        ],
        vec![
            connection!(a > b1),
            connection!(a > b2),
            connection!(b1 > c1),
            connection!(b1 > c2),
            connection!(b2 > c1),
            connection!(b2 > c2),
            connection!(c1.d),
            connection!(c2.d),
        ],
    )
}

#[test]
fn port_node_batching() {
    parser_test_case(
        vec![
            token!(Port, "$"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
            token!(Comma, ","),
            token!(Identifier, "c"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "d"),
        ],
        vec![
            connection!(!a > b),
            connection!(!a > c),
            connection!(b > !d),
            connection!(c > !d),
        ],
    )
}

#[test]
fn error_inconsistant_ident_type_node_batching() {
    parse_error_test_case(
        vec![
            token!(Port, "$"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "b"),
            token!(Comma, ","),
            token!(Identifier, "c"),
            token!(Charge, ">"),
            token!(Port, "$"),
            token!(Identifier, "d"),
        ],
        vec![ParserError::InconstIdKind(
            "b".to_owned(),
            IdentKind::InPort,
            IdentKind::OutPort,
        )],
    )
}

#[test]
fn error_io_min_violated() {
    parse_error_test_case_io_min(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
            token!(EndLine, "\n"),
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "c"),
        ],
        vec![ParserError::IOMin],
    )
}

#[test]
fn operater_at_next_line() {
    parser_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Identifier, "b"),
            token!(Comma, ","),
            token!(Identifier, "c"),
            token!(Charge, ">"),
            token!(Identifier, "d"),
        ],
        vec![
            connection!(a > b),
            connection!(a > c),
            connection!(b > d),
            connection!(c > d),
        ],
    )
}
#[test]
fn unexpected_token_after_opr() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Block, ".", 0, 1),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn unexpected_token_after_port_sign() {
    parse_error_test_case(
        vec![
            token!(Port, "$"),
            token!(Charge, ">", 0, 1),
            token!(Block, "."),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn io_min_violation_wihout_basic_errors() {
    parse_error_test_case_io_min(
        vec![
            token!(Identifier, "a"),
            token!(Charge, ">"),
            token!(Block, ".", 0, 1),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn outport_block_violation() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a"),
            token!(Block, "."),
            token!(Port, "$"),
            token!(Identifier, "b"),
        ],
        vec![ParserError::OutPortBlock("b".to_owned())],
    )
}
