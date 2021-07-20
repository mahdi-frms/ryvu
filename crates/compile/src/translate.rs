use std::collections::HashMap;
use module::{Module, ModuleBuilder};

type IndexMap = HashMap<String,(usize,IdentKind)>;

#[derive(Debug,PartialEq, Eq)]
enum TranslatorError {
    InconstIdent(String,IdentKind,IdentKind)
}

#[derive(Default)]
struct Translator {
    errors:Vec<TranslatorError>,
    indexes:IndexMap
}

#[derive(PartialEq, Eq, Debug)]
pub struct Connection {
    from: Identifier,
    to: Identifier,
    is_charge:bool
}

#[derive(PartialEq, Eq, Debug)]
pub struct Identifier {
    name:String,
    kind:IdentKind,
}

#[derive(Debug,PartialEq,Eq,Clone, Copy)]
pub enum IdentKind {
    Node,
    InPort,
    OutPort
}

#[derive(PartialEq, Eq)]
enum PortIndexResult {
    Error,
    Normal,
    NewPort
}

macro_rules! connection {
    ($f:ident > $t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::Node),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::Node),
        true)
    };
    (!$f:ident > $t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::InPort),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::Node),
        true)
    };
    ($f:ident > !$t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::Node),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::OutPort),
        true)
    };
    (!$f:ident > !$t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::InPort),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::OutPort),
        true)
    };
    ($f:ident . $t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::Node),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::Node),
        false)
    };
    (!$f:ident . $t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::InPort),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::Node),
        false)
    };
    ($f:ident . !$t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::Node),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::OutPort),
        false)
    };
    (!$f:ident . !$t:ident) => {
        Connection::new(
            crate::translate::Identifier::new(stringify!($f).to_owned(), crate::translate::IdentKind::InPort),
            crate::translate::Identifier::new(stringify!($t).to_owned(), crate::translate::IdentKind::OutPort),
        false)
    };
}

fn translate(connections:Vec<Connection>,errors:Vec<TranslatorError>)->(Module,Vec<TranslatorError>) {
    Translator::default().translate(connections, errors)
}

impl Translator {

    fn translate(&mut self,connections:Vec<Connection>,errors:Vec<TranslatorError>)->(Module,Vec<TranslatorError>) {
        self.errors = errors;
        let mut builder = ModuleBuilder::default();
        for con in connections.iter() {

            let from = self.index(&con.from);
            let to = self.index(&con.to);

            if from.1 != PortIndexResult::Error && to.1 != PortIndexResult::Error {
                builder.connect(from.0, to.0, con.is_charge);
                if from.1 == PortIndexResult::NewPort {
                    builder.input(from.0);
                }
                if to.1 == PortIndexResult::NewPort {
                    builder.output(to.0);
                }
            }
        }
        (
            builder.build(),
            std::mem::replace(&mut self.errors, vec![])
        )
    }

    fn index(&mut self,ident:&Identifier)->(usize,PortIndexResult) {
        match self.indexes.get(&ident.name).copied() {
            Some((index,orig_kind))=> {
                if orig_kind == ident.kind {
                    (index,PortIndexResult::Normal)
                }
                else{  
                    self.inconst_ident(ident,orig_kind);
                    (index,PortIndexResult::Error)
                }
            },
            None=>{
                self.indexes.insert(ident.name.clone(), (self.indexes.len(),ident.kind));
                (self.indexes.len()-1,if ident.kind != IdentKind::Node {
                    PortIndexResult::NewPort
                }
                else{
                    PortIndexResult::Normal
                })
            }
        }
    }

    fn inconst_ident(&mut self,ident:&Identifier,orig_kind:IdentKind){
        self.errors.push(TranslatorError::InconstIdent(ident.name.clone(),ident.kind,orig_kind));
    }
}

impl Connection {
    pub fn new(from: Identifier, to: Identifier, is_charge: bool) -> Connection {
        Connection { from, to, is_charge }
    }
}

impl Identifier {
    pub fn new(name: String, kind: IdentKind) -> Identifier {
        Identifier { name, kind } 
    }
}

#[cfg(test)]
mod test {

    use module::ModuleBuilder;
    use crate::translate::{Connection, IdentKind, Module, TranslatorError, translate};

    fn translate_test_case(connections:Vec<Connection>,module:Module){
        let compiled_module = translate(connections,vec![]).0;
        assert_eq!(compiled_module,module);
    }

    fn translate_error_test_case(connections:Vec<Connection>,errors:Vec<TranslatorError>){
        let generated_errors = translate(connections,vec![]).1;
        assert_eq!(generated_errors,errors);
    }

    #[test]
    fn single_connection(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        translate_test_case(vec![
            connection!(a > b)
        ], builder.build())
    }

    #[test]
    fn multiple_connection(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.block(0, 2);
        builder.charge(2, 0);
        builder.block(1, 3);
        builder.charge(3, 3);
        translate_test_case(vec![
            connection!(a > b),
            connection!(a . c),
            connection!(c > a),
            connection!(b . d),
            connection!(d > d)
        ], builder.build())
    }

    #[test]
    fn single_input_single_use(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.block(1, 2);
        builder.input(0);
        translate_test_case(vec![
            connection!(!a > b),
            connection!(b . c),
        ], builder.build())
    }

    #[test]
    fn single_output_single_use(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.block(1, 2);
        builder.output(2);
        translate_test_case(vec![
            connection!(a > b),
            connection!(b . !c),
        ], builder.build())
    }

    #[test]
    fn single_input_multiple_use(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.charge(0, 2);
        builder.input(0);
        translate_test_case(vec![
            connection!(!a > b),
            connection!(!a > c)
        ], builder.build())
    }

    #[test]
    fn single_output_multiple_use(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.charge(2, 1);
        builder.output(1);
        translate_test_case(vec![
            connection!(a > !b),
            connection!(c > !b)
        ], builder.build())
    }

    #[test]
    fn error_on_node_inport_inconsistency(){
        translate_error_test_case(vec![
            connection!(!a > b),
            connection!(a > c)
        ], vec![
            TranslatorError::InconstIdent(String::from("a"),IdentKind::Node,IdentKind::InPort)
        ])
    }

    #[test]
    fn error_on_node_output_inconsistency(){
        translate_error_test_case(vec![
            connection!(a > !b),
            connection!(c > b)
        ], vec![
            TranslatorError::InconstIdent(String::from("b"),IdentKind::Node,IdentKind::OutPort)
        ])
    }

    #[test]
    fn error_on_input_output_inconsistency(){
        translate_error_test_case(vec![
            connection!(a > !b),
            connection!(!b > c)
        ], vec![
            TranslatorError::InconstIdent(String::from("b"),IdentKind::InPort,IdentKind::OutPort)
        ])
    }
}