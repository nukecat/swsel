use std::fs::File;

use sw_structure_io::structs::*;
use sw_structure_io::io::WriteBuilding;

fn main() {
    let version = 0;

    let mut building = Building::default();
    let root = Root::default();
    let mut block = Block::default();

    building.roots.push(root);

    let hello_world = [
        "#  #      # #          #     #           #    #",
        "#  #  ##  # #  ##      #     #  ##  # ## #  ###",
        "#### #### # # #  #     #  #  # #  # ##   # #  #",
        "#  # #    # # #  #      # # #  #  # #    # #  #",
        "#  #  ### # #  ##        # #    ##  #    #  ## ",
    ];

    for y in 0..hello_world.len() {
        let string = hello_world[y];
        for x in 0..string.len() {
            if string.as_bytes()[x] == '#' as u8 {
                block.position = [x as f32, 0f32 - (y as f32), 0f32];

                building.blocks.push(block.clone());
            }
        }
    }

    let mut file = File::create("example_building.structure").unwrap();
    file.write_building(&building, version).unwrap();
}