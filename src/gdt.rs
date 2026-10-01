use x86_64::structures::{gdt, tss};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

const STACK_SIZE: usize = 4096 * 5;

#[allow(unused)]
#[repr(align(16))]
struct GdtStack([u8; STACK_SIZE]);

lazy_static::lazy_static! {
    static ref TSS: tss::TaskStateSegment =  {
        let mut tss = tss::TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut DOUBLE_FAULT_STACK: GdtStack = GdtStack([0; STACK_SIZE]);
            let stack_start = x86_64::VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK);
            stack_start + STACK_SIZE as u64
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut PAGE_FAULT_STACK: GdtStack = GdtStack([0; STACK_SIZE]);
            let stack_start = x86_64::VirtAddr::from_ptr(&raw const PAGE_FAULT_STACK);
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

lazy_static::lazy_static! {
    static ref GDT: (gdt::GlobalDescriptorTable, Selectors) = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        let code_selector = gdt.append(gdt::Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(gdt::Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::registers::segmentation::{CS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
