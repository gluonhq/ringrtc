#![allow(unused_parens)]

use crate::core::signaling;
use crate::webrtc::peer_connection_factory::AudioDevice;

use core::slice;
use std::{
    ffi::c_void,
    fmt,
};

#[repr(C)]
#[derive(Debug)]
pub struct JPString {
    len: usize,
    buff: *mut u8,
}

impl JPString {
    pub fn to_string(&self) -> String {
        let answer = unsafe { String::from_raw_parts(self.buff, self.len, self.len) };
        answer
    }

    /*
        pub fn from_string(src: String) -> Self {
            let string_len = src.len();
            let mut string_bytes = src.as_bytes().as_mut_ptr();
            Self {
                len: string_len,
                buff: string_bytes
            }
        }
    */
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RString<'a> {
    len: usize,
    buff: *const u8,
    phantom: std::marker::PhantomData<&'a u8>,
}

impl<'a> RString<'a> {
    pub fn from_string(src: String) -> Self {
        let string_len = src.len();
        let string_bytes = src.as_bytes().as_ptr();
        Self {
            len: string_len,
            buff: string_bytes,
            phantom: std::marker::PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JArrayByte {
    pub len: usize,
    pub data: *const c_void,
}

impl fmt::Display for JArrayByte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "JArrayByte with {} bytes at {:?}",
            self.len,
            &(self.data)
        )
    }
}
impl JArrayByte {
    pub fn new(vector: Vec<u8>) -> Self {
        let vlen = vector.len();
        let boxed_vector = Box::into_raw(vector.into_boxed_slice());
        JArrayByte {
            len: vlen,
            data: boxed_vector as *const c_void,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JByteArray {
    len: usize,
    pub buff: *const u8,
}

impl fmt::Display for JByteArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let address = &self.buff;
        write!(f, "jByteArray with {} bytes at {:p}", self.len, self.buff)
    }
}

impl JByteArray {
    pub fn new(vector: Vec<u8>) -> Self {
        let slice = vector.as_slice();
        let buffer = slice.as_ptr();
        JByteArray {
            len: vector.len(),
            buff: buffer,
        }
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        let answer = unsafe { slice::from_raw_parts(self.buff, self.len).to_vec() };
        answer
    }

    pub fn empty() -> Self {
        let bar = Vec::new().as_ptr();
        JByteArray { len: 0, buff: bar }
    }

    pub fn from_data(data: *const u8, len: usize) -> Self {
        JByteArray {
            len: len,
            buff: data,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct JByteArray2D {
    pub len: usize,
    pub buff: [JByteArray; 32],
}

impl JByteArray2D {
    pub fn new(vector: Vec<signaling::IceCandidate>) -> Self {
        let vlen = vector.len();
        // let mut myrows = [Opaque::empty(); 25];
        let mut myrows: [JByteArray; 32] = [JByteArray::empty(); 32];
        for i in 0..25 {
            if (i < vlen) {
                myrows[i] =
                    JByteArray::from_data(vector[i].opaque.as_ptr(), vector[i].opaque.len());
            } else {
                myrows[i] = JByteArray::new(Vec::new());
            }
        }
        JByteArray2D {
            len: vlen,
            buff: myrows,
        }
    }
}

#[repr(C)]
struct Buffer {
    data: *mut u8,
    len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TringDevice<'a> {
    index: u32,
    name: RString<'a>,
    unique_id: RString<'a>,
    int_key: RString<'a>,
}

impl<'a> TringDevice<'a> {
    pub fn empty() -> Self {
        let name = RString::from_string("empty".to_string());
        let unique_id = RString::from_string("empty".to_string());
        let int_key = RString::from_string("empty".to_string());
        Self {
            index: 99,
            name: name,
            unique_id: unique_id,
            int_key: int_key,
        }
    }

    pub fn from_audio_device(index: u32, src: AudioDevice) -> Self {
        let src_name = RString::from_string(src.name);
        let src_unique_id = RString::from_string(src.unique_id);
        let src_int_key = RString::from_string(src.i18n_key);
        Self {
            index: index,
            name: src_name,
            unique_id: src_unique_id,
            int_key: src_int_key,
        }
    }
    pub fn from_fields(
        index: u32,
        src_name: String,
        src_unique_id: String,
        src_i18n_key: String,
    ) -> Self {
        let src_name = RString::from_string(src_name);
        let src_unique_id = RString::from_string(src_unique_id);
        let src_int_key = RString::from_string(src_i18n_key);
        Self {
            index: index,
            name: src_name,
            unique_id: src_unique_id,
            int_key: src_int_key,
        }
    }
}
