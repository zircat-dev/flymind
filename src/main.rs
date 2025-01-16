use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use csv::ReaderBuilder;

// Reuse or adapt these from your existing code:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronType {
    Sensory,
    Interneuron,
    Motor,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChemicalSubtype {
    Excitatory,
    Inhibitory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SynapseType {
    ChemicalSend(ChemicalSubtype),
    ChemicalReceive(ChemicalSubtype),
    GapJunction,
    NMJ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Head,
    MidBody,
    Tail,
    Unknown,
}

#[derive(Debug)]
pub struct Neuron {
    pub id: usize,
    pub name: String,
    pub neuron_type: NeuronType,
    pub region: Region,
    pub soma_position: f64,
    // ... etc.
    pub membrane_potential: f64,
    pub just_fired: bool,
}

impl Neuron {
    pub fn new(id: usize, name: &str, neuron_type: NeuronType, region: Region, soma_pos: f64) -> Self {
        Self {
            id,
            name: name.to_string(),
            neuron_type,
            region,
            soma_position: soma_pos,
            membrane_potential: 0.0,
            just_fired: false,
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub from_id: usize,
    pub to_id: usize,
    pub synapse_type: SynapseType,
    pub weight: f64,
}

impl Connection {
    pub fn new(from_id: usize, to_id: usize, synapse_type: SynapseType, weight: f64) -> Self {
        Self {
            from_id,
            to_id,
            synapse_type,
            weight,
        }
    }
}

pub struct Network {
    pub neurons: Vec<Neuron>,
    pub connections: Vec<Connection>,
    pub outgoing_map: HashMap<usize, Vec<usize>>,

    // If you have additional fields (for LIF parameters, etc.), include them here
    // ...
}

impl Network {
    pub fn new() -> Self {
        Self {
            neurons: Vec::new(),
            connections: Vec::new(),
            outgoing_map: HashMap::new(),
        }
    }

    pub fn add_neuron(
        &mut self,
        name: &str,
        neuron_type: NeuronType,
        region: Region,
        soma_position: f64,
    ) -> usize {
        let id = self.neurons.len();
        let neuron = Neuron::new(id, name, neuron_type, region, soma_position);
        self.neurons.push(neuron);
        id
    }

    pub fn add_connection(
        &mut self,
        from_id: usize,
        to_id: usize,
        synapse_type: SynapseType,
        weight: f64,
    ) {
        let conn_index = self.connections.len();
        let conn = Connection::new(from_id, to_id, synapse_type, weight);
        self.connections.push(conn);

        self.outgoing_map
            .entry(from_id)
            .or_default()
            .push(conn_index);
    }
    
    // ...
    // (You may have other methods like update_step, run_simulation, etc.)
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Create a new network
    let mut network = Network::new();

    // 2. We'll store a mapping from neuron name -> neuron ID.
    //    That way, if a neuron name appears multiple times, we reuse the same ID.
    let mut neuron_map: HashMap<String, usize> = HashMap::new();

    // 3. Open the CSV/TSV file (assuming tab-delimited).
    //    Adjust the file name/path as appropriate.
    let file = File::open("NeuronConnect.csv")?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)       // We do have headers: Neuron 1, Neuron 2, Type, Nbr
        .delimiter(b',')        // tab-delimited
        .from_reader(BufReader::new(file));

    // 4. For each record (row) in the file:
    for result in rdr.records() {
        let record = result?;

        // Each record corresponds to [Neuron 1, Neuron 2, Type, Nbr]
        let neuron1_name = &record[0];
        let neuron2_name = &record[1];
        let synapse_str  = &record[2];
        let nbr_str      = &record[3];

        // Convert Nbr to a floating-point weight.
        let weight = nbr_str.parse::<f64>().unwrap_or(1.0);

        let from_id = if let Some(&id) = neuron_map.get(neuron1_name) {
            // If neuron1_name is already in the map, return its ID
            id
        } else {
            // Not in the map; create a new neuron
            let new_id = network.neurons.len();
            neuron_map.insert(neuron1_name.to_string(), new_id);
            network.add_neuron(neuron1_name, NeuronType::Other, Region::Unknown, 0.0);
            new_id
        };
        
        // 4b. Same logic for neuron2_name
        let to_id = if let Some(&id) = neuron_map.get(neuron2_name) {
            id
        } else {
            let new_id = network.neurons.len();
            neuron_map.insert(neuron2_name.to_string(), new_id);
            network.add_neuron(neuron2_name, NeuronType::Other, Region::Unknown, 0.0);
            new_id
        };

        // 4c. Convert the Type field (e.g., EJ, Sp, R) into a SynapseType
        //     (Here is a simple mapping—extend as needed.)
        let syn_type = match synapse_str {
            "EJ" => SynapseType::GapJunction,
            "Sp" => SynapseType::ChemicalSend(ChemicalSubtype::Excitatory), // "Sp" could mean "Send polyadic" 
            "R"  => SynapseType::ChemicalReceive(ChemicalSubtype::Excitatory),
            // You can add more cases or default as needed
            _    => SynapseType::ChemicalSend(ChemicalSubtype::Excitatory),
        };

        // 4d. Finally, add the connection to the network
        network.add_connection(from_id, to_id, syn_type, weight);
    }

    // 5. After loading, we can do whatever we want with the network,
    //    e.g., debug print the total number of neurons and connections:

    println!("Loaded {} neurons", network.neurons.len());
    println!("Loaded {} connections", network.connections.len());

    // For example, list a few of them:
    for (i, conn) in network.connections.iter().take(10).enumerate() {
        let from_name = &network.neurons[conn.from_id].name;
        let to_name   = &network.neurons[conn.to_id].name;
        println!(
            "Conn {i}: {} -> {} (type={:?}, weight={})",
            from_name, to_name, conn.synapse_type, conn.weight
        );
    }

    // 6. Return OK
    Ok(())
}
