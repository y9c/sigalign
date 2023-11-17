use num_traits::PrimInt;

pub type Sequence<'a> = &'a [u8];
pub type OwnedSequence = Vec<u8>;
pub type Label<'a> = &'a str;
pub type OwnedLabel = String;

pub trait SequenceLength:
    PrimInt
    + std::ops::AddAssign
    + std::ops::SubAssign
{
    const ZERO: Self;
    const ONE: Self;
    fn as_u32(self) -> u32;
    fn from_u32(value: u32) -> Self;
    fn as_u64(self) -> u64;
    fn as_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
    fn as_i64(self) -> i64;
}

impl SequenceLength for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_u64(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    fn as_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_i64(self) -> i64 {
        self as _
    }
}
impl SequenceLength for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_u64(self) -> u64 {
        self as u64
    }
    #[inline(always)]
    fn as_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_i64(self) -> i64 {
        self as _
    }
}
