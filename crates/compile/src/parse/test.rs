use crate::{
    lex::{SourcePosition, Token},
    parse::{parse, ParserError},
    translate::{Connection, IdentKind},
};

fn parser_test_case(tokens: Vec<Token>, connections: Vec<Connection>) {
    let pr = parse(tokens, false);
    assert_eq!(pr.1, vec![]);
    assert_eq!(pr.0, connections);
}

fn parse_error_test_case(tokens: Vec<Token>, errors: Vec<ParserError>) {
    let generated_errors = parse(tokens, false).1;
    assert_eq!(generated_errors, errors);
}

fn parse_test_case_force_output(tokens: Vec<Token>, connections: Vec<Connection>) {
    let generated_errors = parse(tokens, false).0;
    assert_eq!(generated_errors, connections);
}

fn parse_error_test_case_io_min(tokens: Vec<Token>, errors: Vec<ParserError>) {
    let generated_errors = parse(tokens, true).1;
    assert_eq!(generated_errors, errors);
}

#[test]
fn empty() {
    parser_test_case(vec![], vec![])
}

#[test]
fn no_tokens() {
    parser_test_case(vec![], vec![])
}

#[test]
fn ignores_spaces() {
    parser_test_case(
        vec![
            token!(Space, "   ", 0, 0),
            token!(EndLine, "\n", 0, 3),
            token!(Space, "    ", 1, 0),
        ],
        vec![],
    )
}

#[test]
fn single_charge() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Identifier, "b", 0, 2),
        ],
        vec![connection!(a > b)],
    )
}

#[test]
fn single_charge_with_space() {
    parser_test_case(
        vec![
            token!(Space, "    ", 0, 0),
            token!(Identifier, "a", 0, 4),
            token!(Space, "   ", 0, 5),
            token!(Charge, ">", 0, 8),
            token!(Space, "  ", 0, 9),
            token!(Identifier, "b", 0, 11),
            token!(Space, " ", 0, 12),
        ],
        vec![connection!(a > b)],
    )
}

#[test]
fn single_charge_same_node() {
    parser_test_case(
        vec![
            token!(Space, "    ", 0, 0),
            token!(Identifier, "a", 0, 4),
            token!(Space, "   ", 0, 5),
            token!(Charge, ">", 0, 8),
            token!(Space, "  ", 0, 9),
            token!(Identifier, "a", 0, 11),
            token!(Space, " ", 0, 12),
        ],
        vec![connection!(a > a)],
    )
}

#[test]
fn chained_statements() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Space, "   ", 0, 1),
            token!(Block, ".", 0, 4),
            token!(Space, "  ", 0, 5),
            token!(Identifier, "b", 0, 7),
            token!(Charge, ">", 0, 8),
            token!(Identifier, "c", 0, 9),
        ],
        vec![connection!(a.b), connection!(b > c)],
    )
}

#[test]
fn chained_statements_reoccurring_idents() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Space, "   ", 0, 1),
            token!(Block, ".", 0, 4),
            token!(Space, "  ", 0, 5),
            token!(Identifier, "b", 0, 7),
            token!(Charge, ">", 0, 8),
            token!(Identifier, "a", 0, 9),
        ],
        vec![connection!(a.b), connection!(b > a)],
    )
}

#[test]
fn semicolon_statement_seperation() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Space, "   ", 0, 1),
            token!(Block, ".", 0, 4),
            token!(Space, "  ", 0, 5),
            token!(Identifier, "b", 0, 7),
            token!(Charge, ">", 0, 8),
            token!(Identifier, "c", 0, 9),
            token!(Semicolon, ";", 0, 10),
            token!(Space, " ", 0, 11),
            token!(Identifier, "a", 0, 12),
            token!(Charge, ">", 0, 13),
            token!(Identifier, "d", 0, 14),
            token!(Semicolon, ";", 0, 15),
        ],
        vec![connection!(a.b), connection!(b > c), connection!(a > d)],
    )
}

#[test]
fn passes_on_sequential_identifiers() {
    parse_test_case_force_output(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Space, "   ", 0, 1),
            token!(Block, ".", 0, 4),
            token!(Space, "  ", 0, 5),
            token!(Identifier, "b", 0, 7),
            token!(Semicolon, ";", 0, 8),
            token!(Identifier, "c", 0, 9),
            token!(Space, "  ", 0, 10),
            token!(Identifier, "a", 0, 12),
            token!(Semicolon, ";", 0, 13),
            token!(Identifier, "a", 0, 14),
            token!(Charge, ">", 0, 15),
            token!(Identifier, "a", 0, 16),
        ],
        vec![connection!(a.b), connection!(a > a)],
    )
}

