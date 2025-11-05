use std::io::{Write, Read};
use std::io;
use crate::{building::*, root::*, block::*, utils::*, io::types::*};
use indexmap::IndexMap;
use std::rc::*;
use byteorder::{WriteBytesExt, LittleEndian};

pub fn write_building<W: Write>(mut w: W, building: &Building, version: u8) -> io::Result<()> {
    let mut building_sdata = BuildingSerializationData::new();
    building_sdata.version = version;

    let [root_count, block_count] = building.count_roots_and_blocks();
    
    {
        let mut broots = building_sdata.roots.borrow_mut();
        broots.reserve(root_count);
        let mut bblocks = building_sdata.blocks.borrow_mut();
        bblocks.reserve(block_count);

        let mut current_bid: u16 = 0;
        let mut current_rid: u16 = 0;

        for root in building.roots.iter() {
            let mut root_sdata = RootSerializationData::new();
            root_sdata.rid = current_rid;
            building_sdata.roots_sdata.insert(Rc::as_ptr(root), root_sdata);
            broots.push(root.clone());
            current_rid
                .checked_add(1)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Too many roots, u16 index overflow!"))?;

            for block in root.blocks.iter() {
                let mut block_sdata = BlockSerializationData::new();
                block_sdata.bid = current_bid;
                building_sdata.blocks_sdata.insert(Rc::as_ptr(block), block_sdata);
                bblocks.push(block.clone());
                current_bid
                    .checked_add(1)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Too many blocks, u16 index overflow!"))?;
            }
        }
        if broots.len() != building_sdata.roots_sdata.len() {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Length of the vector with roots are not equal to the length of the roots seriarizable data map."
                )
            );
        }
        if bblocks.len() != building_sdata.blocks_sdata.len() {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Length of the vector with blocks are not equal to the length of the blocks seriarizable data map."
                )
            );
        }
    }

    w.write_u8(version)?;

    if version > 5 {
        let bblocks = building_sdata.blocks.borrow();

        let mut colors: IndexMap<u16, u8> = IndexMap::new();
        let mut rotations: IndexMap<[u16; 3], u16> = IndexMap::new();

        let mut colored_c: usize = 0;

        for block in bblocks.iter() {
            let block_data = building_sdata.blocks_sdata
                .get_mut(&Rc::as_ptr(block))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Block data not found."))?;
            
            let packed_rotation = pack_rotation(block.rotation);
            let rotations_len = (rotations.len() as u16);
            block_data.packed_rotation = packed_rotation;
            block_data.rotation_id = *rotations.entry(packed_rotation).or_insert(rotations_len);

            if let Some(color) = block.color {
                colored_c += 1;

                let colors_len = colors.len();
                let packed_color = pack_color(color);
                block_data.packed_color = packed_color;
                block_data.color_id = *colors.entry(packed_color).or_insert(colors_len as u8);
            }
        }

        let colors_len = colors.len();
        let avg_colors = colored_c as f32 / colors_len as f32;
        let rotations_len = rotations.len();
        let avg_rotations = block_count as f32 / rotations_len as f32;

        building_sdata.color_lookup = avg_colors > 2f32 && colors_len < u8::MAX as usize;
        building_sdata.single_byte_rotation = rotations_len <= usize::pow(2usize, u8::BITS);
        building_sdata.rotation_lookup = avg_rotations > (if building_sdata.single_byte_rotation {1.2f32} else {1.5f32}) && rotations_len < u16::MAX as usize;
        building_sdata.single_byte_rotation &= building_sdata.rotation_lookup;

        w.write_u8(if building_sdata.color_lookup {colors_len as u8} else {u8::MAX})?;
        w.write_u16::<LittleEndian>(if building_sdata.rotation_lookup {rotations_len as u16} else {u16::MAX})?;

        if building_sdata.color_lookup {
            for color in colors.iter() {
                w.write_u16::<LittleEndian>(*color.0)?;
            }
        }
        if building_sdata.rotation_lookup {
            for rotation in rotations.iter() {
                for value in rotation.0.iter() {
                    w.write_u16::<LittleEndian>(*value)?;
                }
            }
        }
    }

    w.write_u16::<LittleEndian>(root_count as u16)?;
    {
        let broots = building_sdata.roots.borrow().clone();
        for root in broots.iter() {
            write_root(&mut w, root, &mut building_sdata)?;
        }
    }

    w.write_u16::<LittleEndian>(block_count as u16)?;
    {
        let bblocks = building_sdata.blocks.borrow().clone();
        for block in bblocks.iter() {
            write_block(&mut w, block, &mut building_sdata)?;
        }
    }

    Ok(())
}

fn write_root<W: Write>(mut w: W, root: &Root, building_sdata: &mut BuildingSerializationData) -> io::Result<()> {
    Ok(())
}

fn write_block<W: Write>(mut w: W, block: &Block, building_sdata: &mut BuildingSerializationData) -> io::Result<()> {
    Ok(())
}