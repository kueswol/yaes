pub struct Helper {
    pub last_message: String,
}

impl Helper {
    pub fn new() -> Helper {
        Helper {
            last_message: String::new(),
        }
    }

    pub fn get_random_string(&mut self) -> String {
        let i: u8 = rand::random::<u8>() % 3;
        let result = match i {
            0 => String::from("Hello, World!"),
            1 => String::from("Greetings, Universe!"),
            _ => String::from("Salutations, Cosmos!"),
        };
        self.last_message = result.clone();
        result
    }
}

///////////////////////////////////////////////////////////////////////////////////
// DEMO main():
///////////////////////////////////////////////////////////////////////////////////
// mod test;
// fn main() {
//     let mut helper = test::Helper::new();
//     for i in 0..9 {
//         println!("{}: {}", i+1, helper.get_random_string());
//     }
//     println!("\nLetzter gespeicherter Wert: {}", helper.last_message);
// }