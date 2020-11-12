use alloc::boxed::Box;
use alloc::vec::Vec;
use core::num::NonZeroUsize;

use crate::codec_err::EncodeError;
use crate::nested_ser_output::NestedEncodeOutput;
use crate::TypeInfo;
use core::ops::Add;
use generic_array::ArrayLength;
use typenum::{Unsigned, U0, U1, U2, U4, U8};

/// Most types will be encoded without any possibility of error.
/// The trait is used to provide these implementations.
/// This is currently not a substitute for implementing a proper TopEncode.
pub trait NestedEncodeNoErr: Sized {
	fn dep_encode_no_err<O: NestedEncodeOutput>(&self, dest: &mut O);
}

/// Trait that allows zero-copy write of value-references to slices in LE format.
///
/// Implementations should override `using_top_encoded` for value types and `dep_encode` and `size_hint` for allocating types.
/// Wrapper types should override all methods.
pub trait NestedEncode: Sized {
	// !INTERNAL USE ONLY!
	// This const helps elrond-codec to optimize the encoding/decoding by doing fake specialization.
	// It has currently somewhat fallen into disuse.
	#[doc(hidden)]
	const TYPE_INFO: TypeInfo = TypeInfo::Unknown;

	/// NestedEncode to output, using the format of an object nested inside another structure.
	/// Does not provide compact version.
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError>;

	/// Version of `dep_encode` that exits quickly in case of error.
	/// Its purpose is to create smaller implementations
	/// in cases where the application is supposed to exit directly on decode error.
	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		match self.dep_encode(dest) {
			Ok(v) => v,
			Err(e) => exit(c, e),
		}
	}
	/// When set to true, signals that the encoded length is a compile-time constant.
	/// The framework can then tell the compiler to allocate the destination bytes statically on stack.
	const HAS_CONST_ENCODE_LEN: bool = false;

	/// If the encoded length is a compile-time constant, this type tells the compiler how much this length is.
	/// e.g. u32 is always encoded on 4 bytes, so it has a ConstEncodeLen of U4.
	///
	/// For more info about how these types work, consult the `typenum` crate documentation.
	///
	/// Note: this length non-zero does not mean that that the type has fixed representation,
	/// neither vice-versa, we have types that encode to nothing, e.g. `()`.
	/// The `HAS_CONST_ENCODE_LEN` flag is the only one controlling
	/// whether or not a type can pe serialized directly to stack.
	type ConstEncodeLen: ArrayLength<u8> = U0;

	/// Convenience associated function to get the constant encode length as a number in code,
	/// rather than as a type.
	fn const_encode_len() -> usize {
		Self::ConstEncodeLen::to_usize()
	}

	/// Same as `dep_encode_or_exit` but will write bytes directly to a given slice,
	/// instead of appending to a buffer.
	/// Only works if the type has a constant encode length.
	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		_target: &mut [u8],
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		exit(c, EncodeError::UNSUPPORTED_OPERATION);
	}
}

/// Convenience function for getting an object nested-encoded to a Vec<u8> directly.
pub fn dep_encode_to_vec<T: NestedEncode>(obj: &T) -> Result<Vec<u8>, EncodeError> {
	let mut bytes = Vec::<u8>::new();
	obj.dep_encode(&mut bytes)?;
	Ok(bytes)
}

/// Adds the concantenated encoded contents of a slice to an output buffer,
/// without serializing the slice length.
/// Byte slice is treated separately, via direct transmute.
pub fn dep_encode_slice_contents<T: NestedEncode, O: NestedEncodeOutput>(
	slice: &[T],
	dest: &mut O,
) -> Result<(), EncodeError> {
	match T::TYPE_INFO {
		TypeInfo::U8 => {
			// cast &[T] to &[u8]
			let slice: &[u8] =
				unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) };
			dest.write(slice);
		},
		_ => {
			for x in slice {
				x.dep_encode(dest)?;
			}
		},
	}
	Ok(())
}

pub fn dep_encode_slice_contents_or_exit<T, O, ExitCtx>(
	slice: &[T],
	dest: &mut O,
	c: ExitCtx,
	exit: fn(ExitCtx, EncodeError) -> !,
) where
	T: NestedEncode,
	O: NestedEncodeOutput,
	ExitCtx: Clone,
{
	match T::TYPE_INFO {
		TypeInfo::U8 => {
			// cast &[T] to &[u8]
			let slice: &[u8] =
				unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) };
			dest.write(slice);
		},
		_ => {
			for x in slice {
				x.dep_encode_or_exit(dest, c.clone(), exit);
			}
		},
	}
}

