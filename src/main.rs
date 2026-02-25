use smodel::blocks::AsOpcodeUnit;

fn main() {
    let json = smodel::json_from_sb3_file("sb3/direct-len-list-nesting.sb3").unwrap();
    let doc = smodel::ProjectDoc::from_json(&json).unwrap();

    let graph = scratch_test_graph_analysis::BlockGraph::new(&doc);

    for (block, lists) in graph.read_list_edges().iter() {
        let block = doc.get_block(block).unwrap();
        let opcode = block.inner().opcode();
        for list in lists {
            let list_name = doc
                .targets()
                .iter()
                .flat_map(|target| target.lists().get(list))
                .next()
                .map(|l| l.0.name())
                .unwrap();
            println!(
                "Be aware that there is a block ({opcode}) that reads a list ({list_name:?}) as string concatenation of its items. If you are calculating the length of this value, this is NOT the same as the number of items in the list"
            );
        }
    }

    println!("{graph:#?}");
}
