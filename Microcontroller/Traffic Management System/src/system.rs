use core::ptr::{addr_of, addr_of_mut};
use crate::hardware::*;
use crate::traffic_control::TrafficController;
use crate::communication::SerialCommunicationHandler;

// =============================================================================
// HARDWARE CONFIGURATION CONSTANTS
// =============================================================================

// RCC (Reset and Clock Control) register addresses
const RCC_AHB1_ENABLE_REGISTER: u32 = 0x4002_3800 + 0x30;
const RCC_APB1_ENABLE_REGISTER: u32 = 0x4002_3800 + 0x40;
const RCC_APB2_ENABLE_REGISTER: u32 = 0x4002_3800 + 0x44;

// GPIO port base addresses
const GPIO_PORT_A_BASE: u32 = 0x4002_0000;
const GPIO_PORT_B_BASE: u32 = 0x4002_0400;
const GPIO_PORT_C_BASE: u32 = 0x4002_0800;

// UART peripheral base addresses
const USART1_BASE_ADDRESS: u32 = 0x4001_1000; // APB2 - Upstream communication
const USART2_BASE_ADDRESS: u32 = 0x4000_4400; // APB1 - PC console
const USART3_BASE_ADDRESS: u32 = 0x4000_4800; // APB1 - Downstream communication

// =============================================================================
// PIN ASSIGNMENTS
// =============================================================================

// Traffic light LED pins - Road A (East-West)
const ROAD_A_GREEN_LED_PIN: u8 = 5;   // PA5
const ROAD_A_YELLOW_LED_PIN: u8 = 6;  // PA6
const ROAD_A_RED_LED_PIN: u8 = 7;     // PA7

// Traffic light LED pins - Road B (North-South)
const ROAD_B_GREEN_LED_PIN: u8 = 0;   // PB0
const ROAD_B_YELLOW_LED_PIN: u8 = 1;  // PB1
const ROAD_B_RED_LED_PIN: u8 = 2;     // PB2

// Traffic level indicator LED pins - Road A
const ROAD_A_LEVEL_1_LED_PIN: u8 = 8;  // PA8
const ROAD_A_LEVEL_2_LED_PIN: u8 = 9;  // PA9
const ROAD_A_LEVEL_3_LED_PIN: u8 = 10; // PA10

// Traffic level indicator LED pins - Road B
const ROAD_B_LEVEL_1_LED_PIN: u8 = 6;  // PB6
const ROAD_B_LEVEL_2_LED_PIN: u8 = 7;  // PB7
const ROAD_B_LEVEL_3_LED_PIN: u8 = 8;  // PB8

// Push button pins (active-low)
const ROAD_A_BUTTON_PIN: u8 = 13; // PC13 - Road A traffic level button
const ROAD_B_BUTTON_PIN: u8 = 0;  // PC0  - Road B traffic level button

// =============================================================================
// SYSTEM CONFIGURATION
// =============================================================================

const SYSTEM_CLOCK_FREQUENCY_HZ: u32 = 16_000_000;

// =============================================================================
// GLOBAL SYSTEM STATE
// =============================================================================

// Global instances (managed by SystemManager)
static mut TRAFFIC_CONTROLLER: TrafficController = TrafficController::new();
static mut SERIAL_HANDLER: SerialCommunicationHandler = SerialCommunicationHandler::new();
static mut UPSTREAM_UART: Uart<USART1_BASE_ADDRESS> = Uart::new();
static mut CONSOLE_UART: Uart<USART2_BASE_ADDRESS> = Uart::new();
static mut DOWNSTREAM_UART: Uart<USART3_BASE_ADDRESS> = Uart::new();

/// System manager coordinating all hardware and software components
pub struct TrafficManagementSystem {
    pub button_controller: Buttons,
    pub traffic_light_hardware: TrafficLights,
}

impl TrafficManagementSystem {
    /// Create a new traffic management system instance
    pub fn new() -> Self {
        Self {
            button_controller: Buttons::new(
                ROAD_A_BUTTON_PIN,
                ROAD_B_BUTTON_PIN,
                GPIO_PORT_C_BASE,
            ),
            traffic_light_hardware: TrafficLights {
                green_a: ROAD_A_GREEN_LED_PIN,
                yellow_a: ROAD_A_YELLOW_LED_PIN,
                red_a: ROAD_A_RED_LED_PIN,
                green_b: ROAD_B_GREEN_LED_PIN,
                yellow_b: ROAD_B_YELLOW_LED_PIN,
                red_b: ROAD_B_RED_LED_PIN,
                lvl1_a: ROAD_A_LEVEL_1_LED_PIN,
                lvl2_a: ROAD_A_LEVEL_2_LED_PIN,
                lvl3_a: ROAD_A_LEVEL_3_LED_PIN,
                lvl1_b: ROAD_B_LEVEL_1_LED_PIN,
                lvl2_b: ROAD_B_LEVEL_2_LED_PIN,
                lvl3_b: ROAD_B_LEVEL_3_LED_PIN,
                port_a: GPIO_PORT_A_BASE,
                port_b: GPIO_PORT_B_BASE,
            },
        }
    }

