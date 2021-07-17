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


macro_rules! pos {
    ($l:expr,$c:expr) => {
        TokenPosition::new($l,$c)
    };
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
    let mut tokens = vec![];
    let mut buffer = String::default();
    let mut line = 0usize;
    let mut char_index = 0;
    for ch in source.chars() {
        if ch == ' ' {
            buffer.push(ch);
        }
        else if ch == ';' {
            if !buffer.is_empty() {
                tokens.push(
                    Token::new(TokenKind::Space,buffer.clone(),pos!(line,char_index))
                );
                char_index += buffer.len();
                buffer.clear();
            }
            tokens.push(Token::new(TokenKind::Semicolon, ";".to_string(), pos!(line,char_index)));
            char_index += 1;
        }
        else{
            if !buffer.is_empty() {
                tokens.push(
                    Token::new(TokenKind::Space,buffer.clone(),pos!(line,char_index))
                );
                char_index += buffer.len();
                buffer.clear();
            }
            tokens.push(Token::new(TokenKind::EndLine, "\n".to_string(), pos!(line,char_index)));
            line += 1;
            char_index = 0;
        }
    }

    if !buffer.is_empty() {
        tokens.push(
            Token::new(TokenKind::Space,buffer.clone(),pos!(line,char_index))
        );
        char_index += buffer.len();
        buffer.clear();
    }
    tokens
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