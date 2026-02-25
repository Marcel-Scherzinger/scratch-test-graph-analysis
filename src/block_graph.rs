use std::collections::HashMap;

use crate::graph_construction::add_edges_from_block;

#[derive(Debug, PartialEq, derive_getters::Getters)]
pub struct BlockGraph<'a> {
    doc: &'a smodel::ProjectDoc,
    parameter_edges: HashMap<&'a smodel::Id, Vec<&'a smodel::Id>>,
    read_list_edges: HashMap<&'a smodel::Id, Vec<&'a smodel::Id>>,
    next_block_edges: HashMap<&'a smodel::Id, Option<&'a smodel::Id>>,
    parent_block_edges: HashMap<&'a smodel::Id, Option<&'a smodel::Id>>,
}

impl<'a> BlockGraph<'a> {
    pub fn new(doc: &'a smodel::ProjectDoc) -> Self {
        let mut parameter_edges = HashMap::new();
        let mut read_list_edges = HashMap::new();
        let mut next_block_edges = HashMap::new();
        let mut parent_block_edges = HashMap::new();
        for from_block in doc.targets().iter().flat_map(|t| t.blocks().iter_blocks()) {
            add_edges_from_block(
                &mut parameter_edges,
                &mut read_list_edges,
                &mut next_block_edges,
                &mut parent_block_edges,
                from_block,
            );
        }
        Self {
            doc,
            parameter_edges,
            read_list_edges,
            next_block_edges,
            parent_block_edges,
        }
    }
}
