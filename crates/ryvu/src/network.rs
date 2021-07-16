use std::{rc::Rc};

struct Network {
    nodes_state:Vec<NodeState>,
    module:Rc<module::Module>
}


#[derive(Clone)]
struct NodeState {
    is_being_charged:bool,
    is_being_blocked:bool,
    charged:bool,
    blocked:bool
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

    fn set_input(&mut self,index:usize,state:bool){
        self.nodes_state[self.module.inputs[index]].charged = state;
    }
    fn get_output(&mut self,index:usize)->bool{
        self.nodes_state[self.module.outputs[index]].charged
    }
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

    fn from_module(module:Rc<module::Module>)->Network{
        let mut states = Vec::with_capacity(module.size());
        for _ in 0..module.size() {
            states.push(NodeState::default());
        }
        Network{
            module,
            nodes_state:states
        }
    }
}

#[cfg(test)]
mod test {
    use module::Module;

    #[test]
    fn basic_charging(){
        let mut module = Module::default();
        module.charge(0, 1);
        module.charge(1, 2);
        module.charge(2, 3);
    }
}