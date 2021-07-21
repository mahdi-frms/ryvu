pub use module::{Module,ModuleBuilder};

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

    pub fn charge(&mut self,index:usize) {
        self.states[index].charged = true;        
    }
    pub fn seek(&self,index:usize) -> bool{
        self.states[index].charged        
    }
    pub fn next(&mut self){
        for index in 0..self.states.len() {
            if self.states[index].charged && !self.states[index].blocked {
                for other_index in self.module.connections[index].charging.iter() {
                    self.states[*other_index].being_charged = true;
                }
                for other_index in self.module.connections[index].blocking.iter() {
                    self.states[*other_index].being_blocked = true;
                }
            }
        }
        for index in 0..self.module.connections.len() {
            self.states[index].charged = self.states[index].being_charged;
            self.states[index].being_charged = false;
            self.states[index].blocked= self.states[index].being_blocked;
            self.states[index].being_blocked = false;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network::ModuleBuilder;
    use crate::network::Network;

    
    #[test]
    fn input_charging() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);

        let mut network = Network::new(builder.build());
        network.charge(0);
        network.next();
        let charged = network.seek(1);
        
        assert!(charged);
    }

    #[test]
    fn input_charging_discharges_input() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);

        let mut network = Network::new(builder.build());
        network.charge(0);
        network.next();
        let charged = network.seek(0);
        
        assert!(!charged);
    }

    #[test]
    fn input_charging_chain() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.charge(1, 2);
        builder.charge(2, 3);
        builder.charge(3, 4);

        let mut network = Network::new(builder.build());
        network.charge(0);
        network.next();
        network.next();
        network.next();
        let charged = network.seek(3);
        let discharged = 
        !network.seek(0) && 
        !network.seek(1) && 
        !network.seek(2) && 
        !network.seek(4);
        
        assert!(charged && discharged);
    }
    #[test]
    fn basic_blocking() {
        let mut builder = ModuleBuilder::default();
        builder.block(0, 2);
        builder.charge(1, 2);
        builder.charge(2, 3);

        let mut network = Network::new(builder.build());
        network.charge(0);
        network.charge(1);
        network.next();
        network.next();
        let charged = network.seek(3);
        assert!(!charged);
    }
}