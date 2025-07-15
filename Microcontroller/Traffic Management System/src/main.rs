#![no_std]
#![no_main]

use core::fmt::Write;
use core::ptr::{addr_of, addr_of_mut};
use cortex_m_rt::entry;

mod button;
mod control;
mod gpio;
mod timer;
mod traffic;
mod uart;

use button::*;
use control::*;
use gpio::*;
use stm32f4::stm32f446::interrupt;
use timer::*;
use traffic::*;
use uart::Uart;

const RCC_AHB1ENR: u32 = 0x4002_3800 + 0x30;
const RCC_APB1ENR: u32 = 0x4002_3800 + 0x40;
const RCC_APB2ENR: u32 = 0x4002_3800 + 0x44;

const GPIOA_BASE: u32 = 0x4002_0000;
const GPIOB_BASE: u32 = 0x4002_0400;
const GPIOC_BASE: u32 = 0x4002_0800;

// UART base addresses
const USART1_BASE: u32 = 0x4001_1000; // APB2 - UARTa (upstream)
const USART2_BASE: u32 = 0x4000_4400; // APB1 - PC console
const USART3_BASE: u32 = 0x4000_4800; // APB1 - UARTb (downstream)

// Pins for traffic lights & buttons
const A_GREEN: u8 = 0;
const A_YELLOW: u8 = 1;
const A_RED: u8 = 4;

const B_GREEN: u8 = 12;
const B_YELLOW: u8 = 2;
const B_RED: u8 = 1;

const LVL1_A: u8 = 5;
const LVL2_A: u8 = 6;
const LVL3_A: u8 = 7;

const LVL1_B: u8 = 13;
const LVL2_B: u8 = 14;
const LVL3_B: u8 = 15;

const BTN_A_PIN: u8 = 13; // PC13
const BTN_B_PIN: u8 = 0; // PC0

const SYSCLK_HZ: u32 = 16_000_000;

const ANSI_RESET: &str = "\x1B[0m";
const ANSI_RED: &str = "\x1B[31m";
const ANSI_GREEN: &str = "\x1B[32m";
const ANSI_YELLOW: &str = "\x1B[33m";
const ANSI_CYAN: &str = "\x1B[36m";

struct TrafficState {
    level_a: u8,
    level_b: u8,
    state: TrafficPhase,
    phase_start_ms: u32,
}

#[derive(Copy, Clone)]
enum TrafficPhase {
    AGreenBRed,
    AYellowBRed,
    ARedBGreen,
    ARedBYellow,
}

// Static globals for interrupt access
static mut TRAFFIC: TrafficState = TrafficState {
    level_a: 0,
    level_b: 0,
    state: TrafficPhase::AGreenBRed,
    phase_start_ms: 0,
};

static mut UARTA: Uart<USART1_BASE> = Uart::new(); // Upstream
static mut UART2: Uart<USART2_BASE> = Uart::new(); // PC console
static mut UARTB: Uart<USART3_BASE> = Uart::new(); // Downstream
static mut CFG: SystemConfig = SystemConfig::new();

// Line buffer for command parsing
static mut CMD_LINE: heapless::String<128> = heapless::String::new();

// LED state tracking for monitoring
static mut LED_STATE_A_GREEN: bool = false;
static mut LED_STATE_A_YELLOW: bool = false;
static mut LED_STATE_A_RED: bool = false;
static mut LED_STATE_B_GREEN: bool = false;
static mut LED_STATE_B_YELLOW: bool = false;
static mut LED_STATE_B_RED: bool = false;

