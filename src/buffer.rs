use ringbuf::{SharedRb, HeapRb};
use ringbuf::ring_buffer::{Container, Rb, RbBase, RbRead, RbWrite};
use ringbuf::{consumer::Consumer, producer::Producer};
use uninit::extension_traits::MaybeUninitExt;

use std::marker::PhantomData;
use std::{
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroUsize,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
// use crossbeam_utils::CachePadded;

use memmap2::{Mmap, MmapOptions, MmapMut};

const PAGE_SIZE: u64 = n_kb_bytes!(4) as u64;

pub struct TerminalMemoryMappedBuffer<T> {
    buffer: MmapMut,
    head: AtomicUsize,
    tail: AtomicUsize,
    marker: PhantomData<T>
}

impl<T> RbBase<T> for TerminalMemoryMappedBuffer<T> {
    // #[inline]
    // unsafe fn slices(
    //     &self,
    //     head: usize,
    //     tail: usize,
    // ) -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>]) {
    //     self.buffer.as_mut_slices(head, tail)
    // }
    unsafe fn data(&self) -> &mut [MaybeUninit<T>] {
        let (_, body, _) = unsafe { self.buffer.align_to_mut::<T>()};
        std::mem::transmute(body)
    }

    #[inline]
    fn capacity_nonzero(&self) -> NonZeroUsize {
        // self.storage.len()
        NonZeroUsize::new(self.buffer.len()).unwrap()
    }

    #[inline]
    fn head(&self) -> usize {
        let x = HeapRb::<u32>::new(32);
        // x.data()
        self.head.load(Ordering::Acquire)
    }

    #[inline]
    fn tail(&self) -> usize {
        self.tail.load(Ordering::Acquire)
    }
}

impl<T> Rb<T> for TerminalMemoryMappedBuffer<T> {}

impl<T> RbRead<T> for TerminalMemoryMappedBuffer<T> {
    #[inline]
    unsafe fn set_head(&self, value: usize) {
        self.head.store(value, Ordering::Release)
    }
}

impl<T> RbWrite<T> for TerminalMemoryMappedBuffer<T> {
    #[inline]
    unsafe fn set_tail(&self, value: usize) {
        self.tail.store(value, Ordering::Release)
    }
}

impl<T> TerminalMemoryMappedBuffer<T> {
    pub fn new(buffer_size: u64) -> Self {
        let data_size = (buffer_size +  PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let mmap = unsafe { MmapOptions::new().len(data_size.try_into().unwrap()).map_anon()}.expect("Failed to map history buffer");
        Self {
            buffer: mmap,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            marker: PhantomData
        }
    }
}