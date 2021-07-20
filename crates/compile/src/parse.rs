use crate::{lex::{SourcePosition, Token, TokenKind}, translate::{Connection, IdentKind, Identifier}};

#[derive(Default)]
struct Parser{
    state:ParserState,
    connections:Vec<Connection>,
    buffer:String,
    is_charge:bool,
    is_port:bool,
    errors:Vec<ParserError>
}

#[derive(PartialEq, Eq)]
enum ParserState {
    Statement,
    Operator,
    Identifier,
    PortIdent,
    PortStmt,
    Terminate,
    Error
}

#[derive(Debug,PartialEq, Eq)]
enum ParserError {
    UnexpectedToken(SourcePosition),
    UnexpectedEnd
}

fn parse(tokens:Vec<Token>,errors:Vec<ParserError>)->(Vec<Connection>,Vec<ParserError>) {
    Parser::default().parse(tokens,errors)
}

impl Parser {

    fn parse(&mut self,tokens:Vec<Token>,errors:Vec<ParserError>) -> (Vec<Connection>,Vec<ParserError>) {
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
            TokenKind::Charge | TokenKind::Block=>{
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
        if self.state == ParserState::Operator || self.state == ParserState::Identifier {
            self.unexpected_end();
        }
        self.state = ParserState::Statement;

        (
            std::mem::replace(&mut self.connections, vec![]),
            std::mem::replace(&mut self.errors, vec![])
        )
    }

    fn handle_ident(&mut self,token:&Token){
        match self.state {
            ParserState::Statement => {
                self.buffer = token.text().to_owned();
                self.state = ParserState::Operator;
                self.is_port = false;
            },
            ParserState::PortStmt => {
                self.buffer = token.text().to_owned();
                self.state = ParserState::Operator;
                self.is_port = true;
            },
            ParserState::Identifier => {
                self.connect(token.text(),false);
                self.buffer = token.text().to_owned();
                self.is_port = false;
                self.state = ParserState::Terminate;
            },
            ParserState::PortIdent => {
                self.connect(token.text(),true);
                self.buffer = token.text().to_owned();
                self.is_port = true;
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
            ParserState::PortIdent | ParserState::PortStmt => {
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
                self.buffer = String::new();
                self.state = ParserState::Statement;
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
                    self.is_charge = true;
                }
                else{
                    self.is_charge = false;
                }
                self.state = ParserState::Identifier;
            },
            _ =>{
                self.unexpected_error(token);
                self.state = ParserState::Error;
            }
        }
    }

    fn handle_port(&mut self,token:&Token){
        match self.state {
            ParserState::Identifier => {
                self.state = ParserState::PortIdent;
            },
            ParserState::Statement => {
                self.state = ParserState::PortStmt;
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
                self.buffer = String::new();
                self.state = ParserState::Statement;
            },
            _ => {
                // nothing
            }
        }
    }

    fn connect(&mut self,token_text:&str,port:bool){
        let from = Identifier::new(self.buffer.clone(),if self.is_port {
            IdentKind::InPort
        }
        else{
            IdentKind::Node
        });
        let to = Identifier::new(token_text.to_owned(),if port {
            IdentKind::OutPort
        }
        else{
            IdentKind::Node
        });
        self.connections.push(Connection::new(from, to, self.is_charge));
    }

    fn unexpected_error(&mut self,token:&Token){
        self.errors.push(ParserError::UnexpectedToken(token.position()))
    }

    fn unexpected_end(&mut self){
        self.errors.push(ParserError::UnexpectedEnd);
    }
}

impl Default for ParserState {
    fn default() -> Self {
        ParserState::Statement
    }
}

#[cfg(test)]
mod test {
    use crate::{lex::{SourcePosition,Token}, translate::Connection, parse::{parse,ParserError}};

    fn parser_test_case(tokens:Vec<Token>,connections:Vec<Connection>){
        let generated_connections = parse(tokens,vec![]).0;
        assert_eq!(generated_connections,connections);
    }

    fn parse_error_test_case(tokens:Vec<Token>,errors:Vec<ParserError>){
        let generated_errors = parse(tokens,vec![]).1;
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
}