#[derive(PartialEq, Eq, Debug)]
pub enum TokenKind {
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

#[derive(PartialEq, Eq, Debug)]
pub struct Token {
    kind:TokenKind,
    text:String,
    position:TokenPosition
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

macro_rules! token {
    ($k:ident,$t:expr,$l:expr,$c:expr) => {
        Token::new(
            crate::compile::TokenKind::$k,$t.to_string(),
            TokenPosition::new($l,$c)
        )
    };
}


#[derive(Default)]
struct Lexer {
    tokens:Vec<Token>,
    buffer:String,
    line:usize,
    char_index:usize   
}

impl Lexer {
    fn push_space(&mut self){
        if !self.buffer.is_empty() {
            self.tokens.push(
                token!(Space,self.buffer.clone(),self.line,self.char_index)
            );
            self.char_index += self.buffer.len();
            self.buffer.clear();
        }
    }
    fn lex(&mut self,source:&str)->Vec<Token> {
        for ch in source.chars() {
            if ch == ' ' {
                self.buffer.push(ch);
            }
            else if ch == ';' {
                self.push_space();
                self.tokens.push(token!(Semicolon,";",self.line,self.char_index));
                self.char_index += 1;
            }
            else{
                self.push_space();
                self.tokens.push(token!(EndLine,"\n",self.line,self.char_index));
                self.line += 1;
                self.char_index = 0;
            }
        }
        self.push_space();
        std::mem::replace(&mut self.tokens, vec![])
    }
}

pub fn lex(source:&str)->Vec<Token> {
    let mut lexer = Lexer::default();
    lexer.lex(source)
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
    fn supports_semicolons(){
        let source = "  ; \n   \n;";
        let tokens = lex(source);
        assert_eq!(tokens,vec![
            token!(Space,"  ",0,0),
            token!(Semicolon,";",0,2),
            token!(Space," ",0,3),
            token!(EndLine,"\n",0,4),
            token!(Space,"   ",1,0),
            token!(EndLine,"\n",1,3),
            token!(Semicolon,";",2,0)
        ]);
    }
}