#[derive(Default)]
pub struct NodeConnections {
    charging:Vec<usize>,
    blocking:Vec<usize>
}

#[derive(Default)]
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
        self.expand(std::cmp::max(from,to));
        self.module.connections[from].charging.push(to);
    }
    pub fn block(&mut self,from:usize,to:usize) {
        self.expand(std::cmp::max(from,to));
        self.module.connections[from].blocking.push(to);
    }
    pub fn input(&mut self,index:usize) {
        self.expand(index+1);
        self.module.inputs.push(index);
    }
    pub fn output(&mut self,index:usize) {
        self.expand(index+1);
        self.module.outputs.push(index);
    }
    pub fn build(&mut self) -> Module {
        std::mem::replace(&mut self.module,Module::default())
    }
}