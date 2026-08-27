//! Reading primitives out of a byte slice, safely.
//!
//! Every offset in an EXIF block comes from the file, so it may be nonsense.
//! Nothing here indexes; a bad offset yields `None`.

/// Byte order of a TIFF block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl Endian {
    pub fn u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        }
    }

    pub fn u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        }
    }
}

/// A cursor over a byte slice that never panics and never reads out of bounds.
///
/// Every EXIF offset in the wild may be corrupt, so all reads are checked and
/// return `None` instead of indexing.
#[derive(Debug, Clone, Copy)]
pub struct Cursor<'a> {
    data: &'a [u8],
    endian: Endian,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], endian: Endian) -> Self {
        Self { data, endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn slice(&self, offset: usize, len: usize) -> Option<&'a [u8]> {
        self.data.get(offset..offset.checked_add(len)?)
    }

    pub fn u8(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    pub fn u16(&self, offset: usize) -> Option<u16> {
        let b = self.slice(offset, 2)?;
        Some(self.endian.u16([b[0], b[1]]))
    }

    pub fn u32(&self, offset: usize) -> Option<u32> {
        let b = self.slice(offset, 4)?;
        Some(self.endian.u32([b[0], b[1], b[2], b[3]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_both_endians() {
        assert_eq!(Endian::Little.u16([0x01, 0x02]), 0x0201);
        assert_eq!(Endian::Big.u16([0x01, 0x02]), 0x0102);
        assert_eq!(Endian::Little.u32([1, 0, 0, 0]), 1);
        assert_eq!(Endian::Big.u32([0, 0, 0, 1]), 1);
    }

    #[test]
    fn never_reads_out_of_bounds() {
        let cursor = Cursor::new(&[1, 2, 3], Endian::Little);

        assert_eq!(cursor.u16(2), None);
        assert_eq!(cursor.u32(0), None);
        assert_eq!(cursor.slice(2, 5), None);
        assert_eq!(cursor.u8(3), None);
    }

    #[test]
    fn reads_what_is_in_range() {
        let cursor = Cursor::new(&[1, 2, 3, 4, 5], Endian::Little);

        assert_eq!(cursor.u8(0), Some(1));
        assert_eq!(cursor.u16(0), Some(0x0201));
        assert_eq!(cursor.u32(0), Some(0x04030201));
        assert_eq!(cursor.slice(1, 2), Some(&[2u8, 3][..]));
        assert_eq!(cursor.len(), 5);
        assert!(!cursor.is_empty());
    }

    #[test]
    fn an_offset_that_would_overflow_is_rejected() {
        let cursor = Cursor::new(&[1, 2, 3], Endian::Little);
        assert_eq!(cursor.slice(usize::MAX, 1), None);
    }
}