impl NestedEncode for () {
	fn dep_encode<O: NestedEncodeOutput>(&self, _: &mut O) -> Result<(), EncodeError> {
		Ok(())
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		_: &mut O,
		_: ExitCtx,
		_: fn(ExitCtx, EncodeError) -> !,
	) {
	}

	/// `()` has a constant encode length of 0.
	const HAS_CONST_ENCODE_LEN: bool = true;

	/// `()` is always serialized as 0 bytes.
	type ConstEncodeLen = U0;

	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		_: &mut [u8],
		_: ExitCtx,
		_: fn(ExitCtx, EncodeError) -> !,
	) {
	}
}

impl<T: NestedEncode> NestedEncode for &[T] {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		// push size
		self.len().dep_encode(dest)?;
		// actual data
		dep_encode_slice_contents(self, dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		// push size
		self.len().dep_encode_or_exit(dest, c.clone(), exit);
		// actual data
		dep_encode_slice_contents_or_exit(self, dest, c, exit);
	}
}

impl<T: NestedEncode> NestedEncode for &T {
	#[inline]
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		(*self).dep_encode(dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		(*self).dep_encode_or_exit(dest, c, exit);
	}

	const HAS_CONST_ENCODE_LEN: bool = T::HAS_CONST_ENCODE_LEN;

	type ConstEncodeLen = T::ConstEncodeLen;

	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		target: &mut [u8],
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		(*self).dep_encode_in_place_or_exit(target, c, exit);
	}
}

impl NestedEncode for &str {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.as_bytes().dep_encode(dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.as_bytes().dep_encode_or_exit(dest, c, exit);
	}
}

impl<T: NestedEncode> NestedEncode for Vec<T> {
	#[inline]
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.as_slice().dep_encode(dest)
	}

	#[inline]
	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.as_slice().dep_encode_or_exit(dest, c, exit);
	}
}

/// Unlike the other numeric types, there are no bytes to reverse here
/// from LE to BE.
impl NestedEncode for u8 {
	type ConstEncodeLen = U1;
	const HAS_CONST_ENCODE_LEN: bool = true;
	const TYPE_INFO: TypeInfo = TypeInfo::U8;

	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		dest.push_byte(*self);
		Ok(())
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		_: ExitCtx,
		_: fn(ExitCtx, EncodeError) -> !,
	) {
		dest.push_byte(*self);
	}

	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		target: &mut [u8],
		_: ExitCtx,
		_: fn(ExitCtx, EncodeError) -> !,
	) {
		target[0] = *self;
	}
}

/// The main unsigned types need to be reversed before serializing,
/// From LE which is wasm-native to BE, which is the Elrond standard.
macro_rules! encode_num_unsigned {
	($num_type:ty, $size_in_bits:expr, $fsr_len_type:ty, $type_info:expr) => {
		impl NestedEncode for $num_type {
			type ConstEncodeLen = $fsr_len_type;
			const HAS_CONST_ENCODE_LEN: bool = true;
			const TYPE_INFO: TypeInfo = $type_info;

			fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
				dest.write(&self.to_be_bytes()[..]);
				Ok(())
			}

			fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
				&self,
				dest: &mut O,
				_: ExitCtx,
				_: fn(ExitCtx, EncodeError) -> !,
			) {
				dest.write(&self.to_be_bytes()[..]);
			}

			fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
				&self,
				target: &mut [u8],
				_: ExitCtx,
				_: fn(ExitCtx, EncodeError) -> !,
			) {
				target.copy_from_slice(&self.to_be_bytes()[..]);
			}
		}
	};
}

encode_num_unsigned! {u64, 64, U8, TypeInfo::U64}
encode_num_unsigned! {u32, 32, U4, TypeInfo::U32}
encode_num_unsigned! {u16, 16, U2, TypeInfo::U16}

