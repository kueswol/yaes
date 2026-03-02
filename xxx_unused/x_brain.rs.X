use crate::neural_node::NeuralNode;
use crate::connection::Connection;
use rand::Rng;

/// Das Gehirn eines Creatures - einfaches neuronales Netz mit binärer Quantisierung
pub struct X_Brain {
    pub input_nodes: Vec<NeuralNode>,   // Sensoren: Energie, Pos X, Pos Y
    pub hidden_nodes: Vec<NeuralNode>,  // Versteckte Schicht (optional)
    pub output_nodes: Vec<NeuralNode>,  // Ausgabe: Bewegungsrichtung
    pub connections: Vec<Connection>,   // Verbindungen zwischen Knoten
}

impl X_Brain {
    /// Erstellt ein neues Gehirn mit Standard-Konfiguration
    /// Inputs: 3 (energy, pos_x, pos_y)
    /// Hidden: 2
    /// Outputs: 4 (up, down, left, right)
    pub fn new() -> Self {
        let mut brain = Brain {
            input_nodes: vec![
                NeuralNode::new(0),   // Energy
                NeuralNode::new(1),   // Pos X
                NeuralNode::new(2),   // Pos Y
            ],
            hidden_nodes: vec![
                NeuralNode::new(3),
                NeuralNode::new(4),
                NeuralNode::new(5),
                NeuralNode::new(6),
                NeuralNode::new(7),
                NeuralNode::new(8),
            ],
            output_nodes: vec![
                NeuralNode::new(9),   // Up
                NeuralNode::new(10),   // Down
                NeuralNode::new(11),   // Left
                NeuralNode::new(12),   // Right
            ],
            connections: Vec::new(),
        };

        // Standard-Verbindungen (Input -> Hidden -> Output)
        brain.add_default_connections();
        brain
    }

    /// Fügt Standard-Verbindungen hinzu
    fn add_default_connections(&mut self) {
        
        let mut rng = rand::thread_rng();

        // Input zu Hidden
        for i in 0..3 {
            for j in 3..9 {
                self.connections.push(Connection::new(i, j, rng.gen_range(-127..=128) as i8));
            }
        }

        // Hidden zu Output
        for i in 3..9 {
            for j in 9..13 {
                self.connections.push(Connection::new(i, j, rng.gen_range(-127..=128) as i8));
            }
        }
    }

    /// Setzt die Eingänge (Sensoren)
    pub fn set_inputs(&mut self, energy: f32, pos_x: f32, pos_y: f32) {
        self.input_nodes[0].activate(energy);
        self.input_nodes[1].activate(pos_x);
        self.input_nodes[2].activate(pos_y);
    }

    /// Führt ein Forward-Pass durch das Netz durch
    pub fn forward(&mut self) {
        // Hidden Layer aktivieren
        for hidden_node in &mut self.hidden_nodes {
            let mut sum = 0.0;
            for conn in &self.connections {
                if conn.enabled && conn.to_node == hidden_node.id {
                    if let Some(input) = self.input_nodes.iter().find(|n| n.id == conn.from_node) {
                        sum += input.value * conn.weight as f32;
                    }
                }
            }
            hidden_node.activate(sum);
        }

        // Output Layer aktivieren
        for output_node in &mut self.output_nodes {
            let mut sum = 0.0;
            for conn in &self.connections {
                if conn.enabled && conn.to_node == output_node.id {
                    if let Some(hidden) = self.hidden_nodes.iter().find(|n| n.id == conn.from_node) {
                        sum += hidden.value * conn.weight as f32;
                    }
                }
            }
            output_node.activate(sum);
        }
    }

    /// Gibt die beste Ausgabe zurück (welcher Output am höchsten)
    pub fn get_direction(&self) -> usize {
        let mut best_idx = 0;
        let mut best_val = self.output_nodes[0].value;

        for (idx, node) in self.output_nodes.iter().enumerate() {
            if node.value > best_val {
                best_val = node.value;
                best_idx = idx;
            }
        }

        best_idx // 0: Up, 1: Down, 2: Left, 3: Right
    }

    // /// Gibt die Verbindungsanzahl zurück
    // pub fn connection_count(&self) -> usize {
    //     self.connections.len()
    // }
}

impl Default for X_Brain {
    fn default() -> Self {
        Brain::new()
    }
}

impl ToString for X_Brain {
    fn to_string(&self) -> String {
        format!(
            "[I:{},H:{},O:{},C:{}]",
            self.input_nodes.len(),
            self.hidden_nodes.len(),
            self.output_nodes.len(),
            self.connections.len()
        )
    }
}