#[entry]
fn main() -> ! {
    unsafe {
        // Enable GPIO clocks for A, B, C
        enable_gpio_clocks(RCC_AHB1ENR, (1 << 0) | (1 << 1) | (1 << 2));
    }

    // Configure LEDs as outputs
    unsafe {
        for &(port, pin) in &[
            (GPIOA_BASE, A_GREEN),
            (GPIOA_BASE, A_YELLOW),
            (GPIOA_BASE, A_RED),
            (GPIOB_BASE, B_GREEN),
            (GPIOB_BASE, B_YELLOW),
            (GPIOB_BASE, B_RED),
            (GPIOA_BASE, LVL1_A),
            (GPIOA_BASE, LVL2_A),
            (GPIOA_BASE, LVL3_A),
            (GPIOB_BASE, LVL1_B),
            (GPIOB_BASE, LVL2_B),
            (GPIOB_BASE, LVL3_B),
        ] {
            pin_to_output(port, pin);
        }

        // Configure buttons as inputs with pull-up
        pin_to_input_pullup(GPIOC_BASE, BTN_A_PIN);
        pin_to_input_pullup(GPIOC_BASE, BTN_B_PIN);
    }

    // Initialize timer
    unsafe {
        systick_init(SYSCLK_HZ);
    }

    unsafe {
        // PA2 → USART2_TX , PA3 → USART2_RX  (AF7, push‑pull, high‑speed)
        pin_to_af7_usart(GPIOA_BASE, 2);
        pin_to_af7_usart(GPIOA_BASE, 3);
    }

    // Initialize UARTs
    unsafe {
        // UART2 (PC console) - APB1, 16MHz
        (*addr_of_mut!(UART2)).init(16_000_000, 115_200, RCC_APB1ENR as *mut u32, 1 << 17);

        // USART1 (UARTa upstream) - APB2, 16MHz
        (*addr_of_mut!(UARTA)).init(16_000_000, 115_200, RCC_APB2ENR as *mut u32, 1 << 4);

        // USART3 (UARTb downstream) - APB1, 16MHz
        (*addr_of_mut!(UARTB)).init(16_000_000, 115_200, RCC_APB1ENR as *mut u32, 1 << 18);

        use cortex_m::peripheral::NVIC;
        use stm32f4::stm32f446::Interrupt;
        NVIC::unmask(Interrupt::USART1);
        NVIC::unmask(Interrupt::USART2);
        NVIC::unmask(Interrupt::USART3);
    }

    // Enable UART interrupts in NVIC - done automatically by interrupt handlers

    let mut buttons = Buttons::new(BTN_A_PIN, BTN_B_PIN, GPIOC_BASE);
    let traffic_lights = TrafficLights {
        green_a: A_GREEN,
        yellow_a: A_YELLOW,
        red_a: A_RED,
        green_b: B_GREEN,
        yellow_b: B_YELLOW,
        red_b: B_RED,
        lvl1_a: LVL1_A,
        lvl2_a: LVL2_A,
        lvl3_a: LVL3_A,
        lvl1_b: LVL1_B,
        lvl2_b: LVL2_B,
        lvl3_b: LVL3_B,
        port_a: GPIOA_BASE,
        port_b: GPIOB_BASE,
    };

    unsafe {
        traffic_lights.update_bargraphs((*addr_of!(TRAFFIC)).level_a, (*addr_of!(TRAFFIC)).level_b);
        (*addr_of_mut!(TRAFFIC)).phase_start_ms = get_system_ms();

        // Send startup message
        (*addr_of_mut!(UART2)).write_str(ANSI_GREEN);
        (*addr_of_mut!(UART2)).write_str("Traffic Control System Ready\r\n");
        (*addr_of_mut!(UART2)).write_str(ANSI_RESET);
        // {
        //     use core::ptr::{read_volatile, write_volatile};
        //     const DR: *mut u32 = (USART2_BASE + 0x04) as *mut u32;
        //     if let Some(b) = (*addr_of_mut!(UART2)).tx_buf.pop() {
        //         write_volatile(DR, b as u32); // first byte kicked out
        //     }
        //     let cr1 = (USART2_BASE + 0x0C) as *mut u32;
        //     write_volatile(cr1, read_volatile(cr1) | (1 << 7)); // set TXEIE
        // }
    }

    loop {
        let now = get_system_ms();

        // Handle button events
        let (evt_a, evt_b) = buttons.poll(now);
        if evt_a == ButtonEvent::Released {
            unsafe {
                (*addr_of_mut!(TRAFFIC)).level_a = ((*addr_of!(TRAFFIC)).level_a + 1) % 4;
                traffic_lights
                    .update_bargraphs((*addr_of!(TRAFFIC)).level_a, (*addr_of!(TRAFFIC)).level_b);
            }
        }
        if evt_b == ButtonEvent::Released {
            unsafe {
                (*addr_of_mut!(TRAFFIC)).level_b = ((*addr_of!(TRAFFIC)).level_b + 1) % 4;
                traffic_lights
                    .update_bargraphs((*addr_of!(TRAFFIC)).level_a, (*addr_of!(TRAFFIC)).level_b);
            }
        }

        // Handle serial communication
        poll_serial(now);

        // State machine for traffic lights
        unsafe {
            let elapsed = now.wrapping_sub((*addr_of!(TRAFFIC)).phase_start_ms);
            let mut next_phase = false;

            match (*addr_of!(TRAFFIC)).state {
                TrafficPhase::AGreenBRed => {
                    set_lights(&traffic_lights, true, false, false, false, false, true);
                    if elapsed >= green_time_ms((*addr_of!(TRAFFIC)).level_a) {
                        (*addr_of_mut!(TRAFFIC)).state = TrafficPhase::AYellowBRed;
                        next_phase = true;
                    }
                }
                TrafficPhase::AYellowBRed => {
                    set_lights(&traffic_lights, false, true, false, false, false, true);
                    if elapsed >= (*addr_of!(CFG)).tl1.y_ms {
                        (*addr_of_mut!(TRAFFIC)).state = TrafficPhase::ARedBGreen;
                        next_phase = true;
                    }
                }
                TrafficPhase::ARedBGreen => {
                    set_lights(&traffic_lights, false, false, true, true, false, false);
                    if elapsed >= green_time_ms((*addr_of!(TRAFFIC)).level_b) {
                        (*addr_of_mut!(TRAFFIC)).state = TrafficPhase::ARedBYellow;
                        next_phase = true;
                    }
                }
                TrafficPhase::ARedBYellow => {
                    set_lights(&traffic_lights, false, false, true, false, true, false);
                    if elapsed >= (*addr_of!(CFG)).tl2.y_ms {
                        (*addr_of_mut!(TRAFFIC)).state = TrafficPhase::AGreenBRed;
                        next_phase = true;
                    }
                }
            }

            if next_phase {
                (*addr_of_mut!(TRAFFIC)).phase_start_ms = now;
            }
        }

        // Small delay to prevent overwhelming the CPU
        delay_ms(1);
    }
}

