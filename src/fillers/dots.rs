use crate::core::{OpSet, OpSetType};

pub fn empty_dot_fill() -> OpSet {
    OpSet {
        set_type: OpSetType::FillSketch,
        ops: Vec::new(),
        size: None,
        path: None,
    }
}
