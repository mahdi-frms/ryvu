pub use module::{Module,ModuleBuilder};

pub struct Network {
    module:Module,
    states:Vec<NodeState>
}

#[derive(Default)]
struct NodeState (u8);

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
        self.states[index].set_charged(true);        
    }

    pub fn seek(&self,index:usize) -> bool{
        self.states[index].get_charged()
    }
    
    pub fn next(&mut self){
        for index in 0..self.states.len() {
            if self.states[index].get_charged() && !self.states[index].get_blocked() {
                for other_index in self.module.connections[index].charging.iter() {
                    self.states[*other_index].set_being_charged(true);
                }
                for other_index in self.module.connections[index].blocking.iter() {
                    self.states[*other_index].set_being_blocked(true);
                }
            }
        }
        for index in 0..self.module.connections.len() {
            let charged = self.states[index].get_being_charged();
            let blocked = self.states[index].get_being_blocked();
            self.states[index].set_charged(charged);
            self.states[index].set_being_charged(false);
            self.states[index].set_blocked(blocked);
            self.states[index].set_being_blocked(false);
        }
    }
}

impl NodeState {

    fn set_charged(&mut self,value:bool){
        if value {
            self.0 |= 0b0001;
        }
        else{
            self.0 &= 0b1110;
        }
    }

    fn set_blocked(&mut self,value:bool){
        if value {
            self.0 |= 0b0010;
        }
        else{
            self.0 &= 0b1101;
        }
    }

    fn set_being_charged(&mut self,value:bool){
        if value {
            self.0 |= 0b0100;
        }
        else{
            self.0 &= 0b1011;
        }
    }

    fn set_being_blocked(&mut self,value:bool){
        if value {
            self.0 |= 0b1000;
        }
        else{
            self.0 &= 0b0111;
        }
    }

    fn get_charged(&self)->bool {
        self.0 % 2 == 0b0001
    }

    fn get_blocked(&self)->bool {
        self.0 % 4 >= 0b0010
    }

    fn get_being_charged(&self)->bool {
        self.0 % 8 >= 0b0100
    }

    fn get_being_blocked(&self)->bool {
        self.0 % 16 >= 0b1000
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