#[derive(Clone,Default)]
pub struct NodeConnection {
    pub charging:Vec<usize>,
    pub blocking:Vec<usize>
}
#[derive(Default)]
pub struct Module{
    pub connections:Vec<NodeConnection>,
    pub inputs:Vec<usize>,
    pub outputs:Vec<usize>
}

impl Module {
    pub fn expand(&mut self,from:usize,to:usize){
        while !(to < self.connections.len() && from < self.connections.len()) {
            self.connections.push(NodeConnection::default());
        }
    }
    pub fn charge(&mut self,from:usize,to:usize) {
        self.expand(from, to);
        self.connections[from].charging.push(to);
    }
    pub fn block(&mut self,from:usize,to:usize) {
        self.expand(from, to);
        self.connections[from].blocking.push(to);
    }
    pub fn input(&mut self,input:usize){
        self.inputs.push(input);
    }
    pub fn output(&mut self,output:usize){
        self.outputs.push(output);
    }
    pub fn size(&self)->usize{
        self.connections.len()
    }
}