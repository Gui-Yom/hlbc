pub(crate) struct Example {
    pub(crate) name: &'static str,
    pub(crate) source: &'static str,
    pub(crate) data: &'static [u8],
}

macro_rules! examples {
    ($n:literal; $($name:literal)+) => {
        pub(crate) const EXAMPLES: [Example; $n] = [
            $(
            Example {
                name: $name,
                source: include_str!(concat!("../examples/", $name, ".hx")),
                data: include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".hl")),
            }
            ),+
        ];
    };
}

// Remember to add examples to build.rs
examples!(2; "Basic" "BranchExpr");
