pub mod structs;

use once_cell::sync::Lazy;
use serde_cbor::from_slice;
use structs::*;


#[cfg(not(any(rust_analyzer, doc)))]
static BLOCKS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

#[cfg(any(rust_analyzer, doc))]
static BLOCKS: &[u8] = &[]; // = include_bytes!(concat!(env!("OUT_DIR"), "/definitions.cbor"));

static BLOCK_DEFINITIONS: Lazy<BlockDefinitionsFile> = Lazy::new(|| {
    from_slice(BLOCKS).expect("Failed to parse definitions.cbor")
});

pub fn get_block_definitions() -> &'static Vec<BlockDefinition> {
    &BLOCK_DEFINITIONS.blocks
}