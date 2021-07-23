use std::collections::HashMap;
use module::{Module, ModuleBuilder};

type IndexMap = HashMap<String,usize>;

#[derive(Default)]
struct Translator {
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

#[allow(unused_macros)]
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

pub fn translate(connections:Vec<Connection>)->Module {
    Translator::default().translate(connections)
}

impl Translator {

    fn translate(&mut self,connections:Vec<Connection>)->Module {
        let mut builder = ModuleBuilder::default();
        for con in connections.iter() {

            let (from_idx,from_new) = self.index(&con.from);
            let (to_idx,to_new) = self.index(&con.to);

            builder.connect(from_idx, to_idx, con.is_charge);
            if from_new && con.from.kind == IdentKind::InPort {
                builder.input(from_idx);
            }
            if to_new && con.to.kind == IdentKind::OutPort{
                builder.output(to_idx);
            }
        }
        builder.build()
    }

    fn index(&mut self,ident:&Identifier)->(usize,bool) {
        match self.indexes.get(&ident.name).copied() {
            Some(index)=> {
                (index,false)
            },
            None=>{
                self.indexes.insert(ident.name.clone(), self.indexes.len());
                (self.indexes.len()-1,true)
            }
        }
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
    use crate::translate::{Connection, Module, translate};

    fn translate_test_case(connections:Vec<Connection>,module:Module){
        let compiled_module = translate(connections);
        assert_eq!(compiled_module,module);
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
    fn repeated_connection(){
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        translate_test_case(vec![
            connection!(a > b),
            connection!(a > b)
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
}