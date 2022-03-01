use std::{fmt::Debug, io::Write};

#[derive(Debug, Default)]
pub struct MessageWriter {
    data: Vec<u8>,
}

macro_rules! write_primitive {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self, value: $type) {
            self.data.write_all(&value.to_le_bytes()).unwrap();
        }
    };
}

impl MessageWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<S: Serialize>(&mut self, value: S) {
        value.serialize(self);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }

    write_primitive!(write_u8, u8);
    write_primitive!(write_i8, i8);
    write_primitive!(write_u16, u16);
    write_primitive!(write_i16, i16);
    write_primitive!(write_u32, u32);
    write_primitive!(write_i32, i32);
    write_primitive!(write_u64, u64);
    write_primitive!(write_i64, i64);
    write_primitive!(write_f32, f32);
    write_primitive!(write_f64, f64);

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.data.write_all(value).unwrap();
    }

    pub fn write_string(&mut self, value: &str) {
        let str_data = value.as_bytes();
        self.write_i32(str_data.len() as i32);
        self.write_bytes(str_data);
    }

    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}

pub trait Serialize {
    fn serialize(&self, w: &mut MessageWriter);

    fn serialize_bytes(&self) -> Vec<u8> {
        let mut w = MessageWriter::new();
        self.serialize(&mut w);
        w.finish()
    }
}

// Implement for all references
impl<S: Serialize> Serialize for &'_ S {
    #[inline]
    fn serialize(&self, w: &mut MessageWriter) {
        (*self).serialize(w);
    }
}

// Implement for primitives
macro_rules! impl_primitive {
    ($func:ident, $type:ty) => {
        impl Serialize for $type {
            fn serialize(&self, w: &mut MessageWriter) {
                w.$func(*self);
            }
        }
    };
}

impl_primitive!(write_bool, bool);
impl_primitive!(write_u8, u8);
impl_primitive!(write_i8, i8);
impl_primitive!(write_u16, u16);
impl_primitive!(write_i16, i16);
impl_primitive!(write_u32, u32);
impl_primitive!(write_i32, i32);
impl_primitive!(write_u64, u64);
impl_primitive!(write_i64, i64);
impl_primitive!(write_f32, f32);
impl_primitive!(write_f64, f64);
impl_primitive!(write_string, &str);
impl_primitive!(write_string, &String);

impl<S: Serialize> Serialize for Vec<S> {
    fn serialize(&self, w: &mut MessageWriter) {
        w.write(self.len() as i32);
        for v in self.iter() {
            w.write(v);
        }
    }
}

impl<S: Serialize> Serialize for Option<S> {
    fn serialize(&self, w: &mut MessageWriter) {
        w.write(self.is_some());
        if self.is_some() {
            w.write(self.as_ref().unwrap());
        }
    }
}
