/*!
eBPF maps.

Maps are a generic data structure for storage of different types of data.
They allow sharing of data between eBPF kernel programs, and also between
kernel and user-space code.
 */
use core::default::Default;
use core::marker::PhantomData;
use core::mem;
use cty::*;

use crate::bindings::*;

use redbpf_macros::internal_helpers as helpers;

/// Hash table map.
///
/// High level API for BPF_MAP_TYPE_HASH maps.
#[repr(transparent)]
pub struct HashMap<K, V> {
    def: bpf_map_def,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl<K, V> HashMap<K, V> {
    /// Creates a map with the specified maximum number of elements.
    pub const fn with_max_entries(max_entries: u32) -> Self {
        Self {
            def: bpf_map_def {
                type_: bpf_map_type_BPF_MAP_TYPE_HASH,
                key_size: mem::size_of::<K>() as u32,
                value_size: mem::size_of::<V>() as u32,
                max_entries,
                map_flags: 0,
            },
            _k: PhantomData,
            _v: PhantomData,
        }
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    #[helpers]
    pub fn get(&mut self, mut key: K) -> Option<&V> {
        unsafe {
            let value = bpf_map_lookup_elem(
                &mut self.def as *mut _ as *mut c_void,
                &mut key as *mut _ as *mut c_void,
            );
            if value.is_null() {
                None
            } else {
                Some(&*(value as *const V))
            }
        }
    }
}

/// Flags that can be passed to `PerfMap::insert_with_flags`.
#[derive(Debug, Copy, Clone)]
pub struct PerfMapFlags {
    index: Option<u32>,
    xdp_size: u32,
}

impl Default for PerfMapFlags {
    #[inline]
    fn default() -> Self {
        PerfMapFlags {
            index: None,
            xdp_size: 0,
        }
    }
}

impl PerfMapFlags {
    /// Create new default flags.
    ///
    /// Events inserted with default flags are keyed by the current CPU number
    /// and don't include any extra payload data.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create flags for events carrying `size` extra bytes of `XDP` payload data.
    #[inline]
    pub fn with_xdp_size(size: u32) -> Self {
        *PerfMapFlags::new().xdp_size(size)
    }

    /// Set the index key for the event to insert.
    #[inline]
    pub fn index(&mut self, index: u32) -> &mut PerfMapFlags {
        self.index = Some(index);
        self
    }

    /// Set the number of bytes of the `XDP` payload data to append to the event.
    #[inline]
    pub fn xdp_size(&mut self, size: u32) -> &mut PerfMapFlags {
        self.xdp_size = size;
        self
    }
}

impl From<PerfMapFlags> for u64 {
    #[inline]
    fn from(flags: PerfMapFlags) -> u64 {
        (flags.xdp_size as u64) << 32 | (flags.index.unwrap_or(BPF_F_CURRENT_CPU) as u64)
    }
}

/// Perf events map.
///
/// Perf events map that allows eBPF programs to store data in mmap()ed shared
/// memory accessible by user-space. This is a wrapper for
/// `BPF_MAP_TYPE_PERF_EVENT_ARRAY`.
///
/// If you're writing an `XDP` probe, you should use `xdp::PerfMap` instead which
/// exposes `XDP`-specific functionality.
#[repr(transparent)]
pub struct PerfMap<T> {
    def: bpf_map_def,
    _event: PhantomData<T>,
}

impl<T> PerfMap<T> {
    /// Creates a perf map with the specified maximum number of elements.
    pub const fn with_max_entries(max_entries: u32) -> Self {
        Self {
            def: bpf_map_def {
                type_: bpf_map_type_BPF_MAP_TYPE_PERF_EVENT_ARRAY,
                key_size: mem::size_of::<u32>() as u32,
                value_size: mem::size_of::<u32>() as u32,
                max_entries,
                map_flags: 0,
            },
            _event: PhantomData,
        }
    }

    /// Insert a new event in the perf events array keyed by the current CPU number.
    ///
    /// Each array can hold up to `max_entries` events, see `with_max_entries`.
    /// If you want to use a key other than the current CPU, see
    /// `insert_with_flags`.
    #[inline]
    #[helpers]
    pub fn insert<C>(&mut self, ctx: *mut C, data: T) {
        self.insert_with_flags(ctx, data, PerfMapFlags::default())
    }

    /// Insert a new event in the perf events array keyed by the index and with
    /// the additional xdp payload data specified in the given `PerfMapFlags`.
    #[inline]
    #[helpers]
    pub fn insert_with_flags<C>(&mut self, ctx: *mut C, mut data: T, flags: PerfMapFlags) {
        unsafe {
            bpf_perf_event_output(
                ctx as *mut _ as *mut c_void,
                &mut self.def as *mut _ as *mut c_void,
                flags.into(),
                &mut data as *mut _ as *mut c_void,
                mem::size_of::<T>() as u64,
            );
        };
    }
}
