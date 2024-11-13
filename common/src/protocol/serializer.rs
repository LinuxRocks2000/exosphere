// serializers for the exosphere protocol
use serde::{ ser, Serialize };
use super::{ Error, Result, Result2 };


struct LengthGrabber { // a tiny serializer that does nothing but calculate the length of the final frame
    // once the length is known, a single allocation will be performed to hold the frame, which is an acceptable cost
    len : usize
}


pub trait ProtocolSerialize {
    fn get_encoded_size(&self) -> Result2<usize>;

    fn encode(&self) -> Result2<Vec<u8>>;
}


impl<T> ProtocolSerialize for T where T : serde::Serialize {
    fn get_encoded_size(&self) -> Result2<usize> {
        let mut grabber = LengthGrabber { len : 0 };
        self.serialize(&mut grabber)?;
        Ok(grabber.len)
    }

    fn encode(&self) -> Result2<Vec<u8>> {
        let len = self.get_encoded_size()?;
        let mut serializer = Serializer {
            buffer : Vec::with_capacity(len)
        };
        self.serialize(&mut serializer)?;
        Ok(serializer.buffer)
    }
}


impl<'a> ser::Serializer for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v : bool) -> Result {
        self.len += 1;
        Ok(())
    }

    fn serialize_none(self) -> Result {
        self.len += 1;
        Ok(())
    }

    fn serialize_some<T : ?Sized + Serialize>(self, value: &T) -> Result {
        self.len += 1;
        value.serialize(self)
    }

    fn serialize_i8(self, _v : i8) -> Result {
        self.len += 1;
        Ok(())
    }

    fn serialize_i16(self, _v : i16) -> Result {
        self.len += 2;
        Ok(())
    }

    fn serialize_i32(self, _v : i32) -> Result {
        self.len += 4;
        Ok(())
    }

    fn serialize_i64(self, _v : i64) -> Result {
        self.len += 8;
        Ok(())
    }

    fn serialize_u8(self, _v : u8) -> Result {
        self.len += 1;
        Ok(())
    }

    fn serialize_u16(self, _v : u16) -> Result {
        self.len += 2;
        Ok(())
    }

    fn serialize_u32(self, _v : u32) -> Result {
        self.len += 4;
        Ok(())
    }

    fn serialize_u64(self, _v : u64) -> Result {
        self.len += 8;
        Ok(())
    }

    fn serialize_f32(self, _v : f32) -> Result {
        self.len += 4;
        Ok(())
    }

    fn serialize_f64(self, _v : f64) -> Result {
        self.len += 8;
        Ok(())
    }

    fn serialize_char(self, _v : char) -> Result {
        self.len += 4;
        Ok(())
    }

    fn serialize_str(self, v : &str) -> Result {
        self.len += 4 + v.len(); // a 32-bit size before the string content
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result {
        self.len += 4 + v.len();
        Ok(())
    }

    fn serialize_unit(self) -> Result {
        Ok(())
    }

    fn serialize_unit_struct(self, _name : &'static str) -> Result {
        Ok(())
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index : u32, _variant : &'static str) -> Result {
        self.len += 4; // variant index
        Ok(())
    }

    fn serialize_newtype_struct<T : ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result
    where
        T: ?Sized + Serialize,
    {
        self.len += 4;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result2<Self::SerializeSeq> {
        self.len += 4; // length of the length field
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result2<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result2<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeTupleVariant> {
        self.len += 4;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result2<Self::SerializeMap> {
        self.len += 4; // the size of the field containing the number of KV pairs
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeStruct> {
        Ok(self) // essentially a no-op: we expect that, on the other side, the layout is known
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeStructVariant> {
        self.len += 4;
        Ok(self)
    }
}


impl<'a> ser::SerializeSeq for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}


impl<'a> ser::SerializeTuple for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut LengthGrabber {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}


// THE ACTUAL SERIALIZER


struct Serializer {
    buffer : Vec<u8>
}


impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v : bool) -> Result {
        self.buffer.push(if v { 1 } else { 0 });
        Ok(())
    }

    fn serialize_none(self) -> Result {
        self.buffer.push(0);
        Ok(())
    }

    fn serialize_some<T : ?Sized + Serialize>(self, value: &T) -> Result {
        self.buffer.push(1);
        value.serialize(self)
    }

    fn serialize_i8(self, v : i8) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_i16(self, v : i16) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_i32(self, v : i32) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_i64(self, v : i64) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_u8(self, v : u8) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_u16(self, v : u16) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_u32(self, v : u32) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_u64(self, v : u64) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_f32(self, v : f32) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_f64(self, v : f64) -> Result {
        for byte in v.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_char(self, v : char) -> Result {
        let point = self.buffer.len() - 1;
        for _ in 0..4 { // TODO: see if there's a better way to handle this (there probably is)
            self.buffer.push(0);
        }
        v.encode_utf8(&mut self.buffer[point..]);
        Ok(())
    }

    fn serialize_str(self, v : &str) -> Result {
        for byte in (v.len() as u32).to_le_bytes() {
            self.buffer.push(byte);
        }
        for byte in v.as_bytes() {
            self.buffer.push(*byte);
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result {
        for byte in (v.len() as u32).to_le_bytes() {
            self.buffer.push(byte);
        }
        for byte in v {
            self.buffer.push(*byte);
        }
        Ok(())
    }

    fn serialize_unit(self) -> Result {
        Ok(())
    }

    fn serialize_unit_struct(self, _name : &'static str) -> Result {
        Ok(())
    }

    fn serialize_unit_variant(self, _name: &'static str, variant_index : u32, _variant : &'static str) -> Result {
        for byte in variant_index.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(())
    }

    fn serialize_newtype_struct<T : ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result
    where
        T: ?Sized + Serialize,
    {
        for byte in variant_index.to_le_bytes() {
            self.buffer.push(byte);
        }
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result2<Self::SerializeSeq> {
        if let Some(len) = len {
            for byte in (len as u32).to_le_bytes() { // while the conversion from usize to u32 *can* panic, it's quite unlikely to happen
                self.buffer.push(byte);
            }
        }
        else {
            return Err(Error::UnsizedSequence)
        }
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result2<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeTupleVariant> {
        for byte in variant_index.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result2<Self::SerializeMap> {
        if let Some(len) = len {
            for byte in (len as u32).to_le_bytes() {
                self.buffer.push(byte);
            }
        }
        else {
            return Err(Error::UnsizedMap)
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeStruct> {
        Ok(self) // essentially a no-op: we expect that, on the other side, the layout is known
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result2<Self::SerializeStructVariant> {
        for byte in variant_index.to_le_bytes() {
            self.buffer.push(byte);
        }
        Ok(self)
    }
}


impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}


impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result {
        Ok(())
    }
}