pub mod structs;

use once_cell::sync::Lazy;
use serde_cbor::from_slice;
use structs::*;
use indexmap::IndexMap;

#[cfg(not(any(rust_analyzer, doc)))]
static BLOCKS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

#[cfg(any(rust_analyzer, doc))]
static BLOCKS: &[u8] = &[]; // = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

static FLAG_STRINGS: [&'static str; 3] = [
    "non_interactable",
    "tool",
    "custom_block"
];

pub fn flag(string: &str) -> u8 {
    let mut i = 0;
    while i < FLAG_STRINGS.len() {
        if FLAG_STRINGS[i] == string {
            return 1 << i;
        }
        i += 1;
    }
    panic!("No flag found.");
}

static BLOCK_DEFINITIONS_CBOR: Lazy<BlockDefinitionsFile> = Lazy::new(|| {
    from_slice(BLOCKS).expect("Failed to parse definitions.cbor")
});

pub(crate) static BLOCK_NAMES_IDS_MAP: Lazy<IndexMap<&str, u8>> = Lazy::new(|| {
    let mut map: IndexMap<&str, u8> = IndexMap::new();
    for block in BLOCK_DEFINITIONS_CBOR.blocks.iter() {
        map.insert(&block.name, block.id);
    }
    map
});

pub(crate) static BLOCK_FLAGS_VEC: Lazy<[u8; u8::MAX as usize]> = Lazy::new(|| {
    let mut array: [u8; u8::MAX as usize] = [0; u8::MAX as usize];
    for block in BLOCK_DEFINITIONS_CBOR.blocks.iter() {
        let compute_flag = |f: &str| -> u8 {
            if block.flags.contains(&f.to_string()) {
                flag(f)
            } else {
                0
            }
        };

        let mut flags: u8 = 0;
        
        for string in FLAG_STRINGS.iter() {
            flags |= compute_flag(string);
        }

        array[block.id as usize] = flags;
    }
    array 
});

pub fn get_flags(block_id: u8) -> u8 {
    BLOCK_FLAGS_VEC[block_id as usize]
}