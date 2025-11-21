use byteorder::{WriteBytesExt, LE};
use num_traits::{FromPrimitive, PrimInt, ToBytes, Unsigned};
use std::io;
use std::io::{Error, ErrorKind, Read, Write};

use crate::structs::Gradient;

const ROTATION_MULTIPLIER: f32 = (u16::MAX as f32) / 360.0f32;
const ROTATION_INV: f32 = 360.0 / (u16::MAX as f32);

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub(crate) struct Bounds {
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
}

impl Bounds {
    pub(crate) const fn new() -> Self {
        Bounds {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    pub(crate) const fn from_center_and_size(center: [f32; 3], size: [f32; 3]) -> Self {
        let mut min = [0.0f32; 3];
        let mut max = [0.0f32; 3];

        let mut i = 0;
        while i < 3 {
            min[i] = center[i] - size[i] * 0.5;
            max[i] = center[i] + size[i] * 0.5;
            i += 1;
        }

        Self { min, max }
    }

    pub(crate) const fn get_center_and_size(&self) -> ([f32; 3], [f32; 3]) {
        let mut center = [0.0f32; 3];
        let mut size = [0.0f32; 3];

        let mut i = 0;
        while i < 3 {
            center[i] = (self.min[i] + self.max[i]) * 0.5;
            size[i] = self.max[i] - self.min[i];
            i += 1;
        }

        (center, size)
    }

    pub(crate) fn to_inbounds(&self, f: [f32; 3]) -> [i16; 3] {
        let (center, size) = self.get_center_and_size();

        let mut result = [0i16; 3];
        for i in 0..3 {
            let multiplier = (1.0f32 / size[i]) * i16::MAX as f32;
            result[i] = ((f[i] - center[i]) * multiplier).round() as i16
        }
        result
    }

    pub(crate) fn to_global(&self, v: [i16; 3]) -> [f32; 3] {
        let (center, size) = self.get_center_and_size();

        let mut result = [0.0f32; 3];
        for i in 0..3 {
            let multiplier = size[i] / i16::MAX as f32;
            result[i] = center[i] + v[i] as f32 * multiplier;
        }
        result
    }

    pub(crate) fn encapsulate(&mut self, block_position: &[f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(block_position[i]);
            self.max[i] = self.max[i].max(block_position[i]);
        }
    }
}

pub(crate) fn pack_rotation(data: [f32; 3]) -> [u16; 3] {
    let mut out = [0u16; 3];
    for (i, &angle) in data.iter().enumerate() {
        // Normalize angle into [0.0, 360.0)
        let mut a = angle % 360.0_f32;
        if a < 0.0 {
            a += 360.0_f32;
        }

        // Multiply and round to nearest. Use saturating cast to avoid overflow.
        let scaled = a * ROTATION_MULTIPLIER;
        // Clamp into [0.0, u16::MAX as f32] to be safe for extreme inputs
        let clamped = if scaled.is_finite() {
            scaled.max(0.0).min(u16::MAX as f32)
        } else {
            0.0
        };
        out[i] = clamped.round() as u16;
    }
    out
}

pub(crate) fn unpack_rotation(data: [u16; 3]) -> [f32; 3] {
    [
        (data[0] as f32) * ROTATION_INV,
        (data[1] as f32) * ROTATION_INV,
        (data[2] as f32) * ROTATION_INV,
    ]
}

pub(crate) fn pack_bools(bools: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((bools.len() + 7) / 8);
    for chunk in bools.chunks(8) {
        let mut byte = 0u8;
        for (i, &b) in chunk.iter().enumerate() {
            byte |= (b as u8) << i;
        }
        bytes.push(byte);
    }
    bytes
}

pub(crate) fn unpack_bools(bytes: &[u8], count: usize) -> Vec<bool> {
    let mut bools = Vec::with_capacity(count);
    for &byte in bytes.iter() {
        for bit in 0..8 {
            if bools.len() == count {
                return bools;
            }
            bools.push((byte >> bit) & 1 != 0);
        }
    }
    bools
}

pub(crate) fn read_7bit_encoded_int(mut r: impl Read) -> Result<usize> {
    let mut result: usize = 0;
    let mut bits_read: usize = 0;

    loop {
        let mut buf = [0u8];
        r.read_exact(&mut buf)?;
        let byte = buf[0];

        result |= ((byte & 0x7F) as usize) << bits_read;
        bits_read += 7;

        if bits_read > (usize::BITS as usize / 7) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Too many bytes when decoding 7-bit int.",
            )));
        }

        if (byte & 0x80) == 0 {
            break;
        }
    }

    Ok(result)
}

