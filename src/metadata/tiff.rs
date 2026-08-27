//! Minimal, allocation-conscious TIFF/EXIF IFD reader.
//!
//! It is deliberately tolerant: a corrupt offset yields `None` for that field
//! instead of failing the whole parse, because real world files from real
//! world cameras are frequently slightly wrong.

use std::collections::BTreeMap;

use super::bytes::{Cursor, Endian};
use super::value::{FieldType, Value};

/// Payloads larger than this are not decoded eagerly. Their location is still
/// recorded so binary blobs (ICC profiles, embedded previews, maker notes) can
/// be sliced on demand without copying them into every parsed image.
const MAX_EAGER_VALUE_BYTES: usize = 4096;

/// How many IFDs we are willing to walk before assuming the file loops.
const MAX_IFDS: usize = 32;

const TIFF_MAGIC: u16 = 42;
const ENTRY_SIZE: usize = 12;

/// A single IFD field.
#[derive(Debug, Clone)]
pub struct Entry {
    pub field_type: FieldType,
    pub count: u32,
    /// Absolute offset of the payload within the TIFF block.
    pub offset: usize,
    value: Option<Value>,
}

impl Entry {
    /// The decoded value, absent for payloads left undecoded because of their
    /// size. Use [`Entry::offset`] and [`Entry::byte_len`] for those.
    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// Size of the payload in bytes.
    pub fn byte_len(&self) -> usize {
        self.field_type.size().saturating_mul(self.count as usize)
    }
}

/// One image file directory: an ordered map from tag id to field.
#[derive(Debug, Clone, Default)]
pub struct Ifd {
    pub entries: BTreeMap<u16, Entry>,
    next: Option<u32>,
}

impl Ifd {
    pub fn entry(&self, tag: u16) -> Option<&Entry> {
        self.entries.get(&tag)
    }

    pub fn get(&self, tag: u16) -> Option<&Value> {
        self.entry(tag).and_then(Entry::value)
    }

    pub fn u32(&self, tag: u16) -> Option<u32> {
        self.get(tag).and_then(Value::as_u32)
    }

    /// All unsigned components of a tag, for offset lists such as `SubIFDs`.
    pub fn u32_list(&self, tag: u16) -> Vec<u32> {
        self.get(tag).map(Value::unsigned_list).unwrap_or_default()
    }
}

/// A parsed TIFF block: the header plus on-demand access to its directories.
pub struct Tiff<'a> {
    cursor: Cursor<'a>,
    first_ifd: u32,
}

impl<'a> Tiff<'a> {
    /// Parses the 8 byte TIFF header at the start of `data`.
    pub fn new(data: &'a [u8]) -> Option<Tiff<'a>> {
        let endian = match data.get(..2)? {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => return None,
        };

        let cursor = Cursor::new(data, endian);

        // Some raw formats (ORF, RW2) replace the magic with their own; accept
        // anything as long as the IFD offset is sane.
        let magic = cursor.u16(2)?;
        let first_ifd = cursor.u32(4)?;

        if magic != TIFF_MAGIC && !(0x4f52..=0x5352).contains(&magic) {
            tracing::trace!("Unexpected TIFF magic {magic:#x}, trying to parse anyway");
        }

        if first_ifd as usize >= data.len() {
            return None;
        }

        Some(Tiff { cursor, first_ifd })
    }

    /// The whole TIFF block, for slicing binary payloads out of it.
    pub fn bytes(&self, offset: usize, len: usize) -> Option<&'a [u8]> {
        self.cursor.slice(offset, len)
    }