unsafe fn set_lights(
    tl: &TrafficLights,
    ag: bool,
    ay: bool,
    ar: bool,
    bg: bool,
    by: bool,
    br: bool,
) {
    tl.set_pair_a(ag, ay, ar);
    tl.set_pair_b(bg, by, br);

    // Update state tracking
    *addr_of_mut!(LED_STATE_A_GREEN) = ag;
    *addr_of_mut!(LED_STATE_A_YELLOW) = ay;
    *addr_of_mut!(LED_STATE_A_RED) = ar;
    *addr_of_mut!(LED_STATE_B_GREEN) = bg;
    *addr_of_mut!(LED_STATE_B_YELLOW) = by;
    *addr_of_mut!(LED_STATE_B_RED) = br;
}

fn green_time_ms(level: u8) -> u32 {
    unsafe {
        match level {
            0 => (*addr_of!(CFG)).tl1.g_ms,
            1 => (*addr_of!(CFG)).tl1.g_ms + (*addr_of!(CFG)).tl1.u_ms / 3,
            2 => (*addr_of!(CFG)).tl1.g_ms + ((*addr_of!(CFG)).tl1.u_ms * 2) / 3,
            3 => (*addr_of!(CFG)).tl1.g_ms + (*addr_of!(CFG)).tl1.u_ms,
            _ => (*addr_of!(CFG)).tl1.g_ms,
        }
    }
}

