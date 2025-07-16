//! Hardware Abstraction Layer for STM32F446 Traffic Management System
//! 
//! This module provides low-level hardware interfaces for:
//! - GPIO pin control and configuration
//! - SysTick system timer for precise timing
//! - UART communication with interrupt-driven I/O
//! - Button input handling with debouncing
//! - Traffic light LED control

use core::ptr::{read_volatile, write_volatile};
use cortex_m::asm;

// =============================================================================
// GPIO REGISTER OFFSETS AND FUNCTIONALITY
// =============================================================================

/// GPIO mode register offset
pub const GPIO_MODE_REGISTER_OFFSET: u32 = 0x00;
/// GPIO output type register offset
pub const GPIO_OUTPUT_TYPE_REGISTER_OFFSET: u32 = 0x04;
/// GPIO output speed register offset
pub const GPIO_OUTPUT_SPEED_REGISTER_OFFSET: u32 = 0x08;
/// GPIO pull-up/pull-down register offset
pub const GPIO_PULL_UP_DOWN_REGISTER_OFFSET: u32 = 0x0C;
/// GPIO input data register offset
pub const GPIO_INPUT_DATA_REGISTER_OFFSET: u32 = 0x10;
/// GPIO bit set/reset register offset
pub const GPIO_BIT_SET_RESET_REGISTER_OFFSET: u32 = 0x18;

/// Enable specified GPIO port clocks in the RCC AHB1 enable register
pub unsafe fn enable_gpio_port_clocks(rcc_ahb1_enable_register_address: u32, port_enable_mask: u32) {
    let register_pointer = rcc_ahb1_enable_register_address as *mut u32;
    unsafe {
        write_volatile(register_pointer, read_volatile(register_pointer) | port_enable_mask);
    }
}

/// Configure a GPIO pin as a push-pull output with high speed
pub unsafe fn configure_pin_as_output(gpio_port_base_address: u32, pin_number: u8) {
    let mode_register = (gpio_port_base_address + GPIO_MODE_REGISTER_OFFSET) as *mut u32;
    let output_type_register = (gpio_port_base_address + GPIO_OUTPUT_TYPE_REGISTER_OFFSET) as *mut u32;
    let speed_register = (gpio_port_base_address + GPIO_OUTPUT_SPEED_REGISTER_OFFSET) as *mut u32;

    unsafe {
        // Set pin mode to output (01)
        let mut mode_value = read_volatile(mode_register);
        mode_value &= !(0b11 << (pin_number * 2));
        mode_value |= 0b01 << (pin_number * 2);
        write_volatile(mode_register, mode_value);

        // Set output type to push-pull (0)
        let mut output_type_value = read_volatile(output_type_register);
        output_type_value &= !(1 << pin_number);
        write_volatile(output_type_register, output_type_value);

        // Set output speed to high (10)
        let mut speed_value = read_volatile(speed_register);
        speed_value &= !(0b11 << (pin_number * 2));
        speed_value |= 0b10 << (pin_number * 2);
        write_volatile(speed_register, speed_value);
    }
}

/// Configure a GPIO pin as an input with internal pull-up resistor enabled
pub unsafe fn configure_pin_as_input_with_pullup(gpio_port_base_address: u32, pin_number: u8) {
    let mode_register = (gpio_port_base_address + GPIO_MODE_REGISTER_OFFSET) as *mut u32;
    let pull_up_down_register = (gpio_port_base_address + GPIO_PULL_UP_DOWN_REGISTER_OFFSET) as *mut u32;

    unsafe {
        // Set pin mode to input (00)
        let mut mode_value = read_volatile(mode_register);
        mode_value &= !(0b11 << (pin_number * 2));
        write_volatile(mode_register, mode_value);

        // Enable pull-up resistor (01)
        let mut pull_up_down_value = read_volatile(pull_up_down_register);
        pull_up_down_value &= !(0b11 << (pin_number * 2));
        pull_up_down_value |= 0b01 << (pin_number * 2);
        write_volatile(pull_up_down_register, pull_up_down_value);
    }
}

