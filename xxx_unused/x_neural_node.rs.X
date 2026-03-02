/// Einfacher neuronaler Knoten mit binärer Ausgabe
pub struct NeuralNode {
    pub id: usize,
    pub value: f32,  // Aktivierungswert (0.0 oder 1.0 nach Quantisierung)
}

impl NeuralNode {
    pub fn new(id: usize) -> Self {
        NeuralNode {
            id,
            value: 0.0,
        }
    }

    /// Setzt den Wert und quantisiert auf 0.0 oder 1.0
    pub fn set_value(&mut self, input: f32) {
        self.value = if input > 0.0 { 1.0 } else { 0.0 };
    }

    /// Aktivierungsfunktion: Sigmoid mit binärer Quantisierung
    pub fn activate(&mut self, input: f32) {
        self.set_value(input);
    }
}

impl Default for NeuralNode {
    fn default() -> Self {
        NeuralNode::new(0)
    }
}
