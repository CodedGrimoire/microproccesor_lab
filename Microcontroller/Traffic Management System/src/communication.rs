use crate::hardware::Uart;
use crate::command_processor::CommandProcessor;
use crate::traffic_control::TrafficController;

/// Serial communication handler managing console input and monitoring
pub struct SerialCommunicationHandler {
    command_buffer: heapless::String<128>,
    last_status_report_time_ms: u32,
}

impl SerialCommunicationHandler {
    /// Create a new serial communication handler
    pub const fn new() -> Self {
        Self {
            command_buffer: heapless::String::new(),
            last_status_report_time_ms: 0,
        }
    }

    /// Main polling function for serial communication
    /// 
    /// Handles:
    /// - Command input from console (UART2)
    /// - Data relay from downstream to PC (UARTB -> UART2)
    /// - Periodic status monitoring reports
    pub fn poll_serial_interfaces<const UART2_BASE: u32, const UARTA_BASE: u32, const UARTB_BASE: u32>(
        &mut self,
        current_time_ms: u32,
        traffic_controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
        downstream_uart: &mut Uart<UARTB_BASE>,
    ) {
        // Process incoming commands from PC console
        self.process_console_input(traffic_controller, console_uart, upstream_uart);

        // Relay data from downstream system to PC console
        self.relay_downstream_to_console(console_uart, downstream_uart);

        // Send periodic status reports if interval has elapsed
        self.handle_periodic_monitoring(current_time_ms, traffic_controller, console_uart);
    }

    /// Process incoming commands from the console UART
    fn process_console_input<const UART2_BASE: u32, const UARTA_BASE: u32>(
        &mut self,
        traffic_controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
    ) {
        while let Some(byte) = console_uart.try_receive_byte() {
            match byte {
                b'\r' | b'\n' => {
                    self.handle_command_line_completion(traffic_controller, console_uart, upstream_uart);
                }
                0x08 | 0x7F => {
                    self.handle_backspace(console_uart);
                }
                b if Self::is_printable_character(b) => {
                    self.handle_printable_character(b, console_uart);
                }
                _ => {
                    // Ignore other control characters
                }
            }
        }
    }

    /// Handle command line completion (Enter key pressed)
    fn handle_command_line_completion<const UART2_BASE: u32, const UARTA_BASE: u32>(
        &mut self,
        traffic_controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
    ) {
        // Echo newline
        console_uart.transmit_string("\r\n");

        // Process command if buffer is not empty
        if !self.command_buffer.is_empty() {
            let command = CommandProcessor::parse_command_line(&self.command_buffer);
            CommandProcessor::execute_command(command, traffic_controller, console_uart, upstream_uart);
            self.command_buffer.clear();
        }
    }

    /// Handle backspace/delete character
    fn handle_backspace<const UART2_BASE: u32>(&mut self, console_uart: &mut Uart<UART2_BASE>) {
        if self.command_buffer.pop().is_some() {
            // Echo backspace sequence: move cursor back, space, move back again
            console_uart.transmit_string("\x08 \x08");
        }
    }

    /// Handle printable characters (add to buffer and echo)
    fn handle_printable_character<const UART2_BASE: u32>(&mut self, character: u8, console_uart: &mut Uart<UART2_BASE>) {
        if self.command_buffer.push(character as char).is_ok() {
            console_uart.transmit_byte(character);
        }
        // If buffer is full, silently ignore additional characters
    }

    /// Check if a character is printable (visible ASCII or space)
    fn is_printable_character(byte: u8) -> bool {
        byte.is_ascii_graphic() || byte == b' '
    }

    /// Relay data from downstream UART to console
    fn relay_downstream_to_console<const UART2_BASE: u32, const UARTB_BASE: u32>(
        &self,
        console_uart: &mut Uart<UART2_BASE>,
        downstream_uart: &mut Uart<UARTB_BASE>,
    ) {
        while let Some(byte) = downstream_uart.try_receive_byte() {
            console_uart.transmit_byte(byte);
        }
    }

    /// Handle periodic monitoring reports
    fn handle_periodic_monitoring<const UART2_BASE: u32>(
        &mut self,
        current_time_ms: u32,
        traffic_controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
    ) {
        let time_since_last_report = current_time_ms.wrapping_sub(self.last_status_report_time_ms);
        let report_interval = traffic_controller.config.monitor.report_interval_ms;

        if time_since_last_report >= report_interval {
            self.last_status_report_time_ms = current_time_ms;
            traffic_controller.generate_status_report(console_uart, current_time_ms);
        }
    }

    /// Get the current command buffer content (useful for debugging)
    pub fn get_command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Check if command buffer is empty
    pub fn is_command_buffer_empty(&self) -> bool {
        self.command_buffer.is_empty()
    }

    /// Get remaining capacity in command buffer
    pub fn get_command_buffer_remaining_capacity(&self) -> usize {
        self.command_buffer.capacity() - self.command_buffer.len()
    }
} 