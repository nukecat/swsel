use std::rc::Weak;
use std::cell::RefCell;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct Block {
    pub position: [f32; 3],
    pub rotation: [f32; 3],

    pub id: u8,

    pub metadata: Option<BlockMetadata>,

    pub name: Option<String>,
    pub enable_state: f32,
    pub enable_state_current: f32,

    pub connections: RefCell<Vec<Weak<Block>>>,

    pub load: Weak<Block>,

    pub color : Option<[u8; 3]>
}

impl Block {
    pub fn new() -> Self {
        Block {
            position: [0.0f32; 3],
            rotation: [0.0f32; 3],

            id: 0,

            metadata: None,

            name: None,
            enable_state: 0.0f32,
            enable_state_current: 0.0f32,

            connections: RefCell::new(Vec::new()),

            load: Weak::new(),

            color: None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorKey {
    time: f32,
    color: [f32; 4]
}

#[derive(Copy, Clone, Debug)]
pub struct AlphaKey {
    time: f32,
    alpha: f32
}

#[derive(Clone, Debug)]
pub struct Gradient {
    color_keys: Vec<ColorKey>,
    alpha_keys: Vec<AlphaKey>
}

#[derive(Clone, Debug)]
pub struct BlockMetadata {
    pub ticks: Vec<bool>,
    pub values: Vec<f32>,
    pub fields: RefCell<Vec<Vec<Weak<Block>>>>,
    pub colors: Vec<[f32; 4]>,
    pub gradients: Vec<Gradient>,
    pub vectors: Vec<[f32; 3]>
}