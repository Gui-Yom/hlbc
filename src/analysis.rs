use crate::Function;

/// Basically a register we traced use
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

pub fn function_vars(fun: &Function) -> Vec<Variable> {
    if fun.assigns.is_some() {
        Vec::new()
    } else {
        Vec::new()
    }
}