fn poll_serial(now: u32) {
    // Process incoming commands from PC console
    unsafe {
        while let Some(b) = (*addr_of_mut!(UART2)).try_read() {
            match b {
                b'\r' | b'\n' => {
                    // echo newline
                    (*addr_of_mut!(UART2)).write_str("\r\n");
                    if !(*addr_of!(CMD_LINE)).is_empty() {
                        let cmd = control::parse(&*addr_of!(CMD_LINE));
                        execute_cmd(cmd);
                        (*addr_of_mut!(CMD_LINE)).clear();
                    }
                }
                0x08 | 0x7F => {
                    // backspace/delete: remove last char and echo "\x08 \x08"
                    unsafe {
                        let line: &mut heapless::String<128> = &mut *addr_of_mut!(CMD_LINE);
                        if line.pop().is_some() {
                            (*addr_of_mut!(UART2)).write_str("\x08 \x08");
                        }
                    }
                }
                b if b.is_ascii_graphic() || b == b' ' => {
                    // printable: push to buffer and echo
                    if (*addr_of_mut!(CMD_LINE)).push(b as char).is_ok() {
                        (*addr_of_mut!(UART2)).write_byte(b);
                    }
                }
                _ => {
                    // ignore other control codes
                }
            }
        }

        // Relay data from downstream to PC
        while let Some(b) = (*addr_of_mut!(UARTB)).try_read() {
            (*addr_of_mut!(UART2)).write_byte(b);
        }
    }

    // Periodic monitoring...
    static mut LAST_MONITOR_MS: u32 = 0;
    unsafe {
        if now.wrapping_sub(*addr_of!(LAST_MONITOR_MS)) >= (*addr_of!(CFG)).monitor.interval_ms {
            *addr_of_mut!(LAST_MONITOR_MS) = now;
            send_status_report(now);
        }
    }
}

fn execute_cmd(cmd: control::Cmd) {
    unsafe {
        match cmd {
            Cmd::SetTL(idx, new_cfg) => {
                match idx {
                    1 => (*addr_of_mut!(CFG)).tl1 = new_cfg,
                    2 => (*addr_of_mut!(CFG)).tl2 = new_cfg,
                    _ => {
                        (*addr_of_mut!(UART2)).write_str("ERR: Invalid traffic light index\r\n");
                        return;
                    }
                }
                (*addr_of_mut!(UART2)).write_str(ANSI_GREEN);
                (*addr_of_mut!(UART2)).write_str("OK\r\n");
                (*addr_of_mut!(UART2)).write_str(ANSI_RESET);

                // Forward command to upstream
                let mut cmd_str: heapless::String<128> = heapless::String::new();
                let _ = write!(
                    cmd_str,
                    "config traffic light {} G Y R {} {} {} {}\r\n",
                    idx,
                    new_cfg.g_ms / 1000,
                    new_cfg.y_ms / 1000,
                    new_cfg.r_ms / 1000,
                    new_cfg.u_ms / 1000
                );
                (*addr_of_mut!(UARTA)).write_str(&cmd_str);
            }

            Cmd::SetMon(ms) => {
                (*addr_of_mut!(CFG)).monitor.interval_ms = ms;
                (*addr_of_mut!(UART2)).write_str(ANSI_GREEN);
                (*addr_of_mut!(UART2)).write_str("OK\r\n");
                (*addr_of_mut!(UART2)).write_str(ANSI_RESET);

                // Forward to upstream
                let mut cmd_str: heapless::String<64> = heapless::String::new();
                let _ = write!(cmd_str, "config traffic monitor {}\r\n", ms / 1000);
                (*addr_of_mut!(UARTA)).write_str(&cmd_str);
            }

            Cmd::ReadAll => {
                let mut buf: heapless::String<256> = heapless::String::new();
                control::format_cfg(&*addr_of!(CFG), None, &mut buf);
                (*addr_of_mut!(UART2)).write_str(&buf);
            }

            Cmd::ReadTL(idx) => {
                let mut buf: heapless::String<256> = heapless::String::new();
                control::format_cfg(&*addr_of!(CFG), idx, &mut buf);
                (*addr_of_mut!(UART2)).write_str(&buf);
            }

            Cmd::ReadMon => {
                let mut s: heapless::String<64> = heapless::String::new();
                let _ = write!(
                    s,
                    "traffic monitor {}\r\n",
                    (*addr_of!(CFG)).monitor.interval_ms / 1000
                );
                (*addr_of_mut!(UART2)).write_str(&s);
            }

            Cmd::Invalid => {
                (*addr_of_mut!(UART2)).write_str(ANSI_RED);
                (*addr_of_mut!(UART2)).write_str("ERR: Invalid command\r\n");
                (*addr_of_mut!(UART2)).write_str(ANSI_RESET);
            }
        }
    }
}