/// Set or clear a GPIO pin output using atomic bit set/reset register
pub unsafe fn set_gpio_pin_state(gpio_port_base_address: u32, pin_number: u8, pin_high: bool) {
    let bit_set_reset_register = (gpio_port_base_address + GPIO_BIT_SET_RESET_REGISTER_OFFSET) as *mut u32;
    
    unsafe {
        if pin_high {
            // Set bit (lower 16 bits)
            write_volatile(bit_set_reset_register, 1 << pin_number);
        } else {
            // Reset bit (upper 16 bits)
            write_volatile(bit_set_reset_register, 1 << (pin_number + 16));
        }
    }
}

/// Read the current state of a GPIO input pin
pub unsafe fn read_gpio_pin_state(gpio_port_base_address: u32, pin_number: u8) -> bool {
    let input_data_register = (gpio_port_base_address + GPIO_INPUT_DATA_REGISTER_OFFSET) as *const u32;
    unsafe {
        (read_volatile(input_data_register) & (1 << pin_number)) != 0
    }
}

/// Configure a GPIO pin for UART alternate function (AF7)
pub unsafe fn configure_pin_for_uart_alternate_function(gpio_port_base_address: u32, pin_number: u8) {
    const ALTERNATE_FUNCTION_LOW_REGISTER_OFFSET: u32 = 0x20; // Covers pins 0-7
    
    unsafe {
        // Set pin mode to alternate function (10)
        let mode_register = (gpio_port_base_address + GPIO_MODE_REGISTER_OFFSET) as *mut u32;
        let mut mode_value = read_volatile(mode_register);
        mode_value &= !(0b11 << (pin_number * 2));
        mode_value |= 0b10 << (pin_number * 2);
        write_volatile(mode_register, mode_value);

        // Set alternate function to AF7 (UART)
        let af_register = (gpio_port_base_address + ALTERNATE_FUNCTION_LOW_REGISTER_OFFSET) as *mut u32;
        let mut af_value = read_volatile(af_register);
        af_value &= !(0xF << (pin_number * 4));
        af_value |= 0x7 << (pin_number * 4);
        write_volatile(af_register, af_value);

        // Configure as push-pull output with very high speed
        let output_type_register = (gpio_port_base_address + GPIO_OUTPUT_TYPE_REGISTER_OFFSET) as *mut u32;
        let speed_register = (gpio_port_base_address + GPIO_OUTPUT_SPEED_REGISTER_OFFSET) as *mut u32;
        
        write_volatile(output_type_register, read_volatile(output_type_register) & !(1 << pin_number));
        write_volatile(speed_register, read_volatile(speed_register) | 0b11 << (pin_number * 2));
    }
}

// =============================================================================
// SYSTEM TIMER (SYSTICK) FUNCTIONALITY
// =============================================================================

/// SysTick Control and Status Register
const SYSTICK_CONTROL_STATUS_REGISTER: u32 = 0xE000_E010;
/// SysTick Reload Value Register
const SYSTICK_RELOAD_VALUE_REGISTER: u32 = 0xE000_E014;
/// SysTick Current Value Register
const SYSTICK_CURRENT_VALUE_REGISTER: u32 = 0xE000_E018;

/// Global millisecond counter updated by SysTick interrupt
static mut SYSTEM_MILLISECOND_COUNTER: u32 = 0;

/// Initialize SysTick timer for 1ms periodic interrupts
pub unsafe fn initialize_system_timer(system_clock_frequency_hz: u32) {
    // Calculate reload value for 1ms interrupts
    let reload_value = (system_clock_frequency_hz / 1000) - 1;

    unsafe {
        // Disable SysTick during configuration
        write_volatile(SYSTICK_CONTROL_STATUS_REGISTER as *mut u32, 0);

        // Set reload value for 1ms period
        write_volatile(SYSTICK_RELOAD_VALUE_REGISTER as *mut u32, reload_value);

        // Clear current value counter
        write_volatile(SYSTICK_CURRENT_VALUE_REGISTER as *mut u32, 0);

        // Enable SysTick with processor clock source and interrupt
        write_volatile(
            SYSTICK_CONTROL_STATUS_REGISTER as *mut u32,
            (1 << 0) |  // ENABLE - Enable SysTick counter
            (1 << 1) |  // TICKINT - Enable SysTick interrupt
            (1 << 2),   // CLKSOURCE - Use processor clock
        );
    }
}

