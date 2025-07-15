use core::ptr::{read_volatile, write_volatile};
use cortex_m::asm;

// SysTick registers
const SYST_CSR: u32 = 0xE000_E010;
const SYST_RVR: u32 = 0xE000_E014;
const SYST_CVR: u32 = 0xE000_E018;

static mut SYSTEM_MS: u32 = 0;

/// Initialize SysTick timer for 1ms interrupts
pub unsafe fn systick_init(sysclk_hz: u32) {
    let reload_value = (sysclk_hz / 1000) - 1; // 1ms period

    // Disable SysTick first
    write_volatile(SYST_CSR as *mut u32, 0);

    // Set reload value
    write_volatile(SYST_RVR as *mut u32, reload_value);

    // Clear current value
    write_volatile(SYST_CVR as *mut u32, 0);

    // Enable SysTick with processor clock and interrupt
    write_volatile(
        SYST_CSR as *mut u32,
        (1 << 0) |  // ENABLE
        (1 << 1) |  // TICKINT - Enable interrupt
        (1 << 2), // CLKSOURCE - Use processor clock
    );
}

/// Get current system time in milliseconds
pub fn get_system_ms() -> u32 {
    unsafe { SYSTEM_MS }
}

/// Simple delay function (busy wait)
pub fn delay_ms(ms: u32) {
    let start = get_system_ms();
    while get_system_ms().wrapping_sub(start) < ms {
        asm::nop();
    }
}

/// Delay with polling function called during wait
pub fn delay_ms_poll<F>(ms: u32, mut poll_fn: F)
where
    F: FnMut(),
{
    let start = get_system_ms();
    while get_system_ms().wrapping_sub(start) < ms {
        poll_fn();
        asm::nop();
    }
}

/// SysTick interrupt handler - must be registered in main.rs
/// Note: SysTick is a core exception, not a device interrupt
#[cortex_m_rt::exception]
fn SysTick() {
    unsafe {
        SYSTEM_MS = SYSTEM_MS.wrapping_add(1);
    }
}

/// Get microsecond precision timing (approximate)
pub fn get_system_us() -> u32 {
    unsafe {
        let ms = SYSTEM_MS;
        let current = read_volatile(SYST_CVR as *const u32);
        let reload = read_volatile(SYST_RVR as *const u32);

        // Calculate microseconds within current millisecond
        // Note: SysTick counts down, so we need to invert
        let us_in_ms = ((reload - current) * 1000) / (reload + 1);

        ms * 1000 + us_in_ms
    }
}

/// Reset system timer (useful for testing)
pub unsafe fn reset_system_time() {
    SYSTEM_MS = 0;
    write_volatile(SYST_CVR as *mut u32, 0);
}

/// Check if SysTick is enabled and running
pub fn is_systick_enabled() -> bool {
    unsafe {
        let csr = read_volatile(SYST_CSR as *const u32);
        (csr & 1) != 0
    }
}

/// Get the SysTick control and status register value
pub fn get_systick_csr() -> u32 {
    unsafe { read_volatile(SYST_CSR as *const u32) }
}
