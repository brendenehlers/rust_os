use x86_64::structures::paging::{self, mapper};

use crate::allocator::fixed_size_block::FixedSizeBlockAllocator;

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl mapper::Mapper<paging::Size4KiB>,
    frame_allocator: &mut impl paging::FrameAllocator<paging::Size4KiB>,
) -> Result<(), mapper::MapToError<paging::Size4KiB>> {
    let page_range = {
        let heap_start = x86_64::VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = paging::Page::containing_address(heap_start);
        let heap_end_page = paging::Page::containing_address(heap_end);
        paging::Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(mapper::MapToError::FrameAllocationFailed)?;
        let flags = paging::PageTableFlags::PRESENT | paging::PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` be a power of two.
fn align_up(addr: usize, align: usize) -> usize {
    // this do the same thing:
    // let remainder = addr % align;
    // if remainder == 0 {
    //     addr // addr already aligned
    // } else {
    //     addr - remainder + align
    // }
    // i just don't fully understand this one yet
    (addr + align - 1) & !(align - 1)
}
