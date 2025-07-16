use core::fmt::Write;
use crate::hardware::*;

/// Configuration for a single traffic light timing
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrafficLightConfig {
    pub green_duration_ms: u32,
    pub yellow_duration_ms: u32,
    pub red_duration_ms: u32,
    pub extension_duration_ms: u32, // Additional time for heavy traffic
}

impl TrafficLightConfig {
    pub const fn default() -> Self {
        Self {
            green_duration_ms: 10_000,
            yellow_duration_ms: 2_000,
            red_duration_ms: 10_000,
            extension_duration_ms: 5_000,
        }
    }

    /// Create a new configuration with specified durations in seconds
    pub const fn new(green_s: u32, yellow_s: u32, red_s: u32, extension_s: u32) -> Self {
        Self {
            green_duration_ms: green_s * 1000,
            yellow_duration_ms: yellow_s * 1000,
            red_duration_ms: red_s * 1000,
            extension_duration_ms: extension_s * 1000,
        }
    }
}

/// Configuration for traffic monitoring
#[derive(Copy, Clone, Debug)]
pub struct MonitorConfig {
    pub report_interval_ms: u32,
}

impl MonitorConfig {
    pub const fn default() -> Self {
        Self {
            report_interval_ms: 30_000,
        }
    }

    /// Create monitor config with interval in seconds
    pub const fn with_interval_seconds(seconds: u32) -> Self {
        Self {
            report_interval_ms: seconds * 1000,
        }
    }
}

/// Complete system configuration
#[derive(Copy, Clone, Debug)]
pub struct SystemConfiguration {
    pub traffic_light_1: TrafficLightConfig,
    pub traffic_light_2: TrafficLightConfig,
    pub monitor: MonitorConfig,
}

impl SystemConfiguration {
    pub const fn default() -> Self {
        Self {
            traffic_light_1: TrafficLightConfig::default(),
            traffic_light_2: TrafficLightConfig::default(),
            monitor: MonitorConfig::default(),
        }
    }
}

/// Traffic light phases in the intersection cycle
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrafficPhase {
    /// Road A (East-West) green, Road B (North-South) red
    RoadAGreenRoadBRed,
    /// Road A yellow, Road B red (transition)
    RoadAYellowRoadBRed,
    /// Road A red, Road B green
    RoadARedRoadBGreen,
    /// Road A red, Road B yellow (transition)
    RoadARedRoadBYellow,
}

impl TrafficPhase {
    /// Get the next phase in the cycle
    pub fn next_phase(self) -> Self {
        match self {
            Self::RoadAGreenRoadBRed => Self::RoadAYellowRoadBRed,
            Self::RoadAYellowRoadBRed => Self::RoadARedRoadBGreen,
            Self::RoadARedRoadBGreen => Self::RoadARedRoadBYellow,
            Self::RoadARedRoadBYellow => Self::RoadAGreenRoadBRed,
        }
    }

    /// Get human-readable description
    pub fn description(self) -> &'static str {
        match self {
            Self::RoadAGreenRoadBRed => "EW Green, NS Red",
            Self::RoadAYellowRoadBRed => "EW Yellow, NS Red",
            Self::RoadARedRoadBGreen => "EW Red, NS Green",
            Self::RoadARedRoadBYellow => "EW Red, NS Yellow",
        }
    }
}

/// Traffic levels (0-3) representing traffic density
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrafficLevel(u8);

impl TrafficLevel {
    pub const NONE: Self = Self(0);
    pub const LIGHT: Self = Self(1);
    pub const MODERATE: Self = Self(2);
    pub const HEAVY: Self = Self(3);

    pub fn new(level: u8) -> Self {
        Self(level.min(3))
    }

    pub fn get(self) -> u8 {
        self.0
    }

    pub fn increment(self) -> Self {
        Self((self.0 + 1) % 4)
    }

    pub fn is_heavy_traffic(self) -> bool {
        self.0 >= 2
    }

    pub fn description(self) -> &'static str {
        match self.0 {
            0 => "No Traffic",
            1 => "Light Traffic",
            2 => "Moderate Traffic",
            3 => "Heavy Traffic",
            _ => "Unknown",
        }
    }
}

/// Current state of the traffic system
pub struct TrafficSystemState {
    pub road_a_traffic_level: TrafficLevel,
    pub road_b_traffic_level: TrafficLevel,
    pub current_phase: TrafficPhase,
    pub phase_start_time_ms: u32,
}

impl TrafficSystemState {
    pub const fn new() -> Self {
        Self {
            road_a_traffic_level: TrafficLevel::NONE,
            road_b_traffic_level: TrafficLevel::NONE,
            current_phase: TrafficPhase::RoadAGreenRoadBRed,
            phase_start_time_ms: 0,
        }
    }

