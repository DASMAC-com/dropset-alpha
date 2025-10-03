use crate::{
    error::DropsetError,
    state::{
        free_stack::Stack,
        linked_list::LinkedList,
        market_header::{MarketHeader, MARKET_HEADER_SIZE},
        sector::SECTOR_SIZE,
        transmutable::{load_unchecked, load_unchecked_mut},
    },
};

pub struct Market<Header, SectorBytes> {
    pub header: Header,
    pub sectors: SectorBytes,
}

pub type MarketRef<'a> = Market<&'a MarketHeader, &'a [u8]>;
pub type MarketRefMut<'a> = Market<&'a mut MarketHeader, &'a mut [u8]>;

impl AsRef<MarketHeader> for &MarketHeader {
    fn as_ref(&self) -> &MarketHeader {
        self
    }
}

impl AsMut<MarketHeader> for &mut MarketHeader {
    fn as_mut(&mut self) -> &mut MarketHeader {
        self
    }
}

impl<'a> MarketRef<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, DropsetError> {
        let (header_bytes, sectors) = data
            .split_at_checked(MARKET_HEADER_SIZE)
            .ok_or(DropsetError::InsufficientByteLength)?;

        // Safety: `split_at_*` ensures `header_bytes == MarketHeader::LEN`, and MarketHeaders are
        // valid (no undefined behavior) for all bit patterns.
        let header = unsafe { load_unchecked::<MarketHeader>(header_bytes) };
        Ok(Self { header, sectors })
    }
}

impl<'a> MarketRefMut<'a> {
    /// Verifies the account discriminant in the MarketHeader and that the sector bytes match
    /// the amount specified in the header.
    pub fn from_bytes_mut(data: &'a mut [u8]) -> Result<Self, DropsetError> {
        let (header_bytes, sectors) = data
            .split_at_mut_checked(MARKET_HEADER_SIZE)
            .ok_or(DropsetError::InsufficientByteLength)?;

        // Safety:
        // - `split_at_*` ensures `header_bytes == MarketHeader::LEN`.
        // - MarketHeaders are valid (no undefined behavior) for all bit patterns.
        let header = unsafe { load_unchecked_mut::<MarketHeader>(header_bytes) };

        if sectors.len() & SECTOR_SIZE != 0 {
            return Err(DropsetError::MismatchedDataLengths);
        }

        header.verify_discriminant()?;
        Ok(Self { header, sectors })
    }

    /// This function should only be called when `data` represents well-formed Market data.
    /// That is, it should have been passed to the market initialization function at some point.
    pub fn from_bytes_mut_unchecked(data: &'a mut [u8]) -> Result<Self, DropsetError> {
        let (header_bytes, sectors) = data
            .split_at_mut_checked(MARKET_HEADER_SIZE)
            .ok_or(DropsetError::InsufficientByteLength)?;

        // Safety:
        // - `split_at_*` ensures `header_bytes == MarketHeader::LEN`.
        // - MarketHeaders are valid (no undefined behavior) for all bit patterns.
        let header = unsafe { load_unchecked_mut::<MarketHeader>(header_bytes) };
        Ok(Self { header, sectors })
    }

    #[inline(always)]
    pub fn free_stack(&mut self) -> Stack<'_> {
        Stack::new_from_parts(self.header.as_mut().free_stack_top_mut_ref(), self.sectors)
    }

    #[inline(always)]
    pub fn seat_list(&mut self) -> LinkedList<'_> {
        LinkedList::new_from_parts(self.header, self.sectors)
    }
}
