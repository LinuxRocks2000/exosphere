// TODO: find every instance of `unwrap` in this file and replace it with proper error handling!


use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use super::{ Error, Result2 };

pub trait ProtocolDeserialize<'de> {
    fn decode(buf : &'de [u8]) -> Result2<Self> where Self: Sized;
}

impl<'de, T : Deserialize<'de> + Sized> ProtocolDeserialize<'de> for T {
    fn decode(buf : &'de [u8]) -> Result2<T> {
        let mut deserializer = Deserializer {
            buf,
            bufptr : 0
        };
        T::deserialize(&mut deserializer)
    }
}

struct Deserializer<'de> {
    buf : &'de [u8],
    bufptr : usize
}


impl<'de> Deserializer<'de> {
    fn next(&mut self) -> u8 {
        self.bufptr += 1;
        self.buf[self.bufptr - 1]
    }

    fn slice(&mut self, amount : usize) -> &'de [u8] {
        self.bufptr += amount;
        let sl = &self.buf[(self.bufptr - amount)..self.bufptr];
        sl
    }
}


impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_bool(self.next() == 1)
    }

    fn deserialize_i8<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_i8(i8::from_le_bytes(self.slice(1).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_i16<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_i16(i16::from_le_bytes(self.slice(2).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_i32<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_i32(i32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_i64<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_i64(i64::from_le_bytes(self.slice(8).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_u8<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_u8(u8::from_le_bytes(self.slice(1).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_u16<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_u16(u16::from_le_bytes(self.slice(2).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_u32<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_u32(u32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_u64<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_u64(u64::from_le_bytes(self.slice(8).try_into().map_err(|_| Error::BadInteger)?))
    }

    fn deserialize_f32<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_f32(f32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadFloat)?))
    }

    fn deserialize_f64<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_f64(f64::from_le_bytes(self.slice(8).try_into().map_err(|_| Error::BadFloat)?))
    }

    fn deserialize_char<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_char(char::from_u32(u32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?)).ok_or(Error::BadString)?)
    }

    fn deserialize_str<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        let sl = self.slice(4).try_into().map_err(|_| Error::BadInteger)?;
        let len = u32::from_le_bytes(sl);
        visitor.visit_borrowed_str(std::str::from_utf8(self.slice(len as usize)).map_err(|_| Error::BadString)?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        let len = u32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?);
        visitor.visit_bytes(self.slice(len as usize))
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        if self.next() == 1 {
            visitor.visit_some(self)
        }
        else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        visitor.visit_unit() // no-op: we don't encode type data in the packets, so unit structs simply don't exist
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name : &'static str, visitor: V) -> Result2<V::Value> {
        visitor.visit_unit() // see above
    }

    fn deserialize_newtype_struct<V : Visitor<'de>>(self, _name : &'static str, visitor: V) -> Result2<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V : Visitor<'de>>(self, visitor: V) -> Result2<V::Value> {
        let len = u32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?);
        visitor.visit_seq(SequenceParse { de : self, len : len as usize }) // note that the conversion from u32 to usize cannot panic because usize is (at least, on supported platforms) always a u32 or a u64
    }

    fn deserialize_tuple<V : Visitor<'de>>(self, len : usize, visitor : V) -> Result2<V::Value> {
        visitor.visit_seq(SequenceParse { de : self, len })
    }

    fn deserialize_tuple_struct<V : Visitor<'de>>(self, _name : &'static str, len : usize, visitor : V) -> Result2<V::Value> {
        visitor.visit_seq(SequenceParse { de : self, len })
    }

    fn deserialize_map<V : Visitor<'de>>(self, visitor : V) -> Result2<V::Value> {
        let len = u32::from_le_bytes(self.slice(4).try_into().map_err(|_| Error::BadInteger)?);
        visitor.visit_map(MapParse { de : self, len : len as usize })
    }

    fn deserialize_struct<V : Visitor<'de>>(self, _name : &'static str, fields : &'static [&'static str], visitor : V) -> Result2<V::Value> {
        visitor.visit_seq(SequenceParse { de : self, len : fields.len()})
    }

    fn deserialize_enum<V : Visitor<'de>>(self, _name : &'static str, _variants : &'static [&'static str], visitor : V) -> Result2<V::Value> {
        visitor.visit_enum(EnumParse { de : self })
    }

    fn deserialize_identifier<V : Visitor<'de>>(self, visitor : V) -> Result2<V::Value> {
        self.deserialize_u32(visitor)
    }

    fn deserialize_ignored_any<V : Visitor<'de>>(self, _visitor : V) -> Result2<V::Value> {
        Err(Error::InvalidType)
    }

    fn deserialize_any<V : Visitor<'de>>(self, _visitor : V) -> Result2<V::Value> {
        Err(Error::InvalidType)
    }
}


struct SequenceParse<'a, 'de> {
    de : &'a mut Deserializer<'de>,
    len : usize
}


impl<'a, 'de> SeqAccess<'de> for SequenceParse<'a, 'de> {
    type Error = Error;
    fn next_element_seed<T : DeserializeSeed<'de>>(&mut self, seed : T) -> Result2<Option<T::Value>> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}


struct MapParse<'a, 'de> {
    de : &'a mut Deserializer<'de>,
    len : usize
}


impl<'de, 'a> MapAccess<'de> for MapParse<'a, 'de> {
    type Error = Error;
    fn next_key_seed<K : DeserializeSeed<'de>>(&mut self, seed : K) -> Result2<Option<K::Value>> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<K : DeserializeSeed<'de>>(&mut self, seed : K) -> Result2<K::Value> {
        seed.deserialize(&mut *self.de)
    }
}


struct EnumParse<'a, 'de> {
    de : &'a mut Deserializer<'de>
}


impl<'de, 'a> EnumAccess<'de> for EnumParse<'a, 'de> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V : DeserializeSeed<'de>>(self, seed : V) -> Result2<(V::Value, Self::Variant)> {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for EnumParse<'a, 'de> {
    type Error = Error;
    fn unit_variant(self) -> Result2<()> {
        Ok(())
    }

    fn newtype_variant_seed<T : DeserializeSeed<'de>>(self, seed : T) -> Result2<T::Value> {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result2<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result2<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}