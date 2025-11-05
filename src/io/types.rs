use crate::{block::*, root::*};
use std::{cell::RefCell, rc::Rc};
use indexmap::{IndexMap};

pub(crate) struct BlockSerializationData {
    pub(crate) bid: Option<u16>,
    pub(crate) root: Option<u16>,
    pub(crate) color_id: Option<u16>,
    pub(crate) rotation_id: Option<u16>,
    pub(crate) packed_color: Option<u16>,
    pub(crate) packed_rotation: Option<[u16; 3]>
}

impl BlockSerializationData {
    pub(crate) fn new() -> Self {
        BlockSerializationData {
            bid: None,
            root: None,
            color_id: None,
            rotation_id: None,
            packed_color: None,
            packed_rotation: None
        }
    }
}

pub(crate) struct RootSerializationData {
    pub(crate) rid: Option<u16>
}

impl RootSerializationData {
    pub(crate) fn new() -> Self {
        RootSerializationData {
            rid: None
        }
    }
}

pub(crate) struct BuildingSerializationData {
    pub(crate) version: Option<u8>,
    
    pub(crate) blocks: RefCell<Vec<Rc<Block>>>,
    pub(crate) roots: RefCell<Vec<Rc<Root>>>,
    
    pub(crate) blocks_sdata: IndexMap<*const Block, BlockSerializationData>,
    pub(crate) roots_sdata: IndexMap<*const Root, RootSerializationData>
}

impl BuildingSerializationData {
    pub(crate) fn new() -> Self {
        BuildingSerializationData {
            version: None,
            blocks: RefCell::new(Vec::new()),
            roots: RefCell::new(Vec::new()),
            blocks_sdata: IndexMap::new(),
            roots_sdata: IndexMap::new()
        }
    }
}