#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f4::stm32f446::interrupt;

mod hardware;
mod traffic_control;
mod command_processor;
mod communication;
mod system;

use system::TrafficManagementSystem;

/// Main entry point for the embedded traffic management system
#[entry]
fn main() -> ! {
    // Create the traffic management system instance
    let mut traffic_system = TrafficManagementSystem::new();
    
    // Initialize all hardware subsystems
    traffic_system.initialize_hardware_subsystems();
    
    // Start the traffic control system
    traffic_system.start_traffic_system();
    
    // Run the main system loop indefinitely
    traffic_system.run_system_main_loop()
}

// =============================================================================
// INTERRUPT SERVICE ROUTINES
// =============================================================================

/// USART1 interrupt handler (upstream communication)
#[interrupt]
fn USART1() {
    unsafe {
        TrafficManagementSystem::handle_upstream_uart_interrupt();
    }
}

/// USART2 interrupt handler (PC console communication)
#[interrupt]
fn USART2() {
    unsafe {
        TrafficManagementSystem::handle_console_uart_interrupt();
    }
}

/// USART3 interrupt handler (downstream communication)
#[interrupt]
fn USART3() {
    unsafe {
        TrafficManagementSystem::handle_downstream_uart_interrupt();
    }
}

/// Global panic handler for embedded environment
/// 
/// In case of a panic, this will trigger a breakpoint for debugging.
/// In a production environment, this might log the error or perform
/// a system reset instead.
#[panic_handler]
fn panic_handler(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {
        // Trigger breakpoint for debugging
        cortex_m::asm::bkpt();
    }
}
