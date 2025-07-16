use core::fmt::Write;
use crate::traffic_control::{TrafficLightConfig, TrafficController};
use crate::hardware::Uart;

/// Supported command types for traffic management
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Set traffic light configuration: (light_id, config)
    SetTrafficLight(u8, TrafficLightConfig),
    /// Set monitoring interval in milliseconds
    SetMonitorInterval(u32),
    /// Read traffic light configuration: Some(light_id) or None for all
    ReadTrafficLight(Option<u8>),
    /// Read monitoring configuration
    ReadMonitorConfig,
    /// Read all system configuration
    ReadAllConfig,
    /// Invalid or malformed command
    Invalid,
}

/// Command processor for parsing and executing traffic management commands
pub struct CommandProcessor;

impl CommandProcessor {
    /// Parse a command line string into a Command enum
    /// 
    /// Supported commands:
    /// - `config traffic light <id> G Y R <green> <yellow> <red> <extension>`
    /// - `config traffic monitor <seconds>`
    /// - `read traffic light [id]`
    /// - `read traffic monitor`
    /// - `read all`
    /// - `read` (alias for `read all`)
    pub fn parse_command_line(input: &str) -> Command {
        let mut tokens = input.trim().split_ascii_whitespace();
        
        match tokens.next() {
            Some("config") => Self::parse_config_command(tokens),
            Some("read") => Self::parse_read_command(tokens),
            _ => Command::Invalid,
        }
    }

    /// Parse configuration commands
    fn parse_config_command(mut tokens: core::str::SplitAsciiWhitespace) -> Command {
        match tokens.next() {
            Some("traffic") => match tokens.next() {
                Some("light") => Self::parse_traffic_light_config(tokens),
                Some("monitor") => Self::parse_monitor_config(tokens),
                _ => Command::Invalid,
            },
            _ => Command::Invalid,
        }
    }

    /// Parse traffic light configuration command
    fn parse_traffic_light_config(mut tokens: core::str::SplitAsciiWhitespace) -> Command {
        // Expected: <light_id> G Y R <green_s> <yellow_s> <red_s> <extension_s>
        let light_id = match tokens.next().and_then(|s| s.parse::<u8>().ok()) {
            Some(id) if id >= 1 && id <= 2 => id,
            _ => return Command::Invalid,
        };

        // Skip the literal tokens G Y R
        if !matches!((tokens.next(), tokens.next(), tokens.next()), 
                    (Some("G"), Some("Y"), Some("R"))) {
            return Command::Invalid;
        }

        // Parse timing values (in seconds)
        let green_s = tokens.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let yellow_s = tokens.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let red_s = tokens.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let extension_s = tokens.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

        // Validate that all required values are non-zero
        if green_s == 0 || yellow_s == 0 || red_s == 0 {
            return Command::Invalid;
        }

        let config = TrafficLightConfig::new(green_s, yellow_s, red_s, extension_s);
        Command::SetTrafficLight(light_id, config)
    }

    /// Parse monitor configuration command
    fn parse_monitor_config(mut tokens: core::str::SplitAsciiWhitespace) -> Command {
        // Expected: <interval_seconds>
        match tokens.next().and_then(|s| s.parse::<u32>().ok()) {
            Some(seconds) if seconds > 0 => Command::SetMonitorInterval(seconds * 1000),
            _ => Command::Invalid,
        }
    }

    /// Parse read commands
    fn parse_read_command(mut tokens: core::str::SplitAsciiWhitespace) -> Command {
        match tokens.next() {
            None => Command::ReadAllConfig, // Just "read" defaults to read all
            Some("all") => Command::ReadAllConfig,
            Some("traffic") => match tokens.next() {
                Some("light") => {
                    // Optional light ID
                    match tokens.next().and_then(|s| s.parse::<u8>().ok()) {
                        Some(id) if id >= 1 && id <= 2 => Command::ReadTrafficLight(Some(id)),
                        Some(_) => Command::Invalid, // Invalid light ID
                        None => Command::ReadTrafficLight(None), // Read all lights
                    }
                }
                Some("monitor") => Command::ReadMonitorConfig,
                _ => Command::Invalid,
            },
            _ => Command::Invalid,
        }
    }

