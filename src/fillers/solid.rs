use crate::core::{OpSet, OpSetType};

pub fn empty_solid_fill() -> OpSet {
    OpSet {
        set_type: OpSetType::FillPath,
        ops: Vec::new(),
        size: None,
        path: None,
    }
}
