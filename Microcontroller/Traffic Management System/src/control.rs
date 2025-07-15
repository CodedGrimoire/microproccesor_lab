#![allow(clippy::upper_case_acronyms)]
use core::fmt::Write;
use heapless::String;

#[derive(Copy, Clone)]
pub struct TLConfig {
    pub g_ms: u32,
    pub y_ms: u32,
    pub r_ms: u32,
    pub u_ms: u32, // light-traffic extension
}

impl TLConfig {
    pub const fn new() -> Self {
        Self {
            g_ms: 10_000,
            y_ms: 2_000,
            r_ms: 10_000,
            u_ms: 5_000,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MonitorCfg {
    pub interval_ms: u32,
}

impl MonitorCfg {
    pub const fn new() -> Self {
        Self {
            interval_ms: 30_000,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SystemConfig {
    pub tl1: TLConfig,
    pub tl2: TLConfig,
    pub monitor: MonitorCfg,
}

impl SystemConfig {
    pub const fn new() -> Self {
        Self {
            tl1: TLConfig::new(),
            tl2: TLConfig::new(),
            monitor: MonitorCfg::new(),
        }
    }
}

pub enum Cmd {
    SetTL(u8, TLConfig), // traffic light x
    SetMon(u32),         // monitor X
    ReadTL(Option<u8>),  // read traffic light [x]
    ReadMon,             // read traffic monitor
    ReadAll,
    Invalid,
}

/// Quick-n-dirty tokenizer; assumes ASCII & space-delimited.
pub fn parse(line: &str) -> Cmd {
    let mut tok = line.split_ascii_whitespace();
    match tok.next() {
        Some("config") => match tok.next() {
            Some("traffic") => match tok.next() {
                Some("light") => {
                    let idx = tok.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                    if idx == 0 || idx > 2 {
                        return Cmd::Invalid;
                    }

                    // Skip the G Y R literals
                    let _g = tok.next().unwrap_or("");
                    let _y = tok.next().unwrap_or("");
                    let _r = tok.next().unwrap_or("");

                    // Parse the actual timing values
                    let g = tok.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) * 1000;
                    let y = tok.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) * 1000;
                    let r = tok.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) * 1000;
                    let u = tok.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) * 1000;

                    if g == 0 || y == 0 || r == 0 {
                        return Cmd::Invalid;
                    }

                    return Cmd::SetTL(
                        idx,
                        TLConfig {
                            g_ms: g,
                            y_ms: y,
                            r_ms: r,
                            u_ms: u,
                        },
                    );
                }
                Some("monitor") => {
                    let x = tok.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                    if x == 0 {
                        return Cmd::Invalid;
                    }
                    return Cmd::SetMon(x * 1000);
                }
                _ => {}
            },
            _ => {}
        },
        Some("read") => match tok.next() {
            None => return Cmd::ReadAll,
            Some("traffic") => match tok.next() {
                Some("light") => {
                    let idx = tok.next().and_then(|s| s.parse::<u8>().ok());
                    if let Some(i) = idx {
                        if i > 2 {
                            return Cmd::Invalid;
                        }
                    }
                    return Cmd::ReadTL(idx);
                }
                Some("monitor") => return Cmd::ReadMon,
                _ => {}
            },
            Some("all") => return Cmd::ReadAll,
            _ => {}
        },
        _ => {}
    };
    Cmd::Invalid
}

/// Helpers that format current configuration or live status lines
pub fn format_cfg(cfg: &SystemConfig, tl: Option<u8>, out: &mut String<256>) {
    match tl {
        Some(1) => {
            let _ = core::write!(
                out,
                "traffic light 1 G Y R {} {} {} {}\r\n",
                cfg.tl1.g_ms / 1000,
                cfg.tl1.y_ms / 1000,
                cfg.tl1.r_ms / 1000,
                cfg.tl1.u_ms / 1000
            );
        }
        Some(2) => {
            let _ = core::write!(
                out,
                "traffic light 2 G Y R {} {} {} {}\r\n",
                cfg.tl2.g_ms / 1000,
                cfg.tl2.y_ms / 1000,
                cfg.tl2.r_ms / 1000,
                cfg.tl2.u_ms / 1000
            );
        }
        Some(_) => {
            // Ignore invalid traffic light numbers
        }
        None => {
            format_cfg(cfg, Some(1), out);
            format_cfg(cfg, Some(2), out);
            let _ = core::write!(
                out,
                "traffic monitor {}\r\n",
                cfg.monitor.interval_ms / 1000
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_traffic_light() {
        match parse("config traffic light 1 G Y R 8 2 8 4") {
            Cmd::SetTL(idx, cfg) => {
                assert_eq!(idx, 1);
                assert_eq!(cfg.g_ms, 8000);
                assert_eq!(cfg.y_ms, 2000);
                assert_eq!(cfg.r_ms, 8000);
                assert_eq!(cfg.u_ms, 4000);
            }
            _ => panic!("Failed to parse valid command"),
        }
    }

    #[test]
    fn test_parse_config_monitor() {
        match parse("config traffic monitor 15") {
            Cmd::SetMon(ms) => assert_eq!(ms, 15000),
            _ => panic!("Failed to parse valid command"),
        }
    }

    #[test]
    fn test_parse_read_commands() {
        assert!(matches!(parse("read"), Cmd::ReadAll));
        assert!(matches!(parse("read all"), Cmd::ReadAll));
        assert!(matches!(
            parse("read traffic light 1"),
            Cmd::ReadTL(Some(1))
        ));
        assert!(matches!(parse("read traffic light"), Cmd::ReadTL(None)));
        assert!(matches!(parse("read traffic monitor"), Cmd::ReadMon));
    }

    #[test]
    fn test_parse_invalid_commands() {
        assert!(matches!(parse("invalid command"), Cmd::Invalid));
        assert!(matches!(
            parse("config traffic light 0 G Y R 8 2 8 4"),
            Cmd::Invalid
        ));
        assert!(matches!(
            parse("config traffic light 3 G Y R 8 2 8 4"),
            Cmd::Invalid
        ));
        assert!(matches!(parse("config traffic monitor 0"), Cmd::Invalid));
    }
}