    /// Increment traffic level for road A
    pub fn increment_road_a_traffic(&mut self) {
        self.road_a_traffic_level = self.road_a_traffic_level.increment();
    }

    /// Increment traffic level for road B
    pub fn increment_road_b_traffic(&mut self) {
        self.road_b_traffic_level = self.road_b_traffic_level.increment();
    }

    /// Start a new phase timing
    pub fn start_new_phase(&mut self, phase: TrafficPhase, current_time_ms: u32) {
        self.current_phase = phase;
        self.phase_start_time_ms = current_time_ms;
    }

    /// Get elapsed time since phase started
    pub fn time_since_phase_start(&self, current_time_ms: u32) -> u32 {
        current_time_ms.wrapping_sub(self.phase_start_time_ms)
    }
}

/// LED state tracking for status reporting
#[derive(Default, Debug)]
pub struct LedStates {
    pub road_a_green: bool,
    pub road_a_yellow: bool,
    pub road_a_red: bool,
    pub road_b_green: bool,
    pub road_b_yellow: bool,
    pub road_b_red: bool,
}

/// Main traffic controller managing the entire system
pub struct TrafficController {
    pub config: SystemConfiguration,
    pub state: TrafficSystemState,
    pub led_states: LedStates,
}

impl TrafficController {
    pub const fn new() -> Self {
        Self {
            config: SystemConfiguration::default(),
            state: TrafficSystemState::new(),
            led_states: LedStates {
                road_a_green: false,
                road_a_yellow: false,
                road_a_red: false,
                road_b_green: false,
                road_b_yellow: false,
                road_b_red: false,
            },
        }
    }

    /// Update traffic light configuration
    pub fn update_traffic_light_config(&mut self, light_index: u8, new_config: TrafficLightConfig) -> bool {
        match light_index {
            1 => {
                self.config.traffic_light_1 = new_config;
                true
            }
            2 => {
                self.config.traffic_light_2 = new_config;
                true
            }
            _ => false,
        }
    }

    /// Update monitoring configuration
    pub fn update_monitor_config(&mut self, interval_ms: u32) {
        self.config.monitor.report_interval_ms = interval_ms;
    }

    /// Calculate green light duration based on traffic level
    pub fn calculate_green_duration(&self, traffic_level: TrafficLevel) -> u32 {
        let base_duration = self.config.traffic_light_1.green_duration_ms;
        let extension = self.config.traffic_light_1.extension_duration_ms;

        match traffic_level.get() {
            0 => base_duration,
            1 => base_duration + extension / 3,
            2 => base_duration + (extension * 2) / 3,
            3 => base_duration + extension,
            _ => base_duration,
        }
    }

    /// Update physical traffic lights and track LED states
    pub fn update_traffic_lights(&mut self, lights: &TrafficLights, road_a: (bool, bool, bool), road_b: (bool, bool, bool)) {
        let (ag, ay, ar) = road_a;
        let (bg, by, br) = road_b;

        unsafe {
            lights.set_road_a_traffic_lights(ag, ay, ar);
            lights.set_road_b_traffic_lights(bg, by, br);
        }

        // Update state tracking
        self.led_states.road_a_green = ag;
        self.led_states.road_a_yellow = ay;
        self.led_states.road_a_red = ar;
        self.led_states.road_b_green = bg;
        self.led_states.road_b_yellow = by;
        self.led_states.road_b_red = br;
    }

    /// Main state machine update - returns true if phase changed
    pub fn update_traffic_state_machine(&mut self, traffic_lights: &TrafficLights, current_time_ms: u32) -> bool {
        let elapsed = self.state.time_since_phase_start(current_time_ms);
        let mut phase_changed = false;

        let next_phase = match self.state.current_phase {
            TrafficPhase::RoadAGreenRoadBRed => {
                self.update_traffic_lights(traffic_lights, (true, false, false), (false, false, true));
                let required_duration = self.calculate_green_duration(self.state.road_a_traffic_level);
                if elapsed >= required_duration {
                    Some(TrafficPhase::RoadAYellowRoadBRed)
                } else {
                    None
                }
            }
            TrafficPhase::RoadAYellowRoadBRed => {
                self.update_traffic_lights(traffic_lights, (false, true, false), (false, false, true));
                if elapsed >= self.config.traffic_light_1.yellow_duration_ms {
                    Some(TrafficPhase::RoadARedRoadBGreen)
                } else {
                    None
                }
            }
            TrafficPhase::RoadARedRoadBGreen => {
                self.update_traffic_lights(traffic_lights, (false, false, true), (true, false, false));
                let required_duration = self.calculate_green_duration(self.state.road_b_traffic_level);
                if elapsed >= required_duration {
                    Some(TrafficPhase::RoadARedRoadBYellow)
                } else {
                    None
                }
            }
            TrafficPhase::RoadARedRoadBYellow => {
                self.update_traffic_lights(traffic_lights, (false, false, true), (false, true, false));
                if elapsed >= self.config.traffic_light_2.yellow_duration_ms {
                    Some(TrafficPhase::RoadAGreenRoadBRed)
                } else {
                    None
                }
            }
        };

        if let Some(next) = next_phase {
            self.state.start_new_phase(next, current_time_ms);
            phase_changed = true;
        }

        phase_changed
    }