    /// Initialize all hardware components
    pub fn initialize_hardware_subsystems(&self) {
        unsafe {
            self.enable_peripheral_clocks();
            self.configure_gpio_pins();
            self.initialize_system_timer();
            self.configure_uart_pins();
            self.initialize_uart_peripherals();
            self.enable_interrupt_controllers();
        }
    }

    /// Enable necessary peripheral clocks
    unsafe fn enable_peripheral_clocks(&self) {
        unsafe {
            // Enable GPIO clocks for ports A, B, and C
            let gpio_clock_mask = (1 << 0) | (1 << 1) | (1 << 2); // GPIOA, GPIOB, GPIOC
            enable_gpio_clocks(RCC_AHB1_ENABLE_REGISTER, gpio_clock_mask);
        }
    }

    /// Configure all GPIO pins
    unsafe fn configure_gpio_pins(&self) {
        unsafe {
            // Configure LED pins as outputs
            let led_pins = [
                (GPIO_PORT_A_BASE, ROAD_A_GREEN_LED_PIN),
                (GPIO_PORT_A_BASE, ROAD_A_YELLOW_LED_PIN),
                (GPIO_PORT_A_BASE, ROAD_A_RED_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_GREEN_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_YELLOW_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_RED_LED_PIN),
                (GPIO_PORT_A_BASE, ROAD_A_LEVEL_1_LED_PIN),
                (GPIO_PORT_A_BASE, ROAD_A_LEVEL_2_LED_PIN),
                (GPIO_PORT_A_BASE, ROAD_A_LEVEL_3_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_LEVEL_1_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_LEVEL_2_LED_PIN),
                (GPIO_PORT_B_BASE, ROAD_B_LEVEL_3_LED_PIN),
            ];

            for &(port, pin) in &led_pins {
                pin_to_output(port, pin);
            }

            // Configure button pins as inputs with pull-up resistors
            pin_to_input_pullup(GPIO_PORT_C_BASE, ROAD_A_BUTTON_PIN);
            pin_to_input_pullup(GPIO_PORT_C_BASE, ROAD_B_BUTTON_PIN);
        }
    }

    /// Initialize system timer for millisecond timing
    unsafe fn initialize_system_timer(&self) {
        unsafe {
            systick_init(SYSTEM_CLOCK_FREQUENCY_HZ);
        }
    }

    /// Configure UART communication pins
    unsafe fn configure_uart_pins(&self) {
        unsafe {
            // Configure USART2 pins: PA2 (TX), PA3 (RX)
            pin_to_af7_usart(GPIO_PORT_A_BASE, 2); // PA2 → USART2_TX
            pin_to_af7_usart(GPIO_PORT_A_BASE, 3); // PA3 → USART2_RX
        }
    }

    /// Initialize UART peripherals
    unsafe fn initialize_uart_peripherals(&self) {
        let baud_rate = 115_200;
        let clock_frequency = 16_000_000;

        unsafe {
            // UART2 (PC console) - APB1 bus
            (*addr_of_mut!(CONSOLE_UART)).initialize_uart(
                clock_frequency,
                baud_rate,
                RCC_APB1_ENABLE_REGISTER as *mut u32,
                1 << 17, // USART2EN bit
            );

            // USART1 (upstream communication) - APB2 bus
            (*addr_of_mut!(UPSTREAM_UART)).initialize_uart(
                clock_frequency,
                baud_rate,
                RCC_APB2_ENABLE_REGISTER as *mut u32,
                1 << 4, // USART1EN bit
            );

            // USART3 (downstream communication) - APB1 bus
            (*addr_of_mut!(DOWNSTREAM_UART)).initialize_uart(
                clock_frequency,
                baud_rate,
                RCC_APB1_ENABLE_REGISTER as *mut u32,
                1 << 18, // USART3EN bit
            );
        }
    }

    /// Enable UART interrupt controllers
    unsafe fn enable_interrupt_controllers(&self) {
        use cortex_m::peripheral::NVIC;
        use stm32f4::stm32f446::Interrupt;

        unsafe {
            NVIC::unmask(Interrupt::USART1);
            NVIC::unmask(Interrupt::USART2);
            NVIC::unmask(Interrupt::USART3);
        }
    }

