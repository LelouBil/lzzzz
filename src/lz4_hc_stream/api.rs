#![allow(unsafe_code)]

use crate::{binding, binding::LZ4StreamHC, Error, Result};

use crate::lz4_hc::CompressionLevel;
use libc::{c_char, c_int, c_void, size_t};
use std::{
    mem::{size_of, MaybeUninit},
    ptr::NonNull,
};

enum Stream {
    Stack(LZ4StreamHC),
    Heap(NonNull<LZ4StreamHC>),
}

pub struct CompressionContext {
    stream: Stream,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let mut stream = MaybeUninit::<LZ4StreamHC>::zeroed();
        unsafe {
            let ptr = binding::LZ4_initStreamHC(
                stream.as_mut_ptr() as *mut c_void,
                size_of::<LZ4StreamHC>() as size_t,
            );
            if !ptr.is_null() {
                return Ok(Self {
                    stream: Stream::Stack(stream.assume_init()),
                });
            }
            let ptr = NonNull::new(binding::LZ4_createStreamHC());
            ptr.ok_or(Error::Generic).map(|stream| Self {
                stream: Stream::Heap(stream),
            })
        }
    }

    fn get_ptr(&mut self) -> *mut LZ4StreamHC {
        match &mut self.stream {
            Stream::Stack(stream) => stream as *mut LZ4StreamHC,
            Stream::Heap(ptr) => ptr.as_ptr(),
        }
    }

    pub fn set_compression_level(&mut self, compression_level: i32) {
        unsafe { binding::LZ4_setCompressionLevel(self.get_ptr(), compression_level as c_int) }
    }

    pub fn set_favor_dec_speed(&mut self, flag: bool) {
        unsafe {
            binding::LZ4_favorDecompressionSpeed(self.get_ptr(), if flag { 1 } else { 0 } as c_int)
        }
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> usize {
        unsafe {
            binding::LZ4_compress_HC_continue(
                self.get_ptr(),
                src.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                src.len() as c_int,
                dst.len() as c_int,
            ) as usize
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        if let Stream::Heap(mut ptr) = self.stream {
            unsafe {
                binding::LZ4_freeStreamHC(ptr.as_mut());
            }
        }
    }
}