fn send_status_report(now: u32) {
    unsafe {
        let ts = now / 1000;
        let uart = &mut *addr_of_mut!(UART2);

        // ——— Line 1: Traffic Light 1 ———
        uart.write_str(ANSI_CYAN);
        write!(uart, "{} traffic light 1 (EW)  ", ts).ok();
        // A Green
        uart.write_str(if *addr_of!(LED_STATE_A_GREEN) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_A_GREEN) {
            "ON "
        } else {
            "OFF "
        });
        // A Yellow
        uart.write_str(if *addr_of!(LED_STATE_A_YELLOW) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_A_YELLOW) {
            "ON "
        } else {
            "OFF "
        });
        // A Red
        uart.write_str(if *addr_of!(LED_STATE_A_RED) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_A_RED) {
            "ON"
        } else {
            "OFF"
        });
        uart.write_str(ANSI_RESET);
        uart.write_str("\r\n");

        // ——— Line 2: Traffic Light 2 ———
        uart.write_str(ANSI_CYAN);
        write!(uart, "{} traffic light 2 (NS)  ", ts).ok();
        // B Green
        uart.write_str(if *addr_of!(LED_STATE_B_GREEN) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_B_GREEN) {
            "ON "
        } else {
            "OFF "
        });
        // B Yellow
        uart.write_str(if *addr_of!(LED_STATE_B_YELLOW) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_B_YELLOW) {
            "ON "
        } else {
            "OFF "
        });
        // B Red
        uart.write_str(if *addr_of!(LED_STATE_B_RED) {
            ANSI_GREEN
        } else {
            ANSI_RED
        });
        uart.write_str(if *addr_of!(LED_STATE_B_RED) {
            "ON"
        } else {
            "OFF"
        });
        uart.write_str(ANSI_RESET);
        uart.write_str("\r\n");

        // ——— Line 3: Levels ———
        uart.write_str(ANSI_CYAN);
        write!(
            uart,
            "{} traffic levels NS={} EW={}\r\n",
            ts,
            (*addr_of!(TRAFFIC)).level_a,
            (*addr_of!(TRAFFIC)).level_b
        )
        .ok();
        uart.write_str(ANSI_RESET);

        // ——— Line 4 & 5: Directions ———
        let ns = if (*addr_of!(TRAFFIC)).level_a >= 2 {
            "Heavy Traffic"
        } else {
            "Light Traffic"
        };
        let ew = if (*addr_of!(TRAFFIC)).level_b >= 2 {
            "Heavy Traffic"
        } else {
            "Light Traffic"
        };

        uart.write_str(ANSI_YELLOW);
        write!(uart, "{} NS direction: {}\r\n", ts, ns).ok();
        uart.write_str(ANSI_RESET);

        uart.write_str(ANSI_YELLOW);
        write!(uart, "{} EW direction: {}\r\n", ts, ew).ok();
        uart.write_str(ANSI_RESET);

        // ——— Mirror to upstream if desired ———
        let upstream: &mut Uart<USART1_BASE> = &mut *addr_of_mut!(UARTA);

        // upstream.write_str(ANSI_CYAN);
        // upstream.write_str("...(same blocks)...");
        // upstream.write_str(ANSI_RESET);
    }
}

// Interrupt handlers
#[interrupt]
fn USART1() {
    unsafe {
        (*addr_of_mut!(UARTA)).isr();
    }
}

#[interrupt]
fn USART2() {
    unsafe {
        (*addr_of_mut!(UART2)).isr();
    }
}

#[interrupt]
fn USART3() {
    unsafe {
        (*addr_of_mut!(UARTB)).isr();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
