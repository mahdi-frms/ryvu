use std::collections::HashSet;

#[derive(Default,PartialEq, Eq, Debug)]
pub struct NodeConnections {
    pub charging:HashSet<usize>,
    pub blocking:HashSet<usize>
}

#[derive(Default,PartialEq, Eq,Debug)]
pub struct Module {
    pub connections:Vec<NodeConnections>,
    pub inputs:Vec<usize>,
    pub outputs:Vec<usize>
}

#[derive(Default)]
pub struct ModuleBuilder {
    module:Module
}

impl ModuleBuilder {
    fn expand(&mut self,count:usize) {
        while self.module.connections.len() < count {
            self.module.connections.push(NodeConnections::default());
        }        
    }
    pub fn charge(&mut self,from:usize,to:usize) {
        self.expand(std::cmp::max(from,to)+1);
        self.module.connections[from].charging.insert(to);
    }
    pub fn block(&mut self,from:usize,to:usize) {
        self.expand(std::cmp::max(from,to)+1);
        self.module.connections[from].blocking.insert(to);
    }
    pub fn connect(&mut self,from:usize,to:usize,is_charge:bool) {
        self.expand(std::cmp::max(from,to)+1);
        if is_charge {
            self.module.connections[from].charging.insert(to);
        }
        else{
            self.module.connections[from].blocking.insert(to);
        }
    }
    pub fn input(&mut self,index:usize) -> usize {
        self.expand(index+1);
        self.module.inputs.push(index);
        self.module.inputs.len() - 1
    }
    pub fn output(&mut self,index:usize) -> usize {
        self.expand(index+1);
        self.module.outputs.push(index);
        self.module.outputs.len() - 1
    }
    pub fn build(&mut self) -> Module {
        std::mem::replace(&mut self.module,Module::default())
    }
}