/// Get the current system time in milliseconds since startup
pub fn get_current_system_time_ms() -> u32 {
    unsafe { SYSTEM_MILLISECOND_COUNTER }
}

/// Blocking delay for specified number of milliseconds
pub fn delay_milliseconds(duration_ms: u32) {
    let start_time = get_current_system_time_ms();
    while get_current_system_time_ms().wrapping_sub(start_time) < duration_ms {
        asm::nop();
    }
}

/// Delay with a polling function called during the wait period
pub fn delay_milliseconds_with_polling<F>(duration_ms: u32, mut poll_function: F)
where
    F: FnMut(),
{
    let start_time = get_current_system_time_ms();
    while get_current_system_time_ms().wrapping_sub(start_time) < duration_ms {
        poll_function();
        asm::nop();
    }
}

/// SysTick exception handler - automatically called every 1ms
#[cortex_m_rt::exception]
fn SysTick() {
    unsafe {
        SYSTEM_MILLISECOND_COUNTER = SYSTEM_MILLISECOND_COUNTER.wrapping_add(1);
    }
}

/// Get microsecond-precision timing (approximate, based on SysTick counter)
pub fn get_current_system_time_us() -> u32 {
    unsafe {
        let milliseconds = SYSTEM_MILLISECOND_COUNTER;
        let current_counter = read_volatile(SYSTICK_CURRENT_VALUE_REGISTER as *const u32);
        let reload_value = read_volatile(SYSTICK_RELOAD_VALUE_REGISTER as *const u32);

        // Calculate microseconds within current millisecond
        // Note: SysTick counts down, so we need to invert the calculation
        let microseconds_in_current_ms = ((reload_value - current_counter) * 1000) / (reload_value + 1);

        milliseconds * 1000 + microseconds_in_current_ms
    }
}

/// Reset the system timer (useful for testing and debugging)
pub unsafe fn reset_system_timer() {
    unsafe {
        SYSTEM_MILLISECOND_COUNTER = 0;
        write_volatile(SYSTICK_CURRENT_VALUE_REGISTER as *mut u32, 0);
    }
}

/// Check if SysTick timer is currently enabled
pub fn is_system_timer_enabled() -> bool {
    unsafe {
        let control_status = read_volatile(SYSTICK_CONTROL_STATUS_REGISTER as *const u32);
        (control_status & 1) != 0
    }
}

/// Get the current SysTick control and status register value
pub fn get_system_timer_control_status() -> u32 {
    unsafe { read_volatile(SYSTICK_CONTROL_STATUS_REGISTER as *const u32) }
}

// =============================================================================
// UART COMMUNICATION FUNCTIONALITY
// =============================================================================

/// Generic ring buffer for UART communication with power-of-2 size
pub struct CircularBuffer<const BUFFER_SIZE: usize> {
    buffer: [u8; BUFFER_SIZE],
    write_index: usize,
    read_index: usize,
}

impl<const BUFFER_SIZE: usize> CircularBuffer<BUFFER_SIZE> {
    /// Create a new empty circular buffer
    pub const fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            write_index: 0,
            read_index: 0,
        }
    }

    /// Add a byte to the buffer, returns false if buffer is full
    pub fn push_byte(&mut self, byte: u8) -> bool {
        let next_write_index = (self.write_index + 1) & (BUFFER_SIZE - 1);
        if next_write_index == self.read_index {
            return false; // Buffer is full
        }
        self.buffer[self.write_index] = byte;
        self.write_index = next_write_index;
        true
    }

    /// Remove and return a byte from the buffer, returns None if empty
    pub fn pop_byte(&mut self) -> Option<u8> {
        if self.write_index == self.read_index {
            return None; // Buffer is empty
        }
        let byte = self.buffer[self.read_index];
        self.read_index = (self.read_index + 1) & (BUFFER_SIZE - 1);
        Some(byte)
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.write_index == self.read_index
    }

    /// Get the number of bytes currently in the buffer
    pub fn get_byte_count(&self) -> usize {
        (self.write_index.wrapping_sub(self.read_index)) & (BUFFER_SIZE - 1)
    }

    /// Get the maximum capacity of the buffer
    pub fn get_capacity(&self) -> usize {
        BUFFER_SIZE - 1 // One slot is always kept empty to distinguish full from empty
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        ((self.write_index + 1) & (BUFFER_SIZE - 1)) == self.read_index
    }
}

