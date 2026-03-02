/// Verbindung zwischen zwei neuronalen Knoten
/// Mit binärer Quantisierung: Gewichte sind -1, 0 oder 1
#[derive(Clone, Copy)]
pub struct Connection {
    pub from_node: usize,
    pub to_node: usize,
    pub weight: i8,  // -1, 0 oder 1 (binäre Quantisierung)
    pub enabled: bool,
}

impl Connection {
    pub fn new(from_node: usize, to_node: usize, weight: i8) -> Self {
        Connection {
            from_node,
            to_node,
            weight: Self::quantize_weight(weight),
            enabled: true,
        }
    }

    /// Quantisiert das Gewicht auf -1, 0 oder 1
    fn quantize_weight(weight: i8) -> i8 {
        if weight > 0 {
            1
        } else if weight < 0 {
            -1
        } else {
            0
        }
    }

    // pub fn disable(&mut self) {
    //     self.enabled = false;
    // }

    // pub fn enable(&mut self) {
    //     self.enabled = true;
    // }
}

impl Default for Connection {
    fn default() -> Self {
        Connection::new(0, 1, 0)
    }
}
