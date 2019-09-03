use byteorder::{ByteOrder, LittleEndian};
use std::fmt::Debug;


/// Used to inform assemblers on how to implement relocations for each architecture.
/// When implementing a new architecture, one simply has to implement this trait for
/// the architecture's relocation definition.
pub trait Relocation : Debug {
    /// The encoded representation for this relocation that is emitted by the dynasm! macro.
    type Encoding;
    /// construct this relocation from an encoded representation.
    fn from_encoding(encoding: Self::Encoding) -> Self;
    /// construct this relocation from a simple size. This is used to implement relocations in directives and literal pools.
    fn encode_from_size(size: RelocationSize) -> Self::Encoding;
    /// Returns the offset that this relocation is relative to, backwards with respect to the definition
    /// point of this relocation. (i.e. 0 for x64 as relocations are relative to the end of the instruction, and 4 for aarch64 as they are)
    /// Defaults to the size of this relocation.
    fn start_offset(&self) -> usize {
        self.size()
    }
    /// Returns the offset of the start of the bytes containing this relocation, backwards with respect to the definition point
    /// of this relocation.
    /// Defaults to the size of this relocation.
    fn field_offset(&self) -> usize {
        self.size()
    }
    /// The size of the slice of bytes affected by this relocation
    fn size(&self) -> usize;
    /// Write a value into a buffer of size `self.size()` in the format of this relocation.
    /// Any bits not part of the relocation should be preserved.
    fn write_value(&self, buf: &mut [u8], value: isize);
    /// Read a value from a buffer of size `self.size()` in the format of this relocation.
    fn read_value(&self, buf: &[u8]) -> isize;
    /// Specifies what kind of relocation this relocation instance is.
    fn kind(&self) -> RelocationKind;
    /// Specifies the default page size on this platform.
    fn page_size() -> usize;
}


/// Specifies what kind of relocation a relocation is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelocationKind {
    /// A simple, PC-relative relocation. These can be encoded once and do not need
    /// to be adjusted when the executable buffer is moved.
    Relative = 0,
    /// An absolute relocation to a relative address,
    /// i.e. trying to put the address of a dynasm x86 function in a register
    /// This means adjustment is necessary when the executable buffer is moved
    AbsToRel = 1,
    /// A relative relocation to an absolute address,
    /// i.e. trying to call a rust function with a dynasm x86 call.
    /// This means adjustment is necessary when the executable buffer is moved
    RelToAbs = 2,
}

impl RelocationKind {
    /// Converts back from numeric value to RelocationKind
    pub fn from_encoding(encoding: u8) -> Self {
        match encoding {
            0 => Self::Relative,
            1 => Self::AbsToRel,
            2 => Self::RelToAbs,
            x => panic!("Unsupported relocation kind {}", x)
        }
    }
}


/// A descriptor for the size of a relocation. This also doubles as a relocation itself
/// for relocations in data directives. Can be converted to relocations of any kind of architecture
/// using `Relocation::from_size`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelocationSize {
    /// A byte-sized relocation
    Byte = 1,
    /// A two-byte relocation
    Word = 2,
    /// A four-byte sized relocation
    DWord = 4,
    /// An 8-byte sized relocation
    QWord = 8,
}

impl Relocation for RelocationSize {
    type Encoding = u8;
    fn from_encoding(encoding: Self::Encoding) -> Self {
        match encoding {
            1 => RelocationSize::Byte,
            2 => RelocationSize::Word,
            4 => RelocationSize::DWord,
            8 => RelocationSize::QWord,
            x => panic!("Unsupported relocation size {}", x)
        }
    }
    fn encode_from_size(size: RelocationSize) -> Self::Encoding {
        size as u8
    }
    fn size(&self) -> usize {
        *self as usize
    }
    fn write_value(&self, buf: &mut [u8], value: isize) {
        match self {
            RelocationSize::Byte => buf[0] = value as u8,
            RelocationSize::Word => LittleEndian::write_i16(buf, value as i16),
            RelocationSize::DWord => LittleEndian::write_i32(buf, value as i32),
            RelocationSize::QWord => LittleEndian::write_i64(buf, value as i64),
        }
    }
    fn read_value(&self, buf: &[u8]) -> isize {
        match self {
            RelocationSize::Byte => buf[0] as i8 as isize,
            RelocationSize::Word => LittleEndian::read_i16(buf) as isize,
            RelocationSize::DWord => LittleEndian::read_i32(buf) as isize,
            RelocationSize::QWord => LittleEndian::read_i64(buf) as isize,
        }
    }
    fn kind(&self) -> RelocationKind {
        RelocationKind::Relative
    }
    fn page_size() -> usize {
        0
    }
}
