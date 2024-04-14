use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::copy_nonoverlapping;
use crate::proto::network::tcpv4::definitions::TCPv4FragmentData;
use crate::tcpv4::TCPv4FragmentData;

/// This type is necessary because the underlying structure has a flexible array member.
/// Due to this, the memory for the instance needs to be carefully managed.
/// A Box cannot be used because the Box doesn't have the full knowledge of the layout.
/// A wide pointer also cannot be used because the layout needs to be precisely controlled for FFI.
/// Therefore, we use a wrapper 'handle' to manage the lifecycle of the allocation manually.
#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ReceiveDataHandle<'a> {
    ptr: *const TCPv4ReceiveData,
    layout: Layout,
    phantom: PhantomData<&'a ()>,
}

impl<'a> TCPv4ReceiveDataHandle<'a> {
    fn total_layout_size(fragment_count: usize) -> usize {
        let size_of_fragments = mem::size_of::<ManuallyDrop<TCPv4FragmentData>>() * fragment_count;
        mem::size_of::<Self>() + size_of_fragments
    }

    pub(crate) fn new() -> Self {
        let buffer_len = 2048*16;
        let fragment = ManuallyDrop::new(TCPv4FragmentData::with_buffer_len(buffer_len));
        let layout = Layout::from_size_align(
            Self::total_layout_size(1),
            mem::align_of::<Self>(),
        ).unwrap();
        unsafe {
            let ptr = alloc::alloc::alloc(layout) as *mut TCPv4ReceiveData;
            (*ptr).urgent = false;
            (*ptr).data_length = buffer_len as _;

            let fragment_count = 1;
            (*ptr).fragment_count = fragment_count as _;
            copy_nonoverlapping(
                &fragment as *const _,
                (*ptr).fragment_table.as_mut_ptr(),
                fragment_count,
            );

            Self {
                ptr: ptr as _,
                layout,
                phantom: PhantomData,
            }
        }
    }

    pub(crate) fn get_data_ref(&self) -> &'a TCPv4ReceiveData {
        // Safety: The reference is strictly tied to the lifetime of this handle
        unsafe { &*self.ptr }
    }
}

impl Drop for TCPv4ReceiveDataHandle<'_> {
    fn drop(&mut self) {
        let ptr = self.ptr as *mut TCPv4ReceiveData;
        unsafe {
            // First, drop all the fragments
            let fragment_table: *mut ManuallyDrop<TCPv4FragmentData> = (*ptr).fragment_table.as_mut_ptr();
            for i in 0..((*ptr).fragment_count as usize) {
                let fragment_ptr = fragment_table.add(i as _);
                ManuallyDrop::drop(&mut *fragment_ptr);
            }

            // Lastly, drop the allocation itself
            alloc::alloc::dealloc(ptr as *mut u8, self.layout);
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ReceiveData {
    urgent: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: [ManuallyDrop<TCPv4FragmentData>; 0],
}

impl TCPv4ReceiveData {
    pub fn read_buffers(&self) -> Vec<u8> {
        let mut out = vec![];
        unsafe {
            let ptr = self as *const Self;
            let fragment_table: *const ManuallyDrop<TCPv4FragmentData> = (*ptr).fragment_table.as_ptr();
            for i in 0..(self.fragment_count as usize) {
                let fragment_ptr = fragment_table.add(i as _);
                let fragment = &*fragment_ptr;
                let fragment_buf = fragment.fragment_buf as *const u8;
                let fragment_slice = core::slice::from_raw_parts(fragment_buf, self.data_length as _);
                out.extend_from_slice(fragment_slice);
            }
        }
        out
    }
}

impl Drop for TCPv4ReceiveData {
    fn drop(&mut self) {
        panic!("Should be manually dropped by TCPv4ReceiveDataHandle")
    }
}