// Derive the implementation of the other types by casting.
macro_rules! encode_num_mimic {
	($num_type:ty, $mimic_type:ident, $type_info:expr) => {
		impl NestedEncode for $num_type {
			type ConstEncodeLen = <$mimic_type as NestedEncode>::ConstEncodeLen;
			const HAS_CONST_ENCODE_LEN: bool = $mimic_type::HAS_CONST_ENCODE_LEN;
			const TYPE_INFO: TypeInfo = $type_info;

			fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
				(*self as $mimic_type).dep_encode(dest)
			}

			fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
				&self,
				dest: &mut O,
				c: ExitCtx,
				exit: fn(ExitCtx, EncodeError) -> !,
			) {
				(*self as $mimic_type).dep_encode_or_exit(dest, c, exit);
			}

			fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
				&self,
				target: &mut [u8],
				c: ExitCtx,
				exit: fn(ExitCtx, EncodeError) -> !,
			) {
				(*self as $mimic_type).dep_encode_in_place_or_exit(target, c, exit);
			}
		}
	};
}

encode_num_mimic! {usize, u32, TypeInfo::USIZE}
encode_num_mimic! {i64, u64, TypeInfo::I64}
encode_num_mimic! {i32, u32, TypeInfo::I32}
encode_num_mimic! {isize, u32, TypeInfo::ISIZE}
encode_num_mimic! {i16, u16, TypeInfo::I16}
encode_num_mimic! {i8, u8, TypeInfo::I8}
encode_num_mimic! {bool, u8, TypeInfo::Bool}

impl<T: NestedEncode> NestedEncode for Option<T> {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		match self {
			Some(v) => {
				dest.push_byte(1u8);
				v.dep_encode(dest)
			},
			None => {
				dest.push_byte(0u8);
				Ok(())
			},
		}
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		match self {
			Some(v) => {
				dest.push_byte(1u8);
				v.dep_encode_or_exit(dest, c, exit);
			},
			None => {
				dest.push_byte(0u8);
			},
		}
	}
}

impl<T: NestedEncode> NestedEncode for Box<T> {
	#[inline(never)]
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.as_ref().dep_encode(dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.as_ref().dep_encode_or_exit(dest, c, exit);
	}
}

impl<T: NestedEncode> NestedEncode for Box<[T]> {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.as_ref().dep_encode(dest)
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.as_ref().dep_encode_or_exit(dest, c, exit);
	}
}

macro_rules! typenum_add {
	($n1:ty, $n2:ty) => {
		<$n1 as Add<$n2>>::Output
	};
}

/// Treating 1-tuple separately, to account for ConstEncodeLen.
impl<T0> NestedEncode for (T0,)
where
	T0: NestedEncode,
{
	type ConstEncodeLen = T0::ConstEncodeLen;

	const HAS_CONST_ENCODE_LEN: bool = T0::HAS_CONST_ENCODE_LEN;

	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.0.dep_encode(dest)?;
		Ok(())
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.0.dep_encode_or_exit(dest, c.clone(), exit);
	}

	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		target: &mut [u8],
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.0.dep_encode_in_place_or_exit(target, c, exit);
	}
}

/// Treating 2-tuple separately.
impl<T0, T1> NestedEncode for (T0, T1)
where
	T0: NestedEncode,
	T1: NestedEncode,
	T1::ConstEncodeLen: Add<T0::ConstEncodeLen>,
	typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen): ArrayLength<u8>,
{
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.0.dep_encode(dest)?;
		self.1.dep_encode(dest)?;
		Ok(())
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.0.dep_encode_or_exit(dest, c.clone(), exit);
		self.1.dep_encode_or_exit(dest, c, exit);
	}

	/// Has constant encode length if all elements have constant encode length.
	const HAS_CONST_ENCODE_LEN: bool = T0::HAS_CONST_ENCODE_LEN && T1::HAS_CONST_ENCODE_LEN;

	/// Its constant encode length is the sum of its components.
	type ConstEncodeLen = typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen);

	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		target: &mut [u8],
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		let t0_start: usize = 0;
		let t1_start: usize = t0_start + T0::const_encode_len();
		self.0
			.dep_encode_in_place_or_exit(&mut target[t0_start..t1_start], c.clone(), exit);
		self.1
			.dep_encode_in_place_or_exit(&mut target[t1_start..], c, exit);
	}
}

