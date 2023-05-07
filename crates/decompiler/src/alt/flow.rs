use crate::alt::bb::{BasicBlock, BasicBlocks};
use std::collections::HashSet;

struct FlowBlock {}

fn process(blocks: BasicBlocks<'_>) {
    //let mut processed = HashSet::with_capacity(blocks.0.len());
    let mut to_explore = 0;
    while to_explore >= 0 {
        let block = blocks.0.get(&to_explore).expect("Invalid graph");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_process() {}
}
