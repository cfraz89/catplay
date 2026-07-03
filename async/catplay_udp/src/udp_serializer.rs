use super::AlignedOffsetBuf;

pub trait UdpSerializer<const DATAGRAM_SIZE: usize, const PAD: usize> {
    type ItemEncodable<'data>;
    type ItemDecodable<'data>;
    type Error;

    /// Allows applying mutable changes to the buffer stored and owned inside the item, such as encryption.
    fn encode(&mut self, item: Self::ItemEncodable<'_>, buf: &mut AlignedOffsetBuf<DATAGRAM_SIZE, PAD>) -> Result<(), Self::Error>;

    /// Allows decoding item while applying mutable changes to the buffer, such as decryption.
    ///
    /// The returned item is allowed to borrow the buffer(with a lifetime attached), and a seperate [ToOwned]-style utility can be added if payload needs to be kept past the call to `decode`.
    fn decode<'data>(&mut self, buf: &'data mut [u8]) -> Result<Self::ItemDecodable<'data>, Self::Error>;

    fn buffer_uninit() -> AlignedOffsetBuf<DATAGRAM_SIZE, PAD> {
        AlignedOffsetBuf::new()
    }
}
