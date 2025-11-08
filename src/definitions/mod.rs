pub mod structs;

use once_cell::sync::Lazy;
use serde_cbor::from_slice;
use structs::*;
use indexmap::IndexMap;


#[cfg(not(any(rust_analyzer, doc)))]
static BLOCKS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

#[cfg(any(rust_analyzer, doc))]
static BLOCKS: &[u8] = &[]; // = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

static BLOCK_DEFINITIONS_CBOR: Lazy<BlockDefinitionsFile> = Lazy::new(|| {
    from_slice(BLOCKS).expect("Failed to parse definitions.cbor")
});

pub static BLOCK_NAMES_IDS_MAP: Lazy<IndexMap<&str, u8>> = Lazy::new(|| {
    let mut map: IndexMap<&str, u8> = IndexMap::new();
    for block in BLOCK_DEFINITIONS_CBOR.blocks.iter() {
        map.insert(&block.name, block.id);
    }
    map
});