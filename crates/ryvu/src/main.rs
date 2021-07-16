use std::{rc::Rc, usize};

struct Network {
    nodes_state:Vec<NodeState>,
    module:Rc<Module>
}


#[derive(Clone)]
struct NodeState {
    is_being_charged:bool,
    is_being_blocked:bool,
    charged:bool,
    blocked:bool
}

#[derive(Clone,Default)]
struct NodeConnection {
    charging:Vec<usize>,
    blocking:Vec<usize>
}
struct Module{
    connections:Vec<NodeConnection>,
    inputs:Vec<usize>,
    outputs:Vec<usize>
}

impl Default for NodeState {
    fn default() -> NodeState{
        NodeState{
            is_being_blocked:false,
            is_being_charged:false,
            blocked:false,
            charged:false
        }
    }
}

impl Network {
    fn next(&mut self){
        for index in 0..self.nodes_state.len() {
            if self.nodes_state[index].charged && !self.nodes_state[index].blocked {
                for other_index in self.module.connections[index].charging.iter() {
                    self.nodes_state[*other_index].is_being_charged = true;
                }
                for other_index in self.module.connections[index].blocking.iter() {
                    self.nodes_state[*other_index].is_being_blocked = true;
                }
            }
        }

        for index in 0..self.nodes_state.len() {
            if self.nodes_state[index].is_being_charged {
                self.nodes_state[index].charged = true;
            }
            if self.nodes_state[index].is_being_blocked {
                self.nodes_state[index].blocked = true;
            }
        }
    }

    fn from_module(module:Rc<Module>)->Network{
        let mut states = Vec::with_capacity(module.connections.len());
        for _ in 0..module.connections.len() {
            states.push(NodeState::default());
        }
        Network{
            module,
            nodes_state:states
        }
    }
}

impl Module {
    fn expand(&mut self,from:usize,to:usize){
        while !(to < self.connections.len() && from < self.connections.len()) {
            self.connections.push(NodeConnection::default());
        }
    }
    fn charge(&mut self,from:usize,to:usize) {
        self.expand(from, to);
        self.connections[from].charging.push(to);
    }
    fn block(&mut self,from:usize,to:usize) {
        self.expand(from, to);
        self.connections[from].blocking.push(to);
    }
    fn input(&mut self,input:usize){
        self.inputs.push(input);
    }
    fn output(&mut self,output:usize){
        self.outputs.push(output);
    }
}

fn main() {

}