/// UART peripheral driver with interrupt-driven communication
pub struct UartPeripheral<const PERIPHERAL_BASE_ADDRESS: u32> {
    pub transmit_buffer: CircularBuffer<256>,
    pub receive_buffer: CircularBuffer<256>,
}

impl<const BASE_ADDRESS: u32> UartPeripheral<BASE_ADDRESS> {
    /// UART Status Register offset
    const STATUS_REGISTER_OFFSET: u32 = 0x00;
    /// UART Data Register offset
    const DATA_REGISTER_OFFSET: u32 = 0x04;
    /// UART Baud Rate Register offset
    const BAUD_RATE_REGISTER_OFFSET: u32 = 0x08;
    /// UART Control Register 1 offset
    const CONTROL_REGISTER_1_OFFSET: u32 = 0x0C;
    /// UART Control Register 2 offset
    const CONTROL_REGISTER_2_OFFSET: u32 = 0x10;
    /// UART Control Register 3 offset
    const CONTROL_REGISTER_3_OFFSET: u32 = 0x14;

    /// Create a new UART peripheral instance
    pub const fn new() -> Self {
        Self {
            transmit_buffer: CircularBuffer::new(),
            receive_buffer: CircularBuffer::new(),
        }
    }

    /// Initialize UART with specified parameters
    pub unsafe fn initialize_uart(
        &self,
        peripheral_clock_hz: u32,
        baud_rate: u32,
        rcc_enable_register: *mut u32,
        enable_bit_mask: u32,
    ) {
        unsafe {
            // Enable peripheral clock
            write_volatile(rcc_enable_register, read_volatile(rcc_enable_register) | enable_bit_mask);

            // Disable UART during configuration
            write_volatile((BASE_ADDRESS + Self::CONTROL_REGISTER_1_OFFSET) as *mut u32, 0);

            // Calculate and set baud rate
            let baud_rate_divisor = peripheral_clock_hz / baud_rate;
            write_volatile((BASE_ADDRESS + Self::BAUD_RATE_REGISTER_OFFSET) as *mut u32, baud_rate_divisor);

            // Configure for 1 stop bit (default 00 in CR2)
            write_volatile((BASE_ADDRESS + Self::CONTROL_REGISTER_2_OFFSET) as *mut u32, 0);

            // Configure CR3 for no hardware flow control
            write_volatile((BASE_ADDRESS + Self::CONTROL_REGISTER_3_OFFSET) as *mut u32, 0);

            // Enable UART with RX interrupt, transmitter, receiver
            write_volatile(
                (BASE_ADDRESS + Self::CONTROL_REGISTER_1_OFFSET) as *mut u32,
                (1 << 5)  // RXNEIE - RX not empty interrupt enable
                | (1 << 3)  // TE - Transmitter enable  
                | (1 << 2)  // RE - Receiver enable
                | (1 << 13), // UE - UART enable
            );
        }
    }

    /// Queue a string for transmission
    pub fn transmit_string(&mut self, text: &str) {
        for byte in text.bytes() {
            self.transmit_buffer.push_byte(byte);
        }
        // Enable TX empty interrupt to start transmission
        self.enable_transmit_interrupt();
    }

    /// Queue a single byte for transmission
    pub fn transmit_byte(&mut self, byte: u8) {
        self.transmit_buffer.push_byte(byte);
        self.enable_transmit_interrupt();
    }