    /// Payload bytes of an entry, valid also for entries too large to decode.
    pub fn entry_bytes(&self, entry: &Entry) -> Option<&'a [u8]> {
        self.bytes(entry.offset, entry.byte_len())
    }

    /// Reads the directory starting at `offset`.
    pub fn ifd_at(&self, offset: u32) -> Option<Ifd> {
        let base = offset as usize;
        let count = self.cursor.u16(base)? as usize;
        // A directory with thousands of entries is corrupt, not ambitious.
        if count == 0 || count > 4096 {
            return None;
        }

        let mut entries = BTreeMap::new();
        for i in 0..count {
            let entry_at = base + 2 + i * ENTRY_SIZE;
            if let Some((tag, entry)) = self.read_entry(entry_at) {
                entries.insert(tag, entry);
            }
        }

        let next = self
            .cursor
            .u32(base + 2 + count * ENTRY_SIZE)
            .filter(|n| *n != 0 && (*n as usize) < self.cursor.len());

        Some(Ifd { entries, next })
    }

    /// Walks the top level directory chain: IFD0, IFD1 (thumbnail), ...
    pub fn ifds(&self) -> Vec<Ifd> {
        let mut ifds = Vec::new();
        let mut next = Some(self.first_ifd);
        let mut seen = Vec::new();

        while let Some(offset) = next {
            if ifds.len() >= MAX_IFDS || seen.contains(&offset) {
                break;
            }
            seen.push(offset);

            match self.ifd_at(offset) {
                Some(ifd) => {
                    next = ifd.next;
                    ifds.push(ifd);
                }
                None => break,
            }
        }

        ifds
    }

    fn read_entry(&self, at: usize) -> Option<(u16, Entry)> {
        let tag = self.cursor.u16(at)?;
        let field_type = FieldType::from_code(self.cursor.u16(at + 2)?)?;
        let count = self.cursor.u32(at + 4)?;

        let byte_len = field_type.size().checked_mul(count as usize)?;
        // Inline payloads live in the entry itself when they fit in 4 bytes.
        let offset = if byte_len <= 4 {
            at + 8
        } else {
            self.cursor.u32(at + 8)? as usize
        };

        if offset.checked_add(byte_len)? > self.cursor.len() {
            return None;
        }

        let value = if byte_len <= MAX_EAGER_VALUE_BYTES {
            Value::decode(&self.cursor, field_type, count as usize, offset)
        } else {
            None
        };

        Some((
            tag,
            Entry {
                field_type,
                count,
                offset,
                value,
            },
        ))
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// One IFD field to write: `(tag, field type, count, payload)`.
    pub type TestEntry = (u16, FieldType, u32, Vec<u8>);

    /// Size on disk of a directory holding `count` entries.
    fn directory_size(count: usize) -> usize {
        2 + count * ENTRY_SIZE + 4
    }

    /// Serialises one directory plus its heap, resolving payload offsets
    /// against `heap_at` (an absolute offset within the block).
    fn write_ifd(entries: &[TestEntry], heap_at: usize) -> Vec<u8> {
        let mut directory = (entries.len() as u16).to_le_bytes().to_vec();
        let mut heap = Vec::new();

        for (tag, field_type, count, payload) in entries {
            directory.extend_from_slice(&tag.to_le_bytes());
            directory.extend_from_slice(&(*field_type as u16 + 1).to_le_bytes());
            directory.extend_from_slice(&count.to_le_bytes());

            if payload.len() <= 4 {
                let mut inline = payload.clone();
                inline.resize(4, 0);
                directory.extend_from_slice(&inline);
            } else {
                directory.extend_from_slice(&((heap_at + heap.len()) as u32).to_le_bytes());
                heap.extend_from_slice(payload);
            }
        }

        directory.extend_from_slice(&0u32.to_le_bytes());
        directory.extend_from_slice(&heap);
        directory
    }

    fn header() -> Vec<u8> {
        let mut out = b"II".to_vec();
        out.extend_from_slice(&TIFF_MAGIC.to_le_bytes());
        out.extend_from_slice(&8u32.to_le_bytes());
        out
    }

    /// Builds a little endian TIFF block with a single IFD.
    pub fn build_tiff(entries: &[TestEntry]) -> Vec<u8> {
        let mut out = header();
        out.extend_from_slice(&write_ifd(entries, 8 + directory_size(entries.len())));
        out
    }

    /// Builds a TIFF block whose IFD0 points at a sub-directory through
    /// `pointer_tag`, the way real EXIF and GPS directories are linked.
    pub fn build_tiff_with_sub_ifd(
        root: &[TestEntry],
        pointer_tag: u16,
        sub: &[TestEntry],
    ) -> Vec<u8> {
        let root_count = root.len() + 1;
        let root_heap_at = 8 + directory_size(root_count);
        let root_heap_len: usize = root.iter().map(|e| e.3.len()).filter(|l| *l > 4).sum();

        let sub_at = root_heap_at + root_heap_len;
        let sub_heap_at = sub_at + directory_size(sub.len());

        let mut root_entries = root.to_vec();
        root_entries.push((
            pointer_tag,
            FieldType::Long,
            1,
            (sub_at as u32).to_le_bytes().to_vec(),
        ));
        root_entries.sort_by_key(|e| e.0);

        let mut out = header();
        out.extend_from_slice(&write_ifd(&root_entries, root_heap_at));
        out.extend_from_slice(&write_ifd(sub, sub_heap_at));
        out
    }

    /// A `Rational` payload.
    pub fn rational(numerator: u32, denominator: u32) -> Vec<u8> {
        let mut out = numerator.to_le_bytes().to_vec();
        out.extend_from_slice(&denominator.to_le_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::build_tiff;
    use super::*;

    #[test]
    fn reads_inline_and_heap_values() {
        let data = build_tiff(&[
            (0x0112, FieldType::Short, 1, 6u16.to_le_bytes().to_vec()),
            (0x010F, FieldType::Ascii, 6, b"Canon\0".to_vec()),
        ]);

        let tiff = Tiff::new(&data).unwrap();
        let ifds = tiff.ifds();
        assert_eq!(ifds.len(), 1);
        assert_eq!(ifds[0].u32(0x0112), Some(6));
        assert_eq!(ifds[0].get(0x010F).and_then(Value::as_str), Some("Canon"));
    }

    #[test]
    fn rejects_non_tiff_data() {
        assert!(Tiff::new(b"not a tiff at all").is_none());
        assert!(Tiff::new(b"II").is_none());
    }

    #[test]
    fn rejects_out_of_range_offsets() {
        let mut data = build_tiff(&[(0x010F, FieldType::Ascii, 6, b"Canon\0".to_vec())]);
        // Point the single entry's payload past the end of the block.
        let offset_pos = 8 + 2 + 8;
        data[offset_pos..offset_pos + 4].copy_from_slice(&0xFFFF_0000u32.to_le_bytes());

        let tiff = Tiff::new(&data).unwrap();
        assert!(tiff.ifds()[0].get(0x010F).is_none());
    }

    #[test]
    fn large_payloads_stay_undecoded_but_locatable() {
        let blob = vec![0xABu8; MAX_EAGER_VALUE_BYTES + 16];
        let data = build_tiff(&[(
            0x8773,
            FieldType::Undefined,
            blob.len() as u32,
            blob.clone(),
        )]);

        let tiff = Tiff::new(&data).unwrap();
        let ifd = &tiff.ifds()[0];
        let entry = ifd.entry(0x8773).unwrap();

        assert!(entry.value().is_none());
        assert_eq!(entry.byte_len(), blob.len());
        assert_eq!(tiff.entry_bytes(entry), Some(blob.as_slice()));
    }

    #[test]
    fn big_endian_blocks_parse() {
        let mut data = vec![b'M', b'M', 0, 42, 0, 0, 0, 8];
        data.extend_from_slice(&1u16.to_be_bytes());
        data.extend_from_slice(&0x0112u16.to_be_bytes());
        data.extend_from_slice(&3u16.to_be_bytes());
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&[0, 8, 0, 0]);
        data.extend_from_slice(&0u32.to_be_bytes());

        let tiff = Tiff::new(&data).unwrap();
        assert_eq!(tiff.ifds()[0].u32(0x0112), Some(8));
    }
}