/// Treating 3-tuple separately.
/// TODO: figure a way to generate this in the macro.
impl<T0, T1, T2> NestedEncode for (T0, T1, T2)
where
	T0: NestedEncode,
	T1: NestedEncode,
	T1::ConstEncodeLen: Add<T0::ConstEncodeLen>,
	typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen): ArrayLength<u8>,
	T2: NestedEncode,
	T2::ConstEncodeLen: Add<typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen)>,
	typenum_add!(
		T2::ConstEncodeLen,
		typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen)
	): ArrayLength<u8>,
{
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.0.dep_encode(dest)?;
		self.1.dep_encode(dest)?;
		self.2.dep_encode(dest)?;
		Ok(())
	}

	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.0.dep_encode_or_exit(dest, c.clone(), exit);
		self.1.dep_encode_or_exit(dest, c.clone(), exit);
		self.2.dep_encode_or_exit(dest, c, exit);
	}

	/// Has constant encode length if all elements have constant encode length.
	const HAS_CONST_ENCODE_LEN: bool =
		T0::HAS_CONST_ENCODE_LEN && T1::HAS_CONST_ENCODE_LEN && T2::HAS_CONST_ENCODE_LEN;

	/// Its constant encode length is the sum of its components.
	/// TODO: figure a way to generate this in the macro.
	type ConstEncodeLen = typenum_add!(
		T2::ConstEncodeLen,
		typenum_add!(T1::ConstEncodeLen, T0::ConstEncodeLen)
	);

	/// TODO: figure a way to generate this in the macro.
	fn dep_encode_in_place_or_exit<ExitCtx: Clone>(
		&self,
		target: &mut [u8],
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		let t0_start: usize = 0;
		let t1_start: usize = t0_start + T0::const_encode_len();
		let t2_start: usize = t1_start + T1::const_encode_len();
		self.0
			.dep_encode_in_place_or_exit(&mut target[t0_start..t1_start], c.clone(), exit);
		self.1
			.dep_encode_in_place_or_exit(&mut target[t1_start..t2_start], c.clone(), exit);
		self.2
			.dep_encode_in_place_or_exit(&mut target[t2_start..], c, exit);
	}
}

macro_rules! tuple_impls_no_const_en_len {
    ($(($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> NestedEncode for ($($name,)+)
            where
                $($name: NestedEncode,)+
            {
				fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
					$(
                        self.$n.dep_encode(dest)?;
                    )+
					Ok(())
				}

				fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
					$(
                        self.$n.dep_encode_or_exit(dest, c.clone(), exit);
                    )+
				}
            }
        )+
    }
}