    /// Try to read a received byte (non-blocking)
    pub fn try_receive_byte(&mut self) -> Option<u8> {
        self.receive_buffer.pop_byte()
    }

    /// Check if transmit buffer is empty
    

    /// Check if receive buffer has data available
    pub fn is_receive_data_available(&self) -> bool {
        !self.receive_buffer.is_empty()
    }
pub fn is_transmit_buffer_empty(&self) -> bool {
        self.transmit_buffer.is_empty()
    }
    /// Enable transmit empty interrupt
    fn enable_transmit_interrupt(&self) {
        unsafe {
            let control_register_1 = (BASE_ADDRESS + Self::CONTROL_REGISTER_1_OFFSET) as *mut u32;
            write_volatile(control_register_1, read_volatile(control_register_1) | (1 << 7)); // TXEIE
        }
    }

    /// UART interrupt service routine - must be called from interrupt handlers
    pub unsafe fn handle_uart_interrupt(&mut self) {
        unsafe {
            let status_register_value = read_volatile((BASE_ADDRESS + Self::STATUS_REGISTER_OFFSET) as *const u32);

            // Handle received data interrupt
            if (status_register_value & (1 << 5)) != 0 {
                // RXNE - Receive data register not empty
                let received_byte = read_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *const u32) as u8;
                self.receive_buffer.push_byte(received_byte);
            }

            // Handle transmit empty interrupt
            if (status_register_value & (1 << 7)) != 0 {
                // TXE - Transmit data register empty
                if let Some(byte_to_send) = self.transmit_buffer.pop_byte() {
                    write_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *mut u32, byte_to_send as u32);
                } else {
                    // No more bytes to send: disable TXEIE interrupt
                    let control_register_1 = (BASE_ADDRESS + Self::CONTROL_REGISTER_1_OFFSET) as *mut u32;
                    write_volatile(control_register_1, read_volatile(control_register_1) & !(1 << 7));
                }
            }

            // Handle and clear error flags
            if (status_register_value & (1 << 3)) != 0 {
                // ORE - Overrun error
                let _ = read_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *const u32);
            }

            if (status_register_value & (1 << 1)) != 0 {
                // FE - Framing error
                let _ = read_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *const u32);
            }

            if (status_register_value & (1 << 2)) != 0 {
                // NF - Noise detected flag
                let _ = read_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *const u32);
            }
        }
    }

    /// Get current UART status register value (for debugging)
    pub unsafe fn get_uart_status(&self) -> u32 {
        unsafe {
            read_volatile((BASE_ADDRESS + Self::STATUS_REGISTER_OFFSET) as *const u32)
        }
    }

    /// Clear UART error flags
    pub unsafe fn clear_uart_errors(&self) {
        unsafe {
            // Reading data register clears ORE, FE, NF flags
            let _ = read_volatile((BASE_ADDRESS + Self::DATA_REGISTER_OFFSET) as *const u32);
        }
    }
}

impl<const BASE_ADDRESS: u32> core::fmt::Write for UartPeripheral<BASE_ADDRESS> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for byte in text.bytes() {
            self.transmit_buffer.push_byte(byte);
        }
        self.enable_transmit_interrupt();
        Ok(())
    }
}

// =============================================================================
// BUTTON INPUT HANDLING
// =============================================================================

/// Button debounce time in milliseconds
const BUTTON_DEBOUNCE_TIME_MS: u32 = 50;

/// Internal state of a single button
#[derive(Copy, Clone)]
pub struct ButtonInputState {
    pub is_currently_pressed: bool,
    pub last_state_change_time_ms: u32,
}

/// Button events after debouncing
#[derive(PartialEq, Debug)]
pub enum ButtonEvent {
    NoStateChange,
    ButtonPressed,
    ButtonReleased,
}

/// Multi-button input handler with debouncing
pub struct ButtonInputController {
    pub button_a_state: ButtonInputState,
    pub button_b_state: ButtonInputState,
    pub button_a_pin: u8,
    pub button_b_pin: u8,
    pub gpio_port_base: u32,
}

