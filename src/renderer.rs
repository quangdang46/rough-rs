use crate::core::{OpSet, OpSetType, ResolvedOptions};

pub fn empty_path(_options: &ResolvedOptions) -> OpSet {
    OpSet {
        set_type: OpSetType::Path,
        ops: Vec::new(),
        size: None,
        path: None,
    }
}
