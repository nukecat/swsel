use crate::{
    block::*,
    building::*,
    io::{utils::*, write::version},
    root::*,
};
use byteorder::{LE, WriteBytesExt};
use indexmap::IndexSet;
use std::io::Write;
use std::{ops::Deref, rc::Rc};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static VERSION: u8 = 0;

pub(crate) struct SerializableBuilding {
    pub(crate) version: u8,
    pub(crate) roots: Vec<SerializableRoot>,
    pub(crate) blocks: Vec<SerializableBlock>,
    pub(crate) block_ids: IndexSet<*const Block>,

    pub(crate) rotations: Vec<[u16; 3]>,
    pub(crate) colors: Vec<u16>,
}

impl SerializableBuilding {
    fn new(building: &Building, version: u8) -> Result<Self> {
        // Basic initialization
        let (mut root_count, mut block_count): (usize, usize) = (0, 0);
        for root in building.roots.iter() {
            root_count += 1;
            block_count += root.blocks.borrow().len();
        }
        //
        assert!(root_count <= 0xFFFF);
        assert!(block_count <= 0xFFFF);
        //
        let mut roots: Vec<SerializableRoot> = Vec::new();
        let mut blocks: Vec<SerializableBlock> = Vec::new();
        let mut block_ids: IndexSet<*const Block> = IndexSet::new();
        //
        for root in building.roots.iter() {
            for block in root.blocks.borrow().iter() {
                blocks.push(SerializableBlock::new(block.clone()));
                block_ids.insert(Rc::as_ptr(block));
            }
            roots.push(SerializableRoot::new(root.clone()));
        }
        // ---

        // Root ids
        // assert!(root_count <= 0xFF);
        // let (mut root_index, mut block_index): (usize, usize) = (0, 0);
        // for root in building.roots.iter() {
        //     for block in root.blocks.borrow().iter() {
        //        blocks[block_index].root_id = root_index as u8;
        //     }
        //     root_index += 1;
        // }
        // ---

        // Positions
        let (mut root_index, mut block_index): (usize, usize) = (0, 0);
        for root in building.roots.iter() {
            for block in root.blocks.borrow().iter() {
                roots[root_index].bounds.encapsulate(&block.position.get());
                block_index += 1;
            }
            root_index += 1;
        }
        let (mut root_index, mut block_index): (usize, usize) = (0, 0);
        for root in building.roots.iter() {
            for block in root.blocks.borrow().iter() {
                blocks[block_index].position_inbounds =
                    roots[root_index].bounds.to_inbounds(block.position.get());
                block_index += 1;
            }
            root_index += 1;
        }
        // ---

        // Last block indexes
        let (mut root_index, mut block_index): (usize, usize) = (0, 0);
        for root in building.roots.iter() {
            block_index += root.blocks.borrow().len();
            roots[root_index].last_block_index = block_index.checked_sub(1).unwrap_or(0) as u16;
            root_index += 1;
        }
        // ---

        // Rotations
        let mut rotations: IndexSet<[u16; 3]> = IndexSet::new();
        for block in blocks.iter_mut() {
            block.rotation_index =
                rotations.insert_full(pack_rotation(block.rotation.get())).0 as u16;
        }
        let single_byte_rotation = rotations.len() <= 0xFF;
        let avg_rotations = blocks.len() as f32 / rotations.len() as f32;
        if avg_rotations > (if single_byte_rotation { 1.2f32 } else { 1.5f32 }) {
            rotations = IndexSet::new();
        }
        assert!(rotations.len() <= 0xFFFF);
        let rotations: Vec<[u16; 3]> = rotations.into_iter().collect();
        // ---

        // Colors
        let mut colors: IndexSet<u16> = IndexSet::new();
        let mut colored_count: usize = 0;
        for block in blocks.iter_mut() {
            if let Some(color) = block.color.get() {
                if colors.len() >= 0xFF {
                    break;
                }
                block.color_index = colors.insert_full(pack_color(color)).0 as u8;
                colored_count += 1;
            }
        }
        let avg_colors = colored_count as f32 / colors.len() as f32;
        if avg_colors > 2.0f32 && colors.len() >= 0xFF {
            colors = IndexSet::new()
        }
        let colors: Vec<u16> = colors.into_iter().collect();
        // ---

        Ok(Self {
            version,
            roots,
            blocks,
            block_ids,

            rotations,
            colors,
        })
    }
}

