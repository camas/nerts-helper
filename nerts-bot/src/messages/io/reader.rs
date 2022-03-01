use std::{fmt::Debug, io::Read};

// Deserialization and binary reading that mimics LiteNetLib

#[derive(Debug)]
pub struct MessageReader<'a> {
    data: &'a [u8],
}

macro_rules! read_primitive {
    ($name:ident, $type:ty, $size:expr) => {
        #[allow(dead_code)]
        pub fn $name(&mut self) -> $type {
            let mut buf = [0; $size];
            self.data.read_exact(&mut buf).unwrap();
            <$type>::from_le_bytes(buf)
        }
    };
}

#[allow(dead_code)]
impl<'a> MessageReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        MessageReader { data }
    }

    pub fn read<D: Deserialize>(&mut self) -> D {
        D::deserialize(self)
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    read_primitive!(read_u8, u8, 1);
    read_primitive!(read_i8, i8, 1);
    read_primitive!(read_u16, u16, 2);
    read_primitive!(read_i16, i16, 2);
    read_primitive!(read_u32, u32, 4);
    read_primitive!(read_i32, i32, 4);
    read_primitive!(read_u64, u64, 8);
    read_primitive!(read_i64, i64, 8);
    read_primitive!(read_f32, f32, 4);
    read_primitive!(read_f64, f64, 8);

    pub fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut result = vec![0; len];
        self.data.read_exact(&mut result).unwrap();
        result
    }

    pub fn read_string(&mut self) -> String {
        let len = self.read_i32();
        String::from_utf8(self.read_bytes(len as usize)).unwrap()
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        self.data.to_vec()
    }

    pub fn remaining_len(&self) -> usize {
        self.data.len()
    }
}

pub trait Deserialize {
    fn deserialize(r: &mut MessageReader) -> Self;
}

macro_rules! impl_primitive {
    ($func:ident, $type:ty) => {
        impl Deserialize for $type {
            fn deserialize(w: &mut MessageReader) -> Self {
                w.$func()
            }
        }
    };
}

impl_primitive!(read_bool, bool);
impl_primitive!(read_u8, u8);
impl_primitive!(read_i8, i8);
impl_primitive!(read_u16, u16);
impl_primitive!(read_i16, i16);
impl_primitive!(read_u32, u32);
impl_primitive!(read_i32, i32);
impl_primitive!(read_u64, u64);
impl_primitive!(read_i64, i64);
impl_primitive!(read_f32, f32);
impl_primitive!(read_f64, f64);
impl_primitive!(read_string, String);

impl<D: Deserialize> Deserialize for Option<D> {
    fn deserialize(r: &mut MessageReader) -> Self {
        if r.read_bool() {
            Some(D::deserialize(r))
        } else {
            None
        }
    }
}

impl<D: Deserialize> Deserialize for Vec<D> {
    fn deserialize(r: &mut MessageReader) -> Self {
        let len = r.read_i32();
        let mut result = Vec::with_capacity(len as usize);
        for _ in 0..len {
            result.push(D::deserialize(r));
        }
        result
    }
}