    /// Execute a parsed command
    pub fn execute_command<const UART2_BASE: u32, const UARTA_BASE: u32>(
        command: Command,
        controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
    ) {
        match command {
            Command::SetTrafficLight(light_id, config) => {
                Self::handle_set_traffic_light(light_id, config, controller, console_uart, upstream_uart);
            }
            Command::SetMonitorInterval(interval_ms) => {
                Self::handle_set_monitor_interval(interval_ms, controller, console_uart, upstream_uart);
            }
            Command::ReadTrafficLight(light_id) => {
                Self::handle_read_traffic_light(light_id, controller, console_uart);
            }
            Command::ReadMonitorConfig => {
                Self::handle_read_monitor_config(controller, console_uart);
            }
            Command::ReadAllConfig => {
                Self::handle_read_all_config(controller, console_uart);
            }
            Command::Invalid => {
                Self::send_error_response(console_uart, "Invalid command format");
            }
        }
    }

    /// Handle traffic light configuration command
    fn handle_set_traffic_light<const UART2_BASE: u32, const UARTA_BASE: u32>(
        light_id: u8,
        config: TrafficLightConfig,
        controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
    ) {
        if controller.update_traffic_light_config(light_id, config) {
            Self::send_success_response(console_uart);
            Self::forward_traffic_light_config_to_upstream(light_id, &config, upstream_uart);
        } else {
            Self::send_error_response(console_uart, "Invalid traffic light ID");
        }
    }

    /// Handle monitor interval configuration
    fn handle_set_monitor_interval<const UART2_BASE: u32, const UARTA_BASE: u32>(
        interval_ms: u32,
        controller: &mut TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
        upstream_uart: &mut Uart<UARTA_BASE>,
    ) {
        controller.update_monitor_config(interval_ms);
        Self::send_success_response(console_uart);
        Self::forward_monitor_config_to_upstream(interval_ms, upstream_uart);
    }

    /// Handle read traffic light configuration
    fn handle_read_traffic_light<const UART2_BASE: u32>(
        light_id: Option<u8>,
        controller: &TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
    ) {
        let mut response_buffer: heapless::String<256> = heapless::String::new();
        controller.format_configuration(light_id, &mut response_buffer);
        console_uart.transmit_string(&response_buffer);
    }

    /// Handle read monitor configuration
    fn handle_read_monitor_config<const UART2_BASE: u32>(
        controller: &TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
    ) {
        let mut response: heapless::String<64> = heapless::String::new();
        let _ = write!(
            response,
            "traffic monitor {}\r\n",
            controller.config.monitor.report_interval_ms / 1000
        );
        console_uart.transmit_string(&response);
    }

    /// Handle read all configuration
    fn handle_read_all_config<const UART2_BASE: u32>(
        controller: &TrafficController,
        console_uart: &mut Uart<UART2_BASE>,
    ) {
        let mut response_buffer: heapless::String<256> = heapless::String::new();
        controller.format_configuration(None, &mut response_buffer);
        console_uart.transmit_string(&response_buffer);
    }

    /// Send success response with green color
    fn send_success_response<const UART_BASE: u32>(uart: &mut Uart<UART_BASE>) {
        const GREEN: &str = "\x1B[32m";
        const RESET: &str = "\x1B[0m";
        uart.transmit_string(GREEN);
        uart.transmit_string("OK\r\n");
        uart.transmit_string(RESET);
    }

    /// Send error response with red color
    fn send_error_response<const UART_BASE: u32>(uart: &mut Uart<UART_BASE>, message: &str) {
        const RED: &str = "\x1B[31m";
        const RESET: &str = "\x1B[0m";
        uart.transmit_string(RED);
        uart.transmit_string("ERR: ");
        uart.transmit_string(message);
        uart.transmit_string("\r\n");
        uart.transmit_string(RESET);
    }

    /// Forward traffic light configuration to upstream system
    fn forward_traffic_light_config_to_upstream<const UART_BASE: u32>(
        light_id: u8,
        config: &TrafficLightConfig,
        upstream_uart: &mut Uart<UART_BASE>,
    ) {
        let mut command_string: heapless::String<128> = heapless::String::new();
        let _ = write!(
            command_string,
            "config traffic light {} G Y R {} {} {} {}\r\n",
            light_id,
            config.green_duration_ms / 1000,
            config.yellow_duration_ms / 1000,
            config.red_duration_ms / 1000,
            config.extension_duration_ms / 1000
        );
        upstream_uart.transmit_string(&command_string);
    }

    /// Forward monitor configuration to upstream system
    fn forward_monitor_config_to_upstream<const UART_BASE: u32>(
        interval_ms: u32,
        upstream_uart: &mut Uart<UART_BASE>,
    ) {
        let mut command_string: heapless::String<64> = heapless::String::new();
        let _ = write!(command_string, "config traffic monitor {}\r\n", interval_ms / 1000);
        upstream_uart.transmit_string(&command_string);
    }
} 