pub(crate) struct SerializableRoot {
    pub(crate) root: Rc<Root>,

    pub(crate) bounds: Bounds,
    pub(crate) last_block_index: u16,
}

impl Deref for SerializableRoot {
    type Target = Rc<Root>;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl SerializableRoot {
    fn new(root: Rc<Root>) -> Self {
        Self {
            root,

            bounds: Bounds::new(),
            last_block_index: 0,
        }
    }
}

pub(crate) struct SerializableBlock {
    pub(crate) block: Rc<Block>,

    pub(crate) position_inbounds: [i16; 3],
    pub(crate) rotation_index: u16,
    // pub(crate) root_id           : u8,
    pub(crate) color_index: u8,
}

impl Deref for SerializableBlock {
    type Target = Rc<Block>;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl SerializableBlock {
    fn new(block: Rc<Block>) -> Self {
        Self {
            block,

            position_inbounds: [0; 3],
            rotation_index: 0,
            // root_id: 0,
            color_index: 0,
        }
    }
}

pub(crate) fn write_building<W: Write>(mut w: W, building: &Building) -> Result<()> {
    let building = SerializableBuilding::new(building, VERSION)?;

    w.write_u8(VERSION)?;

    if !building.colors.is_empty() {
        w.write_u8(building.colors.len() as u8)?;
        w.write_array_u16::<LE>(&building.colors)?;
    } else {
        w.write_u8(0xFF)?;
    }

    if !building.rotations.is_empty() {
        w.write_u16::<LE>(building.rotations.len() as u16)?;
        for r in building.rotations.iter() {
            w.write_array_u16::<LE>(r)?;
        }
    } else {
        w.write_u16::<LE>(0xFFFF)?;
    }

    w.write_u16::<LE>(u16::try_from(building.roots.len())?)?;
    for root in building.roots.iter() {
        write_root(&mut w, &root, &building)?;
    }

    w.write_u16::<LE>(u16::try_from(building.blocks.len())?)?;
    for block in building.blocks.iter() {
        write_block(&mut w, &block, &building)?;
    }

    Ok(())
}

fn write_root<W: Write>(
    mut w: W,
    root: &SerializableRoot,
    building: &SerializableBuilding,
) -> Result<()> {
    w.write_array_f32::<LE>(&root.position.get())?;
    w.write_array_f32::<LE>(&root.rotation.get())?;

    let (center, size) = root.bounds.get_center_and_size();
    w.write_array_f32::<LE>(&center)?;
    w.write_array_f32::<LE>(&size)?;

    w.write_u16::<LE>(root.last_block_index)?;

    Ok(())
}

fn write_block<W: Write>(
    mut w: W,
    block: &SerializableBlock,
    building: &SerializableBuilding,
) -> Result<()> {
    w.write_array_f32::<LE>(&block.position.get())?;

    if !building.rotations.is_empty() {
        if building.rotations.len() <= 0xFF {
            w.write_u8(u8::try_from(block.rotation_index)?)?;
        } else {
            w.write_u16::<LE>(block.rotation_index)?;
        }
    } else {
        w.write_array_u16::<LE>(&pack_rotation(block.rotation.get()))?;
    }

    w.write_u8(block.id.get())?;

    // w.write_u8(block.root_id)?;

    let esc_is_high = block.enable_state_current.get() > 1.0f32;
    let esc_is_zero = block.enable_state_current.get() == 0.0f32;

    let flags = 
    // ---------------------------------------------------- //
        (block.name.borrow().is_some()                as u8) << 0 |
        ((block.connections.borrow().len() > 0)       as u8) << 1 |
        (block.metadata.borrow().is_none()            as u8) << 2 |
        (block.color.get().is_none()                  as u8) << 3 |
        (block.load.borrow()                          as u8) << 4 |
        (true                                         as u8) << 5 |
        ((block.enable_state_current.get() >  1.0f32) as u8) << 6 |
        ((block.enable_state_current.get() == 0.0f32) as u8) << 7 ;
    // ---------------------------------------------------- //

    Ok(())
}
