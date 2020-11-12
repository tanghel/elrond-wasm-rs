use crate::*;
use elrond_codec::*;

use core::mem::MaybeUninit;
use core::ops::Add;
use generic_array::{ArrayLength, GenericArray};
use typenum::operator_aliases::Sum;

/// Will call closure with reference to computed key.
/// For types that have fixed encoded width,
/// the key will be assembled directly on stack, no heap allocations.
#[inline]
pub fn with_storage_key<A, BigInt, BigUint, K, T, F, R>(
	api: A,
	key_name: GenericArray<u8, K>,
	tuple: T,
	f: F,
) -> R
where
	BigInt: NestedEncode + 'static,
	BigUint: NestedEncode + 'static,
	A: ContractHookApi<BigInt, BigUint> + ContractIOApi<BigInt, BigUint> + 'static,
	K: ArrayLength<u8>,
	T: NestedEncode,
	T::ConstEncodeLen: Add<K>,
	Sum<T::ConstEncodeLen, K>: ArrayLength<u8>,
	F: FnOnce(&[u8]) -> R,
{
	if T::HAS_CONST_ENCODE_LEN {
		unsafe {
			let mut key_arr =
				MaybeUninit::<GenericArray<u8, Sum<T::ConstEncodeLen, K>>>::zeroed().assume_init();
			let key_mut_slice = key_arr.as_mut_slice();
			&key_mut_slice[..K::to_usize()].copy_from_slice(key_name.as_slice());
			tuple.dep_encode_in_place_or_exit(
				&mut key_mut_slice[K::to_usize()..],
				api,
				key_array_exit,
			);
			f(key_arr.as_slice())
		}
	} else {
		let mut key_vec: Vec<u8> = Vec::new();
		key_vec.extend_from_slice(key_name.as_slice());
		tuple.dep_encode_or_exit(&mut key_vec, api, key_array_exit);
		f(key_vec.as_slice())
	}
}

/// Since encoding doesn't really ever fail, this doesn't really ever get called.
#[inline(always)]
fn key_array_exit<A, BigInt, BigUint>(api: A, en_err: EncodeError) -> !
where
	BigInt: NestedEncode + 'static,
	BigUint: NestedEncode + 'static,
	A: ContractHookApi<BigInt, BigUint> + ContractIOApi<BigInt, BigUint> + 'static,
{
	let encode_err_message =
		BoxedBytes::from_concat(&[err_msg::STORAGE_KEY_ENCODE_ERROR, en_err.message_bytes()][..]);
	api.signal_error(encode_err_message.as_slice())
}
