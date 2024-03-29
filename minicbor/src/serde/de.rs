use serde::forward_to_deserialize_any;
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};

use crate::BREAK;
use crate::data::Type;
use crate::decode::{Decoder, Error};

pub struct Deserializer<'de> {
    decoder: Decoder<'de>
}

impl<'de> Deserializer<'de> {
    fn new(d: Decoder<'de>) -> Self {
        Self { decoder: d }
    }

    pub fn from_slice(b: &'de [u8]) -> Self {
        Self::new(Decoder::new(b))
    }

    pub fn decoder(&self) -> &Decoder<'de> {
        &self.decoder
    }

    pub fn decoder_mut(&mut self) -> &mut Decoder<'de> {
        &mut self.decoder
    }

    pub fn into_decoder(self) -> Decoder<'de> {
        self.decoder
    }
}

impl<'de> From<Decoder<'de>> for Deserializer<'de> {
    fn from(d: Decoder<'de>) -> Self {
        Self::new(d)
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.decoder.datatype()? {
            Type::Bool       => self.deserialize_bool(visitor),
            Type::U8         => self.deserialize_u8(visitor),
            Type::U16        => self.deserialize_u16(visitor),
            Type::U32        => self.deserialize_u32(visitor),
            Type::U64        => self.deserialize_u64(visitor),
            Type::I8         => self.deserialize_i8(visitor),
            Type::I16        => self.deserialize_i16(visitor),
            Type::I32        => self.deserialize_i32(visitor),
            Type::I64        => self.deserialize_i64(visitor),
            Type::F32        => self.deserialize_f32(visitor),
            Type::F64        => self.deserialize_f64(visitor),
            Type::Bytes      => visitor.visit_borrowed_bytes(self.decoder.bytes()?),
            Type::String     => visitor.visit_borrowed_str(self.decoder.str()?),
            Type::Null       => {
                self.decoder.skip()?;
                visitor.visit_none()
            }
            Type::BytesIndef => {
                let mut buf = Vec::new();
                for b in self.decoder.bytes_iter()? {
                    buf.extend_from_slice(b?)
                }
                visitor.visit_byte_buf(buf)
            }
            Type::StringIndef => {
                let mut buf = String::new();
                for b in self.decoder.str_iter()? {
                    buf += b?
                }
                visitor.visit_string(buf)
            }
            Type::Array | Type::ArrayIndef => self.deserialize_seq(visitor),
            Type::Map   | Type::MapIndef   => self.deserialize_map(visitor),
            t => Err(Error::type_mismatch(t).with_message("unexpected type").at(self.decoder.position()))
        }
    }

    forward_to_deserialize_any!(str string bytes byte_buf identifier);

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.decoder.bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i8(self.decoder.i8()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i16(self.decoder.i16()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.decoder.i32()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i64(self.decoder.i64()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.decoder.u8()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.decoder.u16()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.decoder.u32()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.decoder.u64()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.decoder.f32()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(self.decoder.f64()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_char(self.decoder.char()?)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if Type::Null == self.decoder.datatype()? {
            self.decoder.skip()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.decoder.decode::<()>()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, v: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_unit(v)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, v: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        v.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = self.decoder.array()?;
        visitor.visit_seq(Seq::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let p = self.decoder.position();
        let n = self.decoder.array()?;
        if Some(len as u64) != n {
            return Err(Error::message("invalid tuple-length {n:?}, was expecting: {len}").at(p))
        }
        visitor.visit_seq(Seq::new(self, n))
    }

    fn deserialize_tuple_struct<V>
        ( self
        , _name: &'static str
        , len: usize
        , visitor: V
        ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = self.decoder.map()?;
        visitor.visit_map(Seq::new(self, len))
    }

    fn deserialize_struct<V>
        ( self
        , _name: &'static str
        , _fields: &'static [&'static str]
        , visitor: V
        ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>
        ( self
        , _name: &'static str
        , _variants: &'static [&'static str]
        , visitor: V
        ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        let p = self.decoder.position();
        if Type::Map == self.decoder.datatype()? {
            let m = self.decoder.map()?;
            if m != Some(1) {
                return Err(Error::message("invalid enum map length").at(p))
            }
        }
        visitor.visit_enum(Enum::new(self))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.decoder.skip()?;
        visitor.visit_unit() // is ignored
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Seq<'a, 'de> {
    deserializer: &'a mut Deserializer<'de>,
    len: Option<u64>
}

impl<'a, 'de> Seq<'a, 'de> {
    fn new(d: &'a mut Deserializer<'de>, len: Option<u64>) -> Self {
        Self { deserializer: d, len }
    }
}

impl<'a, 'de> SeqAccess<'de> for Seq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>
    {
        match self.len {
            None => if BREAK == self.deserializer.decoder.current()? {
                self.deserializer.decoder.read()?;
                Ok(None)
            } else {
                seed.deserialize(&mut *self.deserializer).map(Some)
            }
            Some(0) => Ok(None),
            Some(n) => {
                let x = seed.deserialize(&mut *self.deserializer)?;
                self.len = Some(n - 1);
                Ok(Some(x))
            }
        }
    }
}

impl<'a, 'de> MapAccess<'de> for Seq<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>
    {
        match self.len {
            None => if BREAK == self.deserializer.decoder.current()? {
                self.deserializer.decoder.read()?;
                Ok(None)
            } else {
                seed.deserialize(&mut *self.deserializer).map(Some)
            }
            Some(0) => Ok(None),
            Some(_) => seed.deserialize(&mut *self.deserializer).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>
    {
        if let Some(n) = self.len {
            let x = seed.deserialize(&mut *self.deserializer)?;
            self.len = Some(n - 1);
            Ok(x)
        } else {
            seed.deserialize(&mut *self.deserializer)
        }
    }
}

struct Enum<'a, 'de: 'a> {
    deserializer: &'a mut Deserializer<'de>
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(d: &'a mut Deserializer<'de>) -> Self {
        Self { deserializer: d }
    }
}

impl<'a, 'de> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.deserializer).map(|v| (v, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(self, len: usize, v: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        de::Deserializer::deserialize_tuple(self.deserializer, len, v)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], v: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        de::Deserializer::deserialize_map(self.deserializer, v)
    }
}