// TODO: generalize implementations of 1, 2 and 3 to all.
tuple_impls_no_const_en_len! {
	// (0 T0)
	// (0 T0 1 T1)
	// (0 T0 1 T1 2 T2)
	(0 T0 1 T1 2 T2 3 T3)
	(0 T0 1 T1 2 T2 3 T3 4 T4)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
	(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

// TODO: add constant encode length.
macro_rules! array_impls {
    ($($n: tt,)+) => {
        $(
            impl<T: NestedEncode> NestedEncode for [T; $n] {
				#[inline]
				fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
					dep_encode_slice_contents(&self[..], dest)
				}

				#[inline]
				fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(&self, dest: &mut O, c: ExitCtx, exit: fn(ExitCtx, EncodeError) -> !) {
					dep_encode_slice_contents_or_exit(&self[..], dest, c, exit);
				}
			}
        )+
    }
}

#[rustfmt::skip]
array_impls!(
	1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
	17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
	32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
	52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71,
	72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91,
	92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
	109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124,
	125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140,
	141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156,
	157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172,
	173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188,
	189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204,
	205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
	221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236,
	237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252,
	253, 254, 255, 256, 384, 512, 768, 1024, 2048, 4096, 8192, 16384, 32768,
);

impl NestedEncode for NonZeroUsize {
	#[inline]
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		self.get().dep_encode(dest)
	}

	#[inline]
	fn dep_encode_or_exit<O: NestedEncodeOutput, ExitCtx: Clone>(
		&self,
		dest: &mut O,
		c: ExitCtx,
		exit: fn(ExitCtx, EncodeError) -> !,
	) {
		self.get().dep_encode_or_exit(dest, c, exit);
	}
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
	use super::super::test_struct::*;
	use super::*;
	use crate::test_util::check_dep_encode;
	use core::fmt::Debug;

	fn ser_ok<V>(element: V, expected_bytes: &[u8])
	where
		V: NestedEncode + PartialEq + Debug + 'static,
	{
		let bytes = check_dep_encode(&element);
		assert_eq!(bytes.as_slice(), expected_bytes);
	}

	#[test]
	fn test_dep_encode_numbers() {
		// unsigned positive
		ser_ok(5u8, &[5]);
		ser_ok(5u16, &[0, 5]);
		ser_ok(5u32, &[0, 0, 0, 5]);
		ser_ok(5usize, &[0, 0, 0, 5]);
		ser_ok(5u64, &[0, 0, 0, 0, 0, 0, 0, 5]);
		// signed positive
		ser_ok(5i8, &[5]);
		ser_ok(5i16, &[0, 5]);
		ser_ok(5i32, &[0, 0, 0, 5]);
		ser_ok(5isize, &[0, 0, 0, 5]);
		ser_ok(5i64, &[0, 0, 0, 0, 0, 0, 0, 5]);
		// signed negative
		ser_ok(-5i8, &[251]);
		ser_ok(-5i16, &[255, 251]);
		ser_ok(-5i32, &[255, 255, 255, 251]);
		ser_ok(-5isize, &[255, 255, 255, 251]);
		ser_ok(-5i64, &[255, 255, 255, 255, 255, 255, 255, 251]);
		// non zero usize
		ser_ok(NonZeroUsize::new(5).unwrap(), &[0, 0, 0, 5]);
	}

	#[test]
	fn test_dep_encode_bool() {
		ser_ok(true, &[1]);
		ser_ok(false, &[0]);
	}

	#[test]
	fn test_dep_encode_empty_bytes() {
		let empty_byte_slice: &[u8] = &[];
		ser_ok(empty_byte_slice, &[0, 0, 0, 0]);
	}

	#[test]
	fn test_dep_encode_bytes() {
		ser_ok(&[1u8, 2u8, 3u8][..], &[0, 0, 0, 3, 1u8, 2u8, 3u8]);
	}

	#[test]
	fn test_dep_encode_vec_u8() {
		let some_vec = [1u8, 2u8, 3u8].to_vec();
		ser_ok(some_vec, &[0, 0, 0, 3, 1u8, 2u8, 3u8]);
	}

	#[test]
	fn test_dep_encode_vec_i32() {
		let some_vec = [1i32, 2i32, 3i32].to_vec();
		let expected: &[u8] = &[0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3];
		ser_ok(some_vec, expected);
	}

	#[test]
	fn test_struct() {
		let test = Test {
			int: 1,
			seq: [5, 6].to_vec(),
			another_byte: 7,
		};

		ser_ok(test, &[0, 1, 0, 0, 0, 2, 5, 6, 7]);
	}

	#[test]
	fn test_tuple() {
		ser_ok((7u32, -2i16), &[0, 0, 0, 7, 255, 254]);
	}

	#[test]
	fn test_unit() {
		ser_ok((), &[]);
	}

	#[test]
	fn test_enum() {
		let u = E::Unit;
		let expected: &[u8] = &[/*variant index*/ 0, 0, 0, 0];
		ser_ok(u, expected);

		let n = E::Newtype(1);
		let expected: &[u8] = &[/*variant index*/ 0, 0, 0, 1, /*data*/ 0, 0, 0, 1];
		ser_ok(n, expected);

		let t = E::Tuple(1, 2);
		let expected: &[u8] = &[
			/*variant index*/ 0, 0, 0, 2, /*(*/ 0, 0, 0, 1, /*,*/ 0, 0, 0,
			2, /*)*/
		];
		ser_ok(t, expected);

		let s = E::Struct { a: 1 };
		let expected: &[u8] = &[/*variant index*/ 0, 0, 0, 3, /*data*/ 0, 0, 0, 1];
		ser_ok(s, expected);
	}

	#[test]
	fn test_fixed_repr_len() {
		assert!(u8::HAS_CONST_ENCODE_LEN);
		assert!(u16::HAS_CONST_ENCODE_LEN);
		assert!(u32::HAS_CONST_ENCODE_LEN);
		assert!(u32::HAS_CONST_ENCODE_LEN);

		assert_eq!(u8::const_encode_len(), 1);
		assert_eq!(u16::const_encode_len(), 2);
		assert_eq!(u32::const_encode_len(), 4);
		assert_eq!(u64::const_encode_len(), 8);

		assert!(<(u32, u8) as NestedEncode>::HAS_CONST_ENCODE_LEN);
		assert_eq!(<(u32, u8)>::const_encode_len(), 4 + 1);
		assert!(<(u32, u8, u16) as NestedEncode>::HAS_CONST_ENCODE_LEN);
		assert_eq!(<(u32, u8, u16)>::const_encode_len(), 4 + 1 + 2);

		// no fixed size representation
		assert!(!Vec::<u32>::HAS_CONST_ENCODE_LEN);
		assert_eq!(Vec::<u32>::const_encode_len(), 0);
	}
}