impl ButtonInputController {
    /// Create a new button input controller
    pub fn new(button_a_pin: u8, button_b_pin: u8, gpio_port_base: u32) -> Self {
        ButtonInputController {
            button_a_state: ButtonInputState {
                is_currently_pressed: false,
                last_state_change_time_ms: 0,
            },
            button_b_state: ButtonInputState {
                is_currently_pressed: false,
                last_state_change_time_ms: 0,
            },
            button_a_pin,
            button_b_pin,
            gpio_port_base,
        }
    }

    /// Poll both buttons and return their events
    pub fn poll_button_inputs(&mut self, current_time_ms: u32) -> (ButtonEvent, ButtonEvent) {
        unsafe {
            let input_data_register_value = read_volatile((self.gpio_port_base + GPIO_INPUT_DATA_REGISTER_OFFSET) as *const u32);
            
            // Buttons are active-low, so invert the reading
            let button_a_pressed = (input_data_register_value & (1 << self.button_a_pin)) == 0;
            let button_b_pressed = (input_data_register_value & (1 << self.button_b_pin)) == 0;

            let mut button_a_event = ButtonEvent::NoStateChange;
            let mut button_b_event = ButtonEvent::NoStateChange;

            // Debounce button A
            if current_time_ms.wrapping_sub(self.button_a_state.last_state_change_time_ms) >= BUTTON_DEBOUNCE_TIME_MS {
                if button_a_pressed != self.button_a_state.is_currently_pressed {
                    self.button_a_state.is_currently_pressed = button_a_pressed;
                    self.button_a_state.last_state_change_time_ms = current_time_ms;
                    button_a_event = if button_a_pressed {
                        ButtonEvent::ButtonPressed
                    } else {
                        ButtonEvent::ButtonReleased
                    };
                }
            }

            // Debounce button B
            if current_time_ms.wrapping_sub(self.button_b_state.last_state_change_time_ms) >= BUTTON_DEBOUNCE_TIME_MS {
                if button_b_pressed != self.button_b_state.is_currently_pressed {
                    self.button_b_state.is_currently_pressed = button_b_pressed;
                    self.button_b_state.last_state_change_time_ms = current_time_ms;
                    button_b_event = if button_b_pressed {
                        ButtonEvent::ButtonPressed
                    } else {
                        ButtonEvent::ButtonReleased
                    };
                }
            }

            (button_a_event, button_b_event)
        }
    }
}

// =============================================================================
// TRAFFIC LIGHT LED CONTROL
// =============================================================================

/// Traffic light hardware abstraction
pub struct TrafficLightController {
    // Traffic light pins for road A (East-West)
    pub green_a: u8,
    pub yellow_a: u8,
    pub red_a: u8,

    // Traffic light pins for road B (North-South)
    pub green_b: u8,
    pub yellow_b: u8,
    pub red_b: u8,

    // Traffic level indicator pins for road A
    pub lvl1_a: u8,
    pub lvl2_a: u8,
    pub lvl3_a: u8,

    // Traffic level indicator pins for road B
    pub lvl1_b: u8,
    pub lvl2_b: u8,
    pub lvl3_b: u8,

    // GPIO port base addresses
    pub port_a: u32,
    pub port_b: u32,
}

impl TrafficLightController {
    /// Control road A traffic lights (East-West direction)
    pub unsafe fn set_road_a_traffic_lights(&self, green_on: bool, yellow_on: bool, red_on: bool) {
        unsafe {
            set_gpio_pin_state(self.port_a, self.green_a, green_on);
            set_gpio_pin_state(self.port_a, self.yellow_a, yellow_on);
            set_gpio_pin_state(self.port_a, self.red_a, red_on);
        }
    }

    /// Control road B traffic lights (North-South direction)
    pub unsafe fn set_road_b_traffic_lights(&self, green_on: bool, yellow_on: bool, red_on: bool) {
        unsafe {
            set_gpio_pin_state(self.port_b, self.green_b, green_on);
            set_gpio_pin_state(self.port_b, self.yellow_b, yellow_on);
            set_gpio_pin_state(self.port_b, self.red_b, red_on);
        }
    }