pub(crate) fn read_string_7bit<R: Read>(mut r: R) -> Result<String> {
    let len = read_7bit_encoded_int(&mut r)? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

pub(crate) fn pack_color([r, g, b]: [u8; 3]) -> u16 {
    ((r & 0xF8) as u16) << 8 | ((g & 0xFC) as u16) << 2 | ((b & 0xF8) as u16) >> 3
}

pub(crate) fn unpack_color(rgb565: u16) -> [u8; 3] {
    [
        ((rgb565 >> 8) & 0xF8) as u8,
        ((rgb565 >> 2) & 0xFC) as u8,
        ((rgb565 << 3) & 0xF8) as u8,
    ]
}

macro_rules! impl_write_array {
    ($func_name:ident, $elem_type:ty, $write_fn:ident) => {
        fn $func_name<E: byteorder::ByteOrder>(
            &mut self,
            array: &[$elem_type],
        ) -> std::io::Result<()> {
            for &v in array {
                self.$write_fn::<E>(v)?;
            }
            Ok(())
        }
    };
}

pub trait WriteUtils: Write {
    fn write_array<T: Copy>(
        &mut self,
        array: &[T],
        f: impl Fn(&mut Self, &T) -> Result<()>,
    ) -> Result<()> {
        for value in array.iter() {
            f(self, value)?;
        }
        Ok(())
    }

    fn write_7bit_encoded_int(&mut self, mut value: usize) -> Result<()> {
        while value >= 0x80 {
            self.write_all(&[((value as u8 & 0x7F) | 0x80)])?;
            value >>= 7;
        }
        self.write_all(&[value as u8])?;
        Ok(())
    }

    fn write_string_7bit(&mut self, s: &str) -> Result<()> {
        self.write_7bit_encoded_int(s.len())?;
        self.write_all(s.as_bytes())?;
        Ok(())
    }

    /// Writes array with length. Returns error if length is bigger than N max value.
    fn write_array_with_length<N: PrimInt + Unsigned + FromPrimitive, T: Copy>(
        &mut self,
        l: impl Fn(&mut Self, &N) -> Result<()>,
        f: impl Fn(&mut Self, &T) -> Result<()>,
        array: &[T],
    ) -> Result<()> {
        let len_n = N::from_usize(array.len())
            .ok_or_else(|| Error::new(ErrorKind::Other, "Array length too big for integer type"))?;
        l(self, &len_n)?;
        self.write_array(array, f)?;
        Ok(())
    }

    fn write_gradient(&mut self, gradient: &Gradient) -> Result<()> {
        self.write_u16::<LE>(u16::try_from(gradient.color_keys.len())?)?;
        for v in gradient.color_keys.iter() {
            self.write_array_f32::<LE>(v)?;
        }

        self.write_u16::<LE>(u16::try_from(gradient.color_time_keys.len())?)?;
        self.write_array_f32::<LE>(&gradient.color_time_keys)?;

        self.write_u16::<LE>(u16::try_from(gradient.alpha_keys.len())?)?;
        self.write_array_f32::<LE>(&gradient.alpha_keys)?;

        self.write_u16::<LE>(u16::try_from(gradient.alpha_time_keys.len())?)?;
        self.write_array_f32::<LE>(&gradient.alpha_time_keys)?;

        Ok(())
    }

    impl_write_array!(write_array_f32, f32, write_f32);
    impl_write_array!(write_array_u16, u16, write_u16);
    impl_write_array!(write_array_i16, i16, write_i16);
    impl_write_array!(write_array_i32, i32, write_i32);
    impl_write_array!(write_array_u32, u32, write_u32);
}

impl<W: Write + ?Sized> WriteUtils for W {}

pub trait ReadUtils: Read {
    fn read_array<T: Copy>(
        &mut self,
        len: usize,
        f: impl Fn(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut v: Vec<T> = Vec::new();
        v.reserve(len);
        for _ in 0..len {
            v.push(f(self)?);
        }
        Ok(v)
    }
    fn read_array_with_length<N: PrimInt + Unsigned + FromPrimitive, T: Copy>(
        &mut self,
        l: impl Fn(&mut Self) -> Result<N>,
        f: impl Fn(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let len_n = l(self)?;
        self.read_array(len_n.to_usize().unwrap(), f)
    }
}

impl<R: Read + ?Sized> ReadUtils for R {}

#[macro_export]
macro_rules! debug_val {
    ($val:expr) => {{
        #[cfg(debug_assertions)]
        {
            eprintln!("[debug] {} = {:?}", stringify!($val), &$val);
        }
    }};
}

#[test]
fn test_pack_unpack_roundtrip() {
    use rand::Rng;

    let mut rng = rand::rng();

    for _ in 0..1000 {
        let rgb: [u8; 3] = [rng.random(), rng.random(), rng.random()];
        let packed = pack_color(rgb);
        let unpacked = unpack_color(packed);
        let repacked = pack_color(unpacked);

        assert_eq!(
            packed, repacked,
            "Round-trip failed: original {:?}, packed {:#06x}, unpacked {:?}, repacked {:#06x}",
            rgb, packed, unpacked, repacked
        );
    }
}
