use super::{ErrorApi, Handle};

/// Interface to only be used by code generated by the macros.
/// The smart contract code doesn't have access to these methods directly.
pub trait EndpointFinishApi: ErrorApi {
    fn finish_slice_u8(&self, slice: &[u8]);

    fn finish_big_int_raw(&self, handle: Handle);

    fn finish_big_uint_raw(&self, handle: Handle);

    fn finish_managed_buffer_raw(&self, handle: Handle);

    fn finish_u64(&self, value: u64);

    fn finish_i64(&self, value: i64);
}