    /// Update traffic level indicator bar graphs for both roads
    pub unsafe fn update_traffic_level_indicators(&self, road_a_level: u8, road_b_level: u8) {
        unsafe {
            // Road A level indicators
            set_gpio_pin_state(self.port_a, self.lvl1_a, road_a_level >= 1);
            set_gpio_pin_state(self.port_a, self.lvl2_a, road_a_level >= 2);
            set_gpio_pin_state(self.port_a, self.lvl3_a, road_a_level >= 3);

            // Road B level indicators
            set_gpio_pin_state(self.port_b, self.lvl1_b, road_b_level >= 1);
            set_gpio_pin_state(self.port_b, self.lvl2_b, road_b_level >= 2);
            set_gpio_pin_state(self.port_b, self.lvl3_b, road_b_level >= 3);
        }
    }

    /// Backward compatibility: set road A traffic lights (alias)
    pub unsafe fn set_pair_a(&self, green_on: bool, yellow_on: bool, red_on: bool) {
        unsafe {
            self.set_road_a_traffic_lights(green_on, yellow_on, red_on);
        }
    }

    /// Backward compatibility: set road B traffic lights (alias)
    pub unsafe fn set_pair_b(&self, green_on: bool, yellow_on: bool, red_on: bool) {
        unsafe {
            self.set_road_b_traffic_lights(green_on, yellow_on, red_on);
        }
    }

    /// Backward compatibility: update bargraphs (alias)
    pub unsafe fn update_bargraphs(&self, road_a_level: u8, road_b_level: u8) {
        unsafe {
            self.update_traffic_level_indicators(road_a_level, road_b_level);
        }
    }
}

// Type aliases for backward compatibility
pub use enable_gpio_port_clocks as enable_gpio_clocks;
pub use configure_pin_as_output as pin_to_output;
pub use configure_pin_as_input_with_pullup as pin_to_input_pullup;
pub use configure_pin_for_uart_alternate_function as pin_to_af7_usart;
pub use initialize_system_timer as systick_init;
pub use get_current_system_time_ms as get_system_ms;
pub use delay_milliseconds as delay_ms;
pub use UartPeripheral as Uart;
pub use CircularBuffer as Ring;
pub use ButtonInputController as Buttons;
pub use ButtonInputState as ButtonState;
pub use TrafficLightController as TrafficLights;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer_basic_operations() {
        let mut buffer: CircularBuffer<8> = CircularBuffer::new();

        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.get_byte_count(), 0);

        // Add some data
        assert!(buffer.push_byte(1));
        assert!(buffer.push_byte(2));
        assert!(buffer.push_byte(3));

        assert!(!buffer.is_empty());
        assert_eq!(buffer.get_byte_count(), 3);

        // Remove some data
        assert_eq!(buffer.pop_byte(), Some(1));
        assert_eq!(buffer.pop_byte(), Some(2));
        assert_eq!(buffer.get_byte_count(), 1);

        assert_eq!(buffer.pop_byte(), Some(3));
        assert_eq!(buffer.pop_byte(), None);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_circular_buffer_wraparound() {
        let mut buffer: CircularBuffer<4> = CircularBuffer::new(); // Capacity is 3 (N-1)

        // Fill to capacity
        assert!(buffer.push_byte(1));
        assert!(buffer.push_byte(2));
        assert!(buffer.push_byte(3));
        assert!(buffer.is_full());
        assert!(!buffer.push_byte(4)); // Should fail when full

        // Remove one and add one
        assert_eq!(buffer.pop_byte(), Some(1));
        assert!(buffer.push_byte(4));
        assert!(buffer.is_full());

        // Verify correct order
        assert_eq!(buffer.pop_byte(), Some(2));
        assert_eq!(buffer.pop_byte(), Some(3));
        assert_eq!(buffer.pop_byte(), Some(4));
        assert_eq!(buffer.pop_byte(), None);
    }
} 