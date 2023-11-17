pub(crate) struct Example {
    pub(crate) name: &'static str,
    pub(crate) source: &'static str,
    pub(crate) data: &'static [u8],
}

// TODO maybe build examples as part of the gui build ?

pub(crate) const EXAMPLES: [Example; 1] = [Example {
    name: "Basic",
    source: include_str!("../examples/Basic.hx"),
    data: include_bytes!("../examples/Basic.hl"),
}];
