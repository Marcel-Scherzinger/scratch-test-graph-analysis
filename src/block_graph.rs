use std::collections::{HashMap, HashSet};

use derive_more::Deref;
use itertools::Itertools;
use smodel::{
    ProjectDoc,
    attrs::List,
    blocks::{AsOpcodeUnit, BlockWrapper},
};

use crate::graph_construction::add_edges_from_block;

#[derive(Debug, PartialEq, derive_getters::Getters)]
pub struct BlockGraph<'a> {
    doc: &'a smodel::ProjectDoc,
    parameter_edges: HashMap<&'a smodel::Id, Vec<&'a smodel::Id>>,
    read_list_edges: HashMap<&'a smodel::Id, Vec<&'a List>>,
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

    /// (Block, ListId)
    pub fn blocks_directly_reading_list_item_concatenation(
        &self,
    ) -> impl Iterator<
        Item = Result<(&svalue::ARc<BlockWrapper>, &smodel::Id), smodel::error::NoValidBlockForId>,
    > {
        self.read_list_edges()
            .iter()
            .map(|(block_id, lists)| {
                let block = self.doc.get_block(block_id)?;
                Ok(lists.iter().map(move |list_id| (block, list_id.id())))
            })
            .flatten_ok()
    }

    /// There is a block that returns a list as a string where
    /// all its current items are joined with spaces.
    /// Using this block, especially for calculating its length,
    /// can lead to unexpected behaviour because this wouldn't
    /// return the number of items.
    fn check_for_any_block_using_list_item_concatenation(&self) {
        for (block, lists) in self.read_list_edges().iter() {
            let block = self.doc.get_block(block).unwrap();
            let opcode = block.inner().opcode();
            for list_id in lists {
                let list = self
                    .doc
                    .targets()
                    .iter()
                    .flat_map(|target| target.lists().get(list_id.id()))
                    .next()
                    .map(|l| &l.0)
                    .unwrap();
                // println!(
                // "Be aware that there is a block ({opcode}) that reads a list ({list_name:?}) as string concatenation of its items. If you are calculating the length of this value, this is NOT the same as the number of items in the list"
                // );
            }
        }
    }

    pub fn check_no_cycles_in_next_or_param_edges(
        &self,
    ) -> Result<CycleFreeProjectDoc<'a>, CyclicBlockReferences> {
        let initial_blocks = self.doc().ids_with_opcodes().map(|o| o.0).filter(|id| {
            let is_parameter_somewhere = self.parameter_edges().values().flatten().contains(id);
            let is_next_of_any = self.next_block_edges().values().flatten().contains(id);
            !is_parameter_somewhere && !is_next_of_any
        });

        let mut stack = initial_blocks.collect_vec();
        let mut visited = HashSet::new();

        while let Some(block) = stack.pop() {
            if !visited.insert(block) {
                return Err(CyclicBlockReferences::BlockVisitedTwice(block.clone()));
            }
            if let Some(next) = self.next_block_edges().get(block).and_then(|x| x.as_ref()) {
                stack.push(next);
            }
            if let Some(params) = self.parameter_edges().get(block) {
                stack.extend(params);
            }
        }
        let doc_block_count = self.doc().ids_with_opcodes().count();

        if doc_block_count == visited.len() {
            Ok(CycleFreeProjectDoc { doc: self.doc })
        } else if doc_block_count < visited.len() {
            Err(CyclicBlockReferences::VisitedMoreThanInDoc)
        } else
        /* doc_block_count > visited.len() */
        {
            Err(CyclicBlockReferences::CycleWithoutEntry {
                doc_block_count,
                visited_count: visited.len(),
            })
        }
    }
}

/// Proof that [`ProjectDoc`] was checked by [`BlockGraph::check_no_cycles_in_next_or_param_edges`]
#[derive(Debug, PartialEq, Deref)]
pub struct CycleFreeProjectDoc<'a> {
    doc: &'a ProjectDoc,
}

#[derive(Debug, thiserror::Error)]
pub enum CyclicBlockReferences {
    /// Starting from one entry point leads into a repeating cycle
    /// without the entry point
    /// like A -> B -> D -> B
    #[error("found a reference-cycle (with an entry-point not part of cycle)")]
    BlockVisitedTwice(smodel::Id),
    /// has no entry point and wasn't visited,
    /// but the blocks then must form a cycle
    /// like A -> B -> C -> A
    ///
    /// `doc_block_count` is bigger than `visited_count`
    /// and the difference is the number of blocks that
    /// most likely form one or more cycles.
    #[error(
        "found {doc_block_count}-{visited_count} remaining with indegree > 0, cycle(s) assumed"
    )]
    CycleWithoutEntry {
        doc_block_count: usize,
        visited_count: usize,
    },
    #[error("function visited more blocks than registered in document, this should never happen")]
    VisitedMoreThanInDoc,
}
