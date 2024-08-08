//! module for encoding and decoding js value
fn debug_bin(num: u64) {
  println!("{num} -> {num:0x}");
}

// value type via nan box:
//
// ValEmpty  {  0000:0000:0000:0000
// Null      {  0000:0000:0000:0002
// Wasm      {  0000:0000:0000:0003
// ValDeltd  {  0000:0000:0000:0004
// False     {  0000:0000:0000:0006
// True      {  0000:0000:0000:0007
// Undefined {  0000:0000:0000:000a
// Pointer   {  0000:PPPP:PPPP:PPPP
//            / 0002:****:****:****
// Double    {         ...
//            \ FFFC:****:****:****
// Integer   {  FFFE:0000:IIII:IIII

// binary mask for js value computing
const INT_MASK: u64 = 0xfffe000000000000;
const PTR_LOW_MASK: u64 = 0x0000ffffffffffff;
const PTR_HIGH_MASK: u64 = 0xffff000000000000;
const DOUBLE_OFFSET: u64 = 1 << 49;
static mut PTR_HIGH_BITS: u64 = 1;

// safe integer range
const MAX_SAFE_INTEGER: i64 = 2_i64.pow(53) - 1;
const MIN_SAFE_INTEGER: i64 = -MAX_SAFE_INTEGER;

/// trait for representing js value
pub trait JSVal {
  /// generate `JSVal` from a js double value
  fn from_js_double(n: f64) -> Self;
  /// convert `JSVal` to a js double value
  fn to_js_double(&self) -> f64;
  /// check if `JSVal` represents a js double value
  fn is_js_double(&self) -> bool;
  /// check if `JSVal` represents a js semi-integer value
  fn is_js_semi_int(&self) -> bool;
  /// check if `JSVal` represents a js number value
  fn is_js_num(&self) -> bool;
}

impl JSVal for u64 {
  /// generate `u64` from a js double value
  fn from_js_double(n: f64) -> Self {
    f64::to_bits(n) as u64 + DOUBLE_OFFSET
  }

  /// convert `u64` to a js double value
  fn to_js_double(&self) -> f64 {
    f64::from_bits(self - DOUBLE_OFFSET)
  }

  /// check if a u64 represents a js double value
  fn is_js_double(&self) -> bool {
    (*self >= DOUBLE_OFFSET) & (*self < INT_MASK)
  }

  /// check if a `u64` represents a js semi-integer value
  fn is_js_semi_int(&self) -> bool {
    ((*self & INT_MASK) == INT_MASK) & (*self | INT_MASK == INT_MASK)
  }

  /// check if a `u64` represents a js number value
  fn is_js_num(&self) -> bool {
    self.is_js_double() | self.is_js_semi_int()
  }
}

// Canonical addresses[https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details]
//
//
// fn encode_ptr(ptr: u64)  {
//   unsafe {
//     if(PTR_HIGH_BITS == 1) {
//       PTR_HIGH_BITS = (ptr as u64) & PTR_HIGH_MASK;
//     }
//   }
//   (ptr as u64) & PTR_LOW_MASK;
// }

// fn decode_ptr(&self) {
//  let ptr: *const u64 =  unsafe {
//     std::mem::transmute(val | PTR_HIGH_BITS)
//   };
// }

#[cfg(test)]
mod tests {
  use crate::js_val::debug_bin;

  use super::*;

  #[test]
  fn val_is_num() {
    assert!(!(0_u64 << 49).is_js_num());
    assert!(!(1_u64 << 48).is_js_num());

    let js_val = 1_u64 << 49;
    assert!(js_val.is_js_num());
    assert!(js_val.is_js_double());
    assert!(!js_val.is_js_semi_int());

    let js_val = 1_u64 << 50;
    assert!(js_val.is_js_num());
    assert!(js_val.is_js_double());
    assert!(!js_val.is_js_semi_int());

    let js_val = 0xffff_u64 << 48;
    assert!(!js_val.is_js_num());
    assert!(!js_val.is_js_double());
    assert!(!js_val.is_js_semi_int());

    let js_val = 0xfffe_u64 << 48;
    assert!(js_val.is_js_num());
    assert!(!js_val.is_js_double());
    assert!(js_val.is_js_semi_int());
  }

  #[test]
  fn encode_double_value() {
    assert!(u64::from_js_double(0.0).is_js_double());
    assert!(u64::from_js_double(-0.0).is_js_double());
    assert!(u64::from_js_double(1.2).is_js_double());
    assert!(u64::from_js_double(MAX_SAFE_INTEGER as f64).is_js_double());
    assert!(u64::from_js_double(MIN_SAFE_INTEGER as f64).is_js_double());
    assert_eq!(u64::from_js_double(0.0).to_js_double(), 0.0);
    assert_eq!(u64::from_js_double(-0.0).to_js_double(), 0.0);
    assert_eq!(u64::from_js_double(1.2).to_js_double(), 1.2);
    assert_eq!(u64::from_js_double(2.0 / 3.0).to_js_double(), 2.0 / 3.0);
    assert_eq!(
      u64::from_js_double(MAX_SAFE_INTEGER as f64).to_js_double(),
      MAX_SAFE_INTEGER as f64
    );
    assert_eq!(
      u64::from_js_double(MIN_SAFE_INTEGER as f64).to_js_double(),
      MIN_SAFE_INTEGER as f64
    );
  }
}
