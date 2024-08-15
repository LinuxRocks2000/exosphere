use crate::protocol::DecodeError;

// root of a protocol (this is what you derive)
pub trait ProtocolRoot : Protocol {
    fn report_tag(&self) -> u16; // get the outer tag for this frame

    fn encode(&self) -> Vec<u8> { // convenient wrapper for encode_into
        let mut result = vec![0u8; self.size()];
        self.encode_into(&mut result);
        result
    }
}

// protocol serializable thing
pub trait Protocol {
    fn size(&self) -> usize; // get the size of this protocol entry

    fn encode_into(&self, data : &mut [u8]); // write this protocol entry to an &mut [u8] buffer (guaranteed to be at least as large as size() reports)
    // encoding can't fail; if data isn't encodable, it'll be caught by the compiler
    // the only imaginable error is an allocation failure, which will just explode nonetheless

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> where Self : Sized; // data may be any size; it's up to the decoder to verify that the data it contains is sufficiently long and is valid
}