    /// Format configuration for display
    pub fn format_configuration(&self, light_index: Option<u8>, output: &mut heapless::String<256>) {
        match light_index {
            Some(1) => {
                let _ = core::write!(
                    output,
                    "traffic light 1 G Y R {} {} {} {}\r\n",
                    self.config.traffic_light_1.green_duration_ms / 1000,
                    self.config.traffic_light_1.yellow_duration_ms / 1000,
                    self.config.traffic_light_1.red_duration_ms / 1000,
                    self.config.traffic_light_1.extension_duration_ms / 1000
                );
            }
            Some(2) => {
                let _ = core::write!(
                    output,
                    "traffic light 2 G Y R {} {} {} {}\r\n",
                    self.config.traffic_light_2.green_duration_ms / 1000,
                    self.config.traffic_light_2.yellow_duration_ms / 1000,
                    self.config.traffic_light_2.red_duration_ms / 1000,
                    self.config.traffic_light_2.extension_duration_ms / 1000
                );
            }
            Some(_) => {
                // Invalid traffic light index - ignore
            }
            None => {
                // Display all configurations
                self.format_configuration(Some(1), output);
                self.format_configuration(Some(2), output);
                let _ = core::write!(
                    output,
                    "traffic monitor {}\r\n",
                    self.config.monitor.report_interval_ms / 1000
                );
            }
        }
    }

    /// Generate comprehensive status report
    pub fn generate_status_report<W: Write>(&self, writer: &mut W, current_time_ms: u32) {
        let timestamp_s = current_time_ms / 1000;

        // ANSI color constants
        const CYAN: &str = "\x1B[36m";
        const GREEN: &str = "\x1B[32m";
        const RED: &str = "\x1B[31m";
        const YELLOW: &str = "\x1B[33m";
        const RESET: &str = "\x1B[0m";

        // Status line 1: Traffic Light 1 (East-West)
        let _ = write!(writer, "{}{}{}traffic light 1 (EW)  ", CYAN, timestamp_s, RESET);
        self.write_led_status(writer, self.led_states.road_a_green);
        self.write_led_status(writer, self.led_states.road_a_yellow);
        self.write_led_status(writer, self.led_states.road_a_red);
        let _ = write!(writer, "\r\n");

        // Status line 2: Traffic Light 2 (North-South)
        let _ = write!(writer, "{}{}{}traffic light 2 (NS)  ", CYAN, timestamp_s, RESET);
        self.write_led_status(writer, self.led_states.road_b_green);
        self.write_led_status(writer, self.led_states.road_b_yellow);
        self.write_led_status(writer, self.led_states.road_b_red);
        let _ = write!(writer, "\r\n");

        // Status line 3: Traffic levels
        let _ = write!(
            writer,
            "{}{} traffic levels NS={} EW={}{}\r\n",
            CYAN,
            timestamp_s,
            self.state.road_a_traffic_level.get(),
            self.state.road_b_traffic_level.get(),
            RESET
        );

        // Status lines 4 & 5: Traffic descriptions
        let _ = write!(
            writer,
            "{}{} NS direction: {}{}\r\n",
            YELLOW,
            timestamp_s,
            self.state.road_a_traffic_level.description(),
            RESET
        );

        let _ = write!(
            writer,
            "{}{} EW direction: {}{}\r\n",
            YELLOW,
            timestamp_s,
            self.state.road_b_traffic_level.description(),
            RESET
        );
    }

    /// Helper to write LED status with color coding
    fn write_led_status<W: Write>(&self, writer: &mut W, is_on: bool) {
        const GREEN: &str = "\x1B[32m";
        const RED: &str = "\x1B[31m";
        
        let color = if is_on { GREEN } else { RED };
        let status = if is_on { "ON" } else { "OFF" };
        let _ = write!(writer, "{}{} ", color, status);
    }
} 