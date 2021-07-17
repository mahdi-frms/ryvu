use crate::module::Module;

#[derive(Default)]
struct NodeState {
    charged:bool,
    blocked:bool,
    being_charged:bool,
    being_blocked:bool
}

pub struct Network {
    module:Module,
    states:Vec<NodeState>
}

impl Network {
    pub fn new(module:Module)-> Network {
        let mut states = vec![];
        for _ in 0..module.connections.len() {
            states.push(NodeState::default());
        }
        Network {
            module,
            states
        }
    }
}