    /// Start the traffic management system
    pub fn start_traffic_system(&mut self) {
        unsafe {
            // Initialize traffic level indicators
            let controller = &*addr_of!(TRAFFIC_CONTROLLER);
            self.traffic_light_hardware.update_bargraphs(
                controller.state.road_a_traffic_level.get(),
                controller.state.road_b_traffic_level.get(),
            );

            // Initialize traffic light timing
            let current_time = get_system_ms();
            (*addr_of_mut!(TRAFFIC_CONTROLLER)).state.start_new_phase(
                controller.state.current_phase,
                current_time,
            );

            // Send system startup message
            self.send_startup_message();
        }
    }

    /// Send colorized startup message to console
    unsafe fn send_startup_message(&self) {
        const GREEN: &str = "\x1B[32m";
        const RESET: &str = "\x1B[0m";

        unsafe {
            (*addr_of_mut!(CONSOLE_UART)).transmit_string(GREEN);
            (*addr_of_mut!(CONSOLE_UART)).transmit_string("Traffic Management System Initialized\r\n");
            (*addr_of_mut!(CONSOLE_UART)).transmit_string("Ready for commands...\r\n");
            (*addr_of_mut!(CONSOLE_UART)).transmit_string(RESET);
        }
    }

    /// Main system loop - runs indefinitely
    pub fn run_system_main_loop(&mut self) -> ! {
        loop {
            let current_time_ms = get_system_ms();

            // Handle user button interactions
            self.process_button_events(current_time_ms);

            // Handle serial communication
            self.process_serial_communication(current_time_ms);

            // Update traffic light state machine
            self.update_traffic_control_logic(current_time_ms);

            // Small delay to prevent CPU overload
            delay_ms(1);
        }
    }

    /// Process button events for traffic level adjustment
    fn process_button_events(&mut self, current_time_ms: u32) {
        let (road_a_event, road_b_event) = self.button_controller.poll_button_inputs(current_time_ms);

        unsafe {
            let controller = &mut *addr_of_mut!(TRAFFIC_CONTROLLER);

            if road_a_event == ButtonEvent::ButtonReleased {
                controller.state.increment_road_a_traffic();
                self.update_traffic_level_indicators(controller);
            }

            if road_b_event == ButtonEvent::ButtonReleased {
                controller.state.increment_road_b_traffic();
                self.update_traffic_level_indicators(controller);
            }
        }
    }

    /// Update traffic level indicator LEDs
    fn update_traffic_level_indicators(&self, controller: &TrafficController) {
        unsafe {
            self.traffic_light_hardware.update_bargraphs(
                controller.state.road_a_traffic_level.get(),
                controller.state.road_b_traffic_level.get(),
            );
        }
    }

    /// Process all serial communication
    fn process_serial_communication(&mut self, current_time_ms: u32) {
        unsafe {
            (*addr_of_mut!(SERIAL_HANDLER)).poll_serial_interfaces(
                current_time_ms,
                &mut *addr_of_mut!(TRAFFIC_CONTROLLER),
                &mut *addr_of_mut!(CONSOLE_UART),
                &mut *addr_of_mut!(UPSTREAM_UART),
                &mut *addr_of_mut!(DOWNSTREAM_UART),
            );
        }
    }

    /// Update traffic control state machine
    fn update_traffic_control_logic(&mut self, current_time_ms: u32) {
        unsafe {
            let controller = &mut *addr_of_mut!(TRAFFIC_CONTROLLER);
            let _phase_changed = controller.update_traffic_state_machine(
                &self.traffic_light_hardware,
                current_time_ms,
            );
            // Could add phase change logging or other actions here if needed
        }
    }

    // =============================================================================
    // INTERRUPT HANDLERS
    // =============================================================================

    /// Handle USART1 (upstream) interrupt
    pub unsafe fn handle_upstream_uart_interrupt() {
        unsafe {
            (*addr_of_mut!(UPSTREAM_UART)).handle_uart_interrupt();
        }
    }

    /// Handle USART2 (console) interrupt
    pub unsafe fn handle_console_uart_interrupt() {
        unsafe {
            (*addr_of_mut!(CONSOLE_UART)).handle_uart_interrupt();
        }
    }

    /// Handle USART3 (downstream) interrupt
    pub unsafe fn handle_downstream_uart_interrupt() {
        unsafe {
            (*addr_of_mut!(DOWNSTREAM_UART)).handle_uart_interrupt();
        }
    }
} 