#[test]
fn error_on_sequential_identifiers() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Space, "   ", 0, 1),
            token!(Block, ".", 0, 4),
            token!(Space, "  ", 0, 5),
            token!(Identifier, "b", 0, 7),
            token!(Semicolon, ";", 0, 8),
            token!(Identifier, "c", 0, 9),
            token!(Space, "  ", 0, 10),
            token!(Identifier, "a", 0, 12),
            token!(Semicolon, ";", 0, 13),
            token!(Identifier, "a", 0, 14),
            token!(Charge, ">", 0, 15),
            token!(Identifier, "a", 0, 16),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 12))],
    )
}

#[test]
fn ignores_endline_in_statements() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(EndLine, "\n", 0, 1),
            token!(Block, ".", 0, 2),
            token!(EndLine, "\n", 0, 3),
            token!(Identifier, "b", 0, 4),
        ],
        vec![connection!(a.b)],
    )
}

#[test]
fn endline_terminates_statement() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(EndLine, "\n", 0, 1),
            token!(Block, ".", 0, 2),
            token!(EndLine, "\n", 0, 3),
            token!(Identifier, "b", 0, 4),
            token!(EndLine, "\n", 0, 5),
            token!(Identifier, "a", 1, 0),
            token!(Charge, ">", 1, 1),
            token!(Identifier, "c", 1, 2),
        ],
        vec![connection!(a.b), connection!(a > c)],
    )
}

#[test]
fn endline_recovers_after_error() {
    parse_test_case_force_output(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Block, ".", 0, 1),
            token!(Block, ".", 0, 2),
            token!(EndLine, "\n", 0, 3),
            token!(Identifier, "a", 1, 0),
            token!(Charge, ">", 1, 1),
            token!(Identifier, "c", 1, 2),
        ],
        vec![connection!(a > c)],
    )
}

#[test]
fn error_on_unexpected_end() {
    parse_error_test_case(
        vec![token!(Identifier, "a", 0, 0), token!(Block, ".", 0, 1)],
        vec![ParserError::UnexpectedEnd],
    )
}

#[test]
fn input_ports() {
    parser_test_case(
        vec![
            token!(Port, "$", 0, 0),
            token!(Identifier, "a", 0, 1),
            token!(Charge, ">", 0, 2),
            token!(Space, "  ", 0, 3),
            token!(Identifier, "b", 0, 5),
        ],
        vec![connection!(!a > b)],
    )
}

#[test]
fn error_port_notfollewedby_ident() {
    parse_error_test_case(
        vec![
            token!(Port, "$", 0, 0),
            token!(Space, " ", 0, 1),
            token!(Identifier, "a", 0, 2),
            token!(Charge, ">", 0, 3),
            token!(Space, "  ", 0, 4),
            token!(Identifier, "b", 0, 6),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn output_ports() {
    parser_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Port, "$", 0, 2),
            token!(Identifier, "b", 0, 3),
        ],
        vec![connection!(a > !b)],
    )
}

#[test]
fn error_inconsistant_ident_type() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Port, "$", 0, 2),
            token!(Identifier, "b", 0, 3),
            token!(Semicolon, ";", 0, 4),
            token!(Port, "$", 0, 5),
            token!(Identifier, "b", 0, 6),
            token!(Charge, ">", 0, 7),
            token!(Port, "$", 0, 8),
            token!(Identifier, "a", 0, 9),
            token!(Semicolon, ";", 0, 10),
            token!(Port, "$", 0, 11),
            token!(Identifier, "a", 0, 12),
            token!(Charge, ">", 0, 13),
            token!(Identifier, "c", 0, 14),
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
            token!(EndLine, "\n"),
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
            token!(Identifier, "a", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Block, ".", 0, 2),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 2))],
    )
}

#[test]
fn unexpected_token_after_port_sign() {
    parse_error_test_case(
        vec![
            token!(Port, "$", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Block, ".", 0, 2),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 1))],
    )
}

#[test]
fn io_min_violation_wihout_basic_errors() {
    parse_error_test_case_io_min(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Charge, ">", 0, 1),
            token!(Block, ".", 0, 2),
        ],
        vec![ParserError::UnexpectedToken(SourcePosition::new(0, 2))],
    )
}

#[test]
fn outport_block_violation() {
    parse_error_test_case(
        vec![
            token!(Identifier, "a", 0, 0),
            token!(Block, ".", 0, 1),
            token!(Port, "$", 0, 2),
            token!(Identifier, "b", 0, 3)
        ],
        vec![ParserError::OutPortBlock("b".to_owned())],
    )
}