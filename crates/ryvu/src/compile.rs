#[derive(Default)]
struct Lexer {
    tokens:Vec<Token>,
    buffer:String,
    line:usize,
    char_index:usize,
    buffer_is_space:bool
}


#[derive(PartialEq, Eq, Debug)]
pub struct Token {
    kind:TokenKind,
    text:String,
    position:TokenPosition
}

#[derive(PartialEq, Eq, Debug)]
pub enum TokenKind {
    Port,
    Identifier,
    Space,
    EndLine,
    Charge,
    Block,
    Semicolon
}

#[derive(PartialEq, Eq, Debug)]
pub struct TokenPosition {
    line:usize,
    ch:usize
}

macro_rules! token {
    ($k:ident,$t:expr,$l:expr,$c:expr) => {
        Token::new(
            crate::compile::TokenKind::$k,$t.to_string(),
            TokenPosition::new($l,$c)
        )
    };
}

pub fn lex(source:&str)->Vec<Token> {
    let mut lexer = Lexer::default();
    lexer.lex(source)
}

impl Lexer {
    fn lex(&mut self,source:&str)->Vec<Token> {
        for ch in source.chars() {
            if ch == ' ' {
                self.handle_space();
            }
            else if [';','.','>','$'].contains(&ch) {
                self.handle_signs(ch);
            }
            else if Self::is_alphanumeric(ch) {
                self.handle_ident(ch)
            }
            else{
                self.handle_endl();
            }
        }
        self.finalize()
    }
    fn handle_signs(&mut self,ch:char){
        self.push_space();
        self.push_ident();
        let kind = match ch {
            ';' => TokenKind::Semicolon,
            '$' => TokenKind::Port,
            '>' => TokenKind::Charge,
            _ => TokenKind::Block
        };
        self.tokens.push(Token::new(kind, ch.to_string(), TokenPosition::new(self.line,self.char_index)));
        self.char_index += 1;
    }
    fn handle_space(&mut self){
        self.push_ident();
        self.buffer_is_space = true;
        self.buffer.push(' ');
    }
    fn handle_ident(&mut self,ch:char){
        self.push_space();
        self.buffer_is_space = false;
        self.buffer.push(ch);
    }
    fn handle_endl(&mut self){
        self.push_space();
        self.push_ident();
        self.tokens.push(token!(EndLine,"\n",self.line,self.char_index));
        self.line += 1;
        self.char_index = 0;
    }
    fn finalize(&mut self)->Vec<Token> {
        self.push_space();
        self.push_ident();
        std::mem::replace(&mut self.tokens, vec![])
    }
    fn is_alphanumeric(ch:char)->bool {
        Self::is_alphabetic(ch) || Self::is_numeric(ch)
    }
    fn push_space(&mut self){
        if !self.buffer.is_empty() && self.buffer_is_space {
            self.push_buffer(TokenKind::Space);
        }
    }
    fn push_ident(&mut self){
        if !self.buffer.is_empty() && !self.buffer_is_space {
            self.push_buffer(TokenKind::Identifier);
        }
    }
    fn is_alphabetic(ch:char)->bool {
        (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
    }
    fn is_numeric(ch:char)->bool {
        ch >= '0' && ch <= '9'
    }
    fn push_buffer(&mut self,kind:TokenKind) {
        self.tokens.push(
            Token::new(kind,self.buffer.clone(),TokenPosition::new(self.line,self.char_index))
        );
        self.char_index += self.buffer.len();
        self.buffer.clear();
    }
}


impl Token {
    fn new(kind:TokenKind,text:String,position:TokenPosition)->Token{
        Token{
            kind,text,position
        }
    }
}

impl TokenPosition {
    fn new(line:usize,ch:usize)->TokenPosition{
        TokenPosition {
            line,ch
        }
    }
}

#[cfg(test)]
mod test {

    use crate::compile::{Token, TokenPosition, lex};

    #[test]
    fn empty_source(){
        let source = "";
        let tokens = lex(source);
        assert_eq!(tokens,vec![]);
    }

    #[test]
    fn space_only(){
        let source = "    ";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,source,0,0)
        ]);
    }

    #[test]
    fn spaces_and_endlines(){
        let source = "    \n   \n\n     ";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,"    ",0,0),
            token!(EndLine,"\n",0,4),
            token!(Space,"   ",1,0),
            token!(EndLine,"\n",1,3),
            token!(EndLine,"\n",2,0),
            token!(Space,"     ",3,0)
        ]);
    }
    #[test]
    fn supports_charge(){
        let source = "  > \n   \n>";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,"  ",0,0),
            token!(Charge,">",0,2),
            token!(Space," ",0,3),
            token!(EndLine,"\n",0,4),
            token!(Space,"   ",1,0),
            token!(EndLine,"\n",1,3),
            token!(Charge,">",2,0)
        ]);
    }

    #[test]
    fn supports_block(){
        let source = "  . \n   \n.";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,"  ",0,0),
            token!(Block,".",0,2),
            token!(Space," ",0,3),
            token!(EndLine,"\n",0,4),
            token!(Space,"   ",1,0),
            token!(EndLine,"\n",1,3),
            token!(Block,".",2,0)
        ]);
    }

    #[test]
    fn supports_port(){
        let source = "  $ \n   \n$";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,"  ",0,0),
            token!(Port,"$",0,2),
            token!(Space," ",0,3),
            token!(EndLine,"\n",0,4),
            token!(Space,"   ",1,0),
            token!(EndLine,"\n",1,3),
            token!(Port,"$",2,0)
        ]);
    }

    #[test]
    fn supports_identifier(){
        let source = "$input > $output;\nmid";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Port,"$",0,0),
            token!(Identifier,"input",0,1),
            token!(Space," ",0,6),
            token!(Charge,">",0,7),
            token!(Space," ",0,8),
            token!(Port,"$",0,9),
            token!(Identifier,"output",0,10),
            token!(Semicolon,";",0,16),
            token!(EndLine,"\n",0,17),
            token!(Identifier,"mid",1,0),
        ]);
    }

    #[test]
    fn supports_identifier_uppercase(){
        let source = "$InPuT > $Output;\nMID";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Port,"$",0,0),
            token!(Identifier,"InPuT",0,1),
            token!(Space," ",0,6),
            token!(Charge,">",0,7),
            token!(Space," ",0,8),
            token!(Port,"$",0,9),
            token!(Identifier,"Output",0,10),
            token!(Semicolon,";",0,16),
            token!(EndLine,"\n",0,17),
            token!(Identifier,"MID",1,0),
        ]);
    }
}