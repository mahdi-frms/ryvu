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

    pub fn charge(&mut self,input:usize) {
        self.states[self.module.inputs[input]].charged = true;        
    }
    pub fn seek(&self,input:usize) -> bool{
        self.states[self.module.outputs[input]].charged        
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
    use crate::module::ModuleBuilder;
    use crate::network::Network;

    
    #[test]
    fn input_charging() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        let input_index = builder.input(0);
        let output_index = builder.output(1);

        let mut network = Network::new(builder.build());
        network.charge(input_index);
        network.next();
        let charged = network.seek(output_index);
        
        assert!(charged);
    }

    #[test]
    fn input_charging_discharges_input() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        let input_index = builder.input(0);
        let input_seek_index = builder.output(0);

        let mut network = Network::new(builder.build());
        network.charge(input_index);
        network.next();
        let charged = network.seek(input_seek_index);
        
        assert!(!charged);
    }

    #[test]
    fn input_charging_chain() {
        let mut builder = ModuleBuilder::default();
        builder.charge(0, 1);
        builder.charge(1, 2);
        builder.charge(2, 3);
        builder.charge(3, 4);
        let input_index = builder.input(0);
        let output_indexes = vec![
            builder.output(0),
            builder.output(1),
            builder.output(2),
            builder.output(3),
            builder.output(4)
        ];

        let mut network = Network::new(builder.build());
        network.charge(input_index);
        network.next();
        network.next();
        network.next();
        let charged = network.seek(output_indexes[3]);
        let discharged = 
        !network.seek(output_indexes[0]) && 
        !network.seek(output_indexes[1]) && 
        !network.seek(output_indexes[2]) && 
        !network.seek(output_indexes[4]);
        
        assert!(charged && discharged);
    }
    #[test]
    fn basic_blocking() {
        let mut builder = ModuleBuilder::default();
        builder.block(0, 2);
        builder.charge(1, 2);
        builder.charge(2, 3);
        let input_index = builder.input(0);
        let input_index2 = builder.input(1);
        let output_index = builder.output(3);

        let mut network = Network::new(builder.build());
        network.charge(input_index);
        network.charge(input_index2);
        network.next();
        network.next();
        let charged = network.seek(output_index);
        assert!(!charged);
    }
}