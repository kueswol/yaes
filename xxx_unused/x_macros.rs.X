/// Enum der Input-Parameter für das Gehirn
macro_rules! brain_inputs {
    ($($name:ident),+ $(,)?) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy)]
        pub enum BrainInput {
            $($name),+
        }

        impl BrainInput {
            pub const COUNT: usize = brain_inputs!(@count $($name),+);

            pub const ALL: [BrainInput; Self::COUNT] = [
                $(BrainInput::$name),+
            ];

            pub const fn index(self) -> usize {
                self as usize
            }
        }
    };

    (@count $($name:ident),+) => {
        <[()]>::len(&[$(brain_inputs!(@unit $name)),+])
    };

    (@unit $_name:ident) => { () };
}
pub(crate) use brain_inputs;

/// Enum der Output-Parameter für das Gehirn
macro_rules! brain_outputs {
    ($($name:ident),+ $(,)?) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy)]
        pub enum BrainOutput {
            $($name),+
        }

        impl BrainOutput {
            pub const COUNT: usize = brain_outputs!(@count $($name),+);

            pub const ALL: [BrainOutput; Self::COUNT] = [
                $(BrainOutput::$name),+
            ];

            pub const fn index(self) -> usize {
                self as usize
            }
        }
    };

    (@count $($name:ident),+) => {
        <[()]>::len(&[$(brain_outputs!(@unit $name)),+])
    };

    (@unit $_name:ident) => { () };
}
pub(crate) use brain_outputs;
