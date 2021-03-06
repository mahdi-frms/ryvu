#[derive(Default)]
struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<LexerError>,
    buffer: String,
    line: usize,
    char_index: usize,
    buffer_state: BufferState,
}

#[derive(PartialEq, Eq)]
enum BufferState {
    Space,
    Ident,
    InvIdent,
    Empty,
    Comment,
}

#[derive(PartialEq, Eq, Debug)]
pub struct LexerError {
    error_kind: LexerErrorKind,
    position: SourcePosition,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    text: String,
    position: SourcePosition,
}

#[derive(PartialEq, Eq, Debug)]
pub enum LexerErrorKind {
    UnknownChar(char),
    InvalidIdentifier(String),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenKind {
    Port,
    Identifier,
    Space,
    EndLine,
    Charge,
    Block,
    Semicolon,
    Comma,
    Comment,
    Mod,
    Lcrb,
    Rcrb,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct SourcePosition {
    line: usize,
    ch: usize,
}

macro_rules! token {
    ($k:ident,$t:expr,$l:expr,$c:expr) => {
        Token::new(
            crate::lex::TokenKind::$k,
            $t.to_string(),
            crate::lex::SourcePosition::new($l, $c),
        )
    };
    ($k:ident,$t:expr) => {
        Token::new(
            crate::lex::TokenKind::$k,
            $t.to_string(),
            crate::lex::SourcePosition::new(0, 0),
        )
    };
}

pub fn lex(source: &str) -> (Vec<Token>, Vec<LexerError>) {
    let mut lexer = Lexer::default();
    lexer.lex(source)
}

impl Lexer {
    fn lex(&mut self, source: &str) -> (Vec<Token>, Vec<LexerError>) {
        for ch in source.chars() {
            if self.buffer_state == BufferState::Comment {
                if ch == '\n' {
                    self.handle_endl();
                } else {
                    self.buffer.push(ch)
                }
            } else if ch == ' ' {
                self.handle_space();
            } else if [';', '.', '>', '$', ',', '}', '{'].contains(&ch) {
                self.handle_signs(ch);
            } else if ch == '#' {
                self.handle_comment(ch);
            } else if Self::is_alphanumeric(ch) {
                self.handle_ident(ch)
            } else if ch == '\n' {
                self.handle_endl();
            } else {
                self.handle_unknown(ch);
            }
        }
        self.finalize()
    }
    fn handle_unknown(&mut self, ch: char) {
        self.push_space();
        self.push_ident();
        self.errors.push(LexerError {
            error_kind: LexerErrorKind::UnknownChar(ch),
            position: self.current_pos(),
        });
        self.char_index += 1;
    }
    fn handle_signs(&mut self, ch: char) {
        self.push_space();
        self.push_ident();
        let kind = match ch {
            ';' => TokenKind::Semicolon,
            '$' => TokenKind::Port,
            '>' => TokenKind::Charge,
            ',' => TokenKind::Comma,
            '{' => TokenKind::Lcrb,
            '}' => TokenKind::Rcrb,
            _ => TokenKind::Block,
        };
        self.tokens
            .push(Token::new(kind, ch.to_string(), self.current_pos()));
        self.char_index += 1;
    }
    fn handle_space(&mut self) {
        self.push_ident();
        self.buffer_state = BufferState::Space;
        self.buffer.push(' ');
    }
    fn handle_ident(&mut self, ch: char) {
        self.push_space();
        if self.buffer_state == BufferState::Empty {
            if Self::is_numeric(ch) {
                self.buffer_state = BufferState::InvIdent;
            } else {
                self.buffer_state = BufferState::Ident;
            }
        }
        self.buffer.push(ch);
    }
    fn handle_comment(&mut self, ch: char) {
        self.push_space();
        self.push_ident();
        self.buffer_state = BufferState::Comment;
        self.buffer.push(ch);
    }
    fn handle_endl(&mut self) {
        self.push_space();
        self.push_ident();
        self.push_comment();
        self.tokens
            .push(token!(EndLine, "\n", self.line, self.char_index));
        self.line += 1;
        self.char_index = 0;
    }
    fn finalize(&mut self) -> (Vec<Token>, Vec<LexerError>) {
        self.push_space();
        self.push_ident();
        self.push_comment();
        (
            std::mem::replace(&mut self.tokens, vec![]),
            std::mem::replace(&mut self.errors, vec![]),
        )
    }
    fn is_alphanumeric(ch: char) -> bool {
        Self::is_alphabetic(ch) || Self::is_numeric(ch)
    }
    fn current_pos(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            ch: self.char_index,
        }
    }
    fn push_space(&mut self) {
        if self.buffer_state == BufferState::Space {
            self.push_buffer(TokenKind::Space);
        }
    }
    fn push_comment(&mut self) {
        if self.buffer_state == BufferState::Comment {
            self.push_buffer(TokenKind::Comment);
        }
    }
    fn push_ident(&mut self) {
        if self.buffer_state == BufferState::Ident {
            self.push_buffer(TokenKind::Identifier);
        }
        if self.buffer_state == BufferState::InvIdent {
            self.push_inv_ident();
        }
    }
    fn push_inv_ident(&mut self) {
        self.errors.push(LexerError {
            error_kind: LexerErrorKind::InvalidIdentifier(self.buffer.clone()),
            position: self.current_pos(),
        });
        self.clear_buffer();
    }
    fn is_alphabetic(ch: char) -> bool {
        (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
    }
    fn is_numeric(ch: char) -> bool {
        ch >= '0' && ch <= '9'
    }
    fn push_buffer(&mut self, mut kind: TokenKind) {
        if kind == TokenKind::Identifier && &self.buffer == "mod" {
            kind = TokenKind::Mod;
        }
        self.tokens
            .push(Token::new(kind, self.buffer.clone(), self.current_pos()));
        self.clear_buffer();
    }
    fn clear_buffer(&mut self) {
        self.char_index += self.buffer.len();
        self.buffer.clear();
        self.buffer_state = BufferState::Empty;
    }
}

impl Default for BufferState {
    fn default() -> Self {
        BufferState::Empty
    }
}

impl Token {
    pub fn new(kind: TokenKind, text: String, position: SourcePosition) -> Token {
        Token {
            kind,
            text,
            position,
        }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    /// Get a reference to the token's position.
    pub fn position(&self) -> SourcePosition {
        self.position
    }
}

impl SourcePosition {
    pub fn new(line: usize, ch: usize) -> SourcePosition {
        SourcePosition { line, ch }
    }
}

#[cfg(test)]
mod test {

    use crate::lex::{lex, LexerError, LexerErrorKind, SourcePosition, Token};

    #[test]
    fn empty_source() {
        let source = "";
        let (tokens, errors) = lex(source);
        assert_eq!(tokens, vec![]);
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn space_only() {
        let source = "    ";
        let (tokens, errors) = lex(source);
        assert_eq!(tokens, vec![token!(Space, source, 0, 0)]);
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn spaces_and_endlines() {
        let source = "    \n   \n\n     ";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Space, "    ", 0, 0),
                token!(EndLine, "\n", 0, 4),
                token!(Space, "   ", 1, 0),
                token!(EndLine, "\n", 1, 3),
                token!(EndLine, "\n", 2, 0),
                token!(Space, "     ", 3, 0)
            ]
        );
        assert_eq!(errors, vec![]);
    }
    #[test]
    fn supports_charge() {
        let source = "  > \n   \n>";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Space, "  ", 0, 0),
                token!(Charge, ">", 0, 2),
                token!(Space, " ", 0, 3),
                token!(EndLine, "\n", 0, 4),
                token!(Space, "   ", 1, 0),
                token!(EndLine, "\n", 1, 3),
                token!(Charge, ">", 2, 0)
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_block() {
        let source = "  . \n   \n.";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Space, "  ", 0, 0),
                token!(Block, ".", 0, 2),
                token!(Space, " ", 0, 3),
                token!(EndLine, "\n", 0, 4),
                token!(Space, "   ", 1, 0),
                token!(EndLine, "\n", 1, 3),
                token!(Block, ".", 2, 0)
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_port() {
        let source = "  $ \n   \n$";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Space, "  ", 0, 0),
                token!(Port, "$", 0, 2),
                token!(Space, " ", 0, 3),
                token!(EndLine, "\n", 0, 4),
                token!(Space, "   ", 1, 0),
                token!(EndLine, "\n", 1, 3),
                token!(Port, "$", 2, 0)
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_comma() {
        let source = "  , \n   \n,";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Space, "  ", 0, 0),
                token!(Comma, ",", 0, 2),
                token!(Space, " ", 0, 3),
                token!(EndLine, "\n", 0, 4),
                token!(Space, "   ", 1, 0),
                token!(EndLine, "\n", 1, 3),
                token!(Comma, ",", 2, 0)
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_identifier() {
        let source = "$input > $output;\nmid";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Port, "$", 0, 0),
                token!(Identifier, "input", 0, 1),
                token!(Space, " ", 0, 6),
                token!(Charge, ">", 0, 7),
                token!(Space, " ", 0, 8),
                token!(Port, "$", 0, 9),
                token!(Identifier, "output", 0, 10),
                token!(Semicolon, ";", 0, 16),
                token!(EndLine, "\n", 0, 17),
                token!(Identifier, "mid", 1, 0),
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_identifier_uppercase() {
        let source = "$InPuT > $Output;\nMID";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Port, "$", 0, 0),
                token!(Identifier, "InPuT", 0, 1),
                token!(Space, " ", 0, 6),
                token!(Charge, ">", 0, 7),
                token!(Space, " ", 0, 8),
                token!(Port, "$", 0, 9),
                token!(Identifier, "Output", 0, 10),
                token!(Semicolon, ";", 0, 16),
                token!(EndLine, "\n", 0, 17),
                token!(Identifier, "MID", 1, 0),
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn error_on_unknown_character() {
        let source = "$InPuT @ $Output;\nMID";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Port, "$", 0, 0),
                token!(Identifier, "InPuT", 0, 1),
                token!(Space, " ", 0, 6),
                token!(Space, " ", 0, 8),
                token!(Port, "$", 0, 9),
                token!(Identifier, "Output", 0, 10),
                token!(Semicolon, ";", 0, 16),
                token!(EndLine, "\n", 0, 17),
                token!(Identifier, "MID", 1, 0),
            ]
        );
        assert_eq!(
            errors,
            vec![LexerError {
                error_kind: LexerErrorKind::UnknownChar('@'),
                position: SourcePosition { line: 0, ch: 7 }
            }]
        );
    }

    #[test]
    fn error_on_invalid_ident() {
        let source = "$1nPuT > $Ou4put;\nMID";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Port, "$", 0, 0),
                token!(Space, " ", 0, 6),
                token!(Charge, ">", 0, 7),
                token!(Space, " ", 0, 8),
                token!(Port, "$", 0, 9),
                token!(Identifier, "Ou4put", 0, 10),
                token!(Semicolon, ";", 0, 16),
                token!(EndLine, "\n", 0, 17),
                token!(Identifier, "MID", 1, 0),
            ]
        );
        assert_eq!(
            errors,
            vec![LexerError {
                error_kind: LexerErrorKind::InvalidIdentifier("1nPuT".to_string()),
                position: SourcePosition { line: 0, ch: 1 }
            }]
        );
    }

    #[test]
    fn supports_comment() {
        let source = "a > b # this is a comment";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Identifier, "a", 0, 0),
                token!(Space, " ", 0, 1),
                token!(Charge, ">", 0, 2),
                token!(Space, " ", 0, 3),
                token!(Identifier, "b", 0, 4),
                token!(Space, " ", 0, 5),
                token!(Comment, "# this is a comment", 0, 6)
            ]
        );
        assert_eq!(errors, vec![]);
    }
    #[test]
    fn supports_comment_only() {
        let source = "# this is a comment";
        let (tokens, errors) = lex(source);
        assert_eq!(tokens, vec![token!(Comment, "# this is a comment")]);
        assert_eq!(errors, vec![]);
    }
    #[test]
    fn supports_comment_before_statement() {
        let source = "# this is a comment\na > b ";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Comment, "# this is a comment", 0, 0),
                token!(EndLine, "\n", 0, 19),
                token!(Identifier, "a", 1, 0),
                token!(Space, " ", 1, 1),
                token!(Charge, ">", 1, 2),
                token!(Space, " ", 1, 3),
                token!(Identifier, "b", 1, 4),
                token!(Space, " ", 1, 5),
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_mod_keyword() {
        let source = "mod A;";
        let (tokens, errors) = lex(source);
        assert_eq!(
            tokens,
            vec![
                token!(Mod, "mod", 0, 0),
                token!(Space, " ", 0, 3),
                token!(Identifier, "A", 0, 4),
                token!(Semicolon, ";", 0, 5),
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_lcrb() {
        let source = "{";
        let (tokens, errors) = lex(source);
        assert_eq!(tokens, vec![token!(Lcrb, "{", 0, 0)]);
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn supports_rcrb() {
        let source = "}";
        let (tokens, errors) = lex(source);
        assert_eq!(tokens, vec![token!(Rcrb, "}", 0, 0)]);
        assert_eq!(errors, vec![]);
    }
}
