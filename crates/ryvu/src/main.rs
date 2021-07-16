struct Network {
    nodes_state:Vec<NodeState>,
    nodes_connections:Vec<NodeConnection>
}


#[derive(Clone)]
struct NodeState {
    is_being_charged:bool,
    is_being_blocked:bool,
    charged:bool,
    blocked:bool
}

#[derive(Clone)]
struct NodeConnection {
    charging:Vec<usize>,
    blocking:Vec<usize>
}
struct Module{
    connections:Vec<NodeConnection>
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
                for other_index in self.nodes_connections[index].charging.iter() {
                    self.nodes_state[*other_index].is_being_charged = true;
                }
                for other_index in self.nodes_connections[index].blocking.iter() {
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

    fn from_module(module:&Module)->Network{
        let mut states = Vec::with_capacity(module.connections.len());
        for _ in 0..module.connections.len() {
            states.push(NodeState::default());
        }
        Network{
            nodes_connections:module.connections.clone(),
            nodes_state:states
        }
    }
}

fn main() {

}