#![allow(dead_code)]
use core::ptr::{read_volatile, write_volatile};

/// A minimal ring buffer (power of two size).
pub struct Ring<const N: usize> {
    buf: [u8; N],
    head: usize,
    tail: usize,
}

impl<const N: usize> Ring<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, b: u8) -> bool {
        let next = (self.head + 1) & (N - 1);
        if next == self.tail {
            return false;
        } // full
        self.buf[self.head] = b;
        self.head = next;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            return None;
        }
        let b = self.buf[self.tail];
        self.tail = (self.tail + 1) & (N - 1);
        Some(b)
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn len(&self) -> usize {
        (self.head.wrapping_sub(self.tail)) & (N - 1)
    }

    pub fn capacity(&self) -> usize {
        N - 1 // One slot is always kept empty to distinguish full from empty
    }

    pub fn is_full(&self) -> bool {
        ((self.head + 1) & (N - 1)) == self.tail
    }
}

/// A very small driver for a single USART in **asynchronous 8-N-1**.
/// * `USARTx_BASE` is 0x4001_1000 for USART1, 0x4000_4400 for USART2, 0x4000_4800 for USART3, etc.
pub struct Uart<const USARTx_BASE: u32> {
    tx_buf: Ring<256>,
    rx_buf: Ring<256>,
}

impl<const B: u32> Uart<B> {
    const SR: u32 = 0x00;
    const DR: u32 = 0x04;
    const BRR: u32 = 0x08;
    const CR1: u32 = 0x0C;
    const CR2: u32 = 0x10;
    const CR3: u32 = 0x14;

    pub const fn new() -> Self {
        Self {
            tx_buf: Ring::new(),
            rx_buf: Ring::new(),
        }
    }

    /// Enable clocks, set baud-rate, 8-N-1, RXNE + TXE interrupts.
    pub unsafe fn init(&self, pclk_hz: u32, baud: u32, apb_enr_addr: *mut u32, enr_bit: u32) {
        // 1) Enable clock
        write_volatile(apb_enr_addr, read_volatile(apb_enr_addr) | enr_bit);

        // 2) Disable USART before configuration
        write_volatile((B + Self::CR1) as *mut u32, 0);

        // 3) Set baud rate: BRR = pclk / baud
        let brr = pclk_hz / baud;
        write_volatile((B + Self::BRR) as *mut u32, brr);

        // 4) Configure CR2 for 1 stop bit (default 00)
        write_volatile((B + Self::CR2) as *mut u32, 0);

        // 5) Configure CR3 (no hardware flow control)
        write_volatile((B + Self::CR3) as *mut u32, 0);

        // 6) Enable RXNEIE | TE | RE | UE (no TXEIE initially)
        write_volatile(
            (B + Self::CR1) as *mut u32,
            (1 << 5)  // RXNEIE - RX not empty interrupt enable
            | (1 << 3)  // TE - Transmitter enable  
            | (1 << 2)  // RE - Receiver enable
            | (1 << 13), // UE - USART enable
        );
    }

    /// Called from main-line loop to push outbound data.
    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.tx_buf.push(b);
        }
        // Enable TXE interrupt to start transmission
        unsafe {
            let cr1 = (B + Self::CR1) as *mut u32;
            write_volatile(cr1, read_volatile(cr1) | (1 << 7)); // TXEIE
        }
    }

    /// Write a single byte
    pub fn write_byte(&mut self, b: u8) {
        self.tx_buf.push(b);
        // Enable TXE interrupt to start transmission
        unsafe {
            let cr1 = (B + Self::CR1) as *mut u32;
            write_volatile(cr1, read_volatile(cr1) | (1 << 7)); // TXEIE
        }
    }

    pub fn try_read(&mut self) -> Option<u8> {
        self.rx_buf.pop()
    }

    /// Check if transmit buffer is empty
    pub fn tx_ready(&self) -> bool {
        self.tx_buf.is_empty()
    }

    /// Check if receive buffer has data
    pub fn rx_available(&self) -> bool {
        !self.rx_buf.is_empty()
    }

    /// Interrupt handler **must** be registered in `#[interrupt]`
    /// wrappers in main.rs.
    pub unsafe fn isr(&mut self) {
        let sr = read_volatile((B + Self::SR) as *const u32);

        // Handle receive interrupt
        if (sr & (1 << 5)) != 0 {
            // RXNE - Receive data register not empty
            let b = read_volatile((B + Self::DR) as *const u32) as u8;
            self.rx_buf.push(b);
        }

        // Handle transmit interrupt
        if (sr & (1 << 7)) != 0 {
            // TXE - Transmit data register empty
            if let Some(b) = self.tx_buf.pop() {
                write_volatile((B + Self::DR) as *mut u32, b as u32);
            } else {
                // No more bytes: disable TXEIE
                let cr1 = (B + Self::CR1) as *mut u32;
                write_volatile(cr1, read_volatile(cr1) & !(1 << 7));
            }
        }

        // Handle other potential interrupts/errors
        if (sr & (1 << 3)) != 0 {
            // ORE - Overrun error
            // Clear by reading DR
            let _ = read_volatile((B + Self::DR) as *const u32);
        }

        if (sr & (1 << 1)) != 0 {
            // FE - Framing error
            // Clear by reading DR
            let _ = read_volatile((B + Self::DR) as *const u32);
        }

        if (sr & (1 << 2)) != 0 {
            // NF - Noise detected flag
            // Clear by reading DR
            let _ = read_volatile((B + Self::DR) as *const u32);
        }
    }

    /// Get the current status register value (for debugging)
    pub unsafe fn get_status(&self) -> u32 {
        read_volatile((B + Self::SR) as *const u32)
    }

    /// Clear all error flags
    pub unsafe fn clear_errors(&self) {
        // Reading DR clears ORE, FE, NF flags
        let _ = read_volatile((B + Self::DR) as *const u32);
    }
}

impl<const B: u32> core::fmt::Write for Uart<B> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.tx_buf.push(b);
        }
        // Enable TXE interrupt to start transmission
        unsafe {
            let cr1 = (B + Self::CR1) as *mut u32;
            write_volatile(cr1, read_volatile(cr1) | (1 << 7)); // TXEIE
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut ring: Ring<8> = Ring::new();

        assert!(ring.is_empty());
        assert!(!ring.is_full());
        assert_eq!(ring.len(), 0);

        // Push some data
        assert!(ring.push(1));
        assert!(ring.push(2));
        assert!(ring.push(3));

        assert!(!ring.is_empty());
        assert_eq!(ring.len(), 3);

        // Pop some data
        assert_eq!(ring.pop(), Some(1));
        assert_eq!(ring.pop(), Some(2));
        assert_eq!(ring.len(), 1);

        assert_eq!(ring.pop(), Some(3));
        assert_eq!(ring.pop(), None);
        assert!(ring.is_empty());
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let mut ring: Ring<4> = Ring::new(); // Capacity is 3 (N-1)

        // Fill to capacity
        assert!(ring.push(1));
        assert!(ring.push(2));
        assert!(ring.push(3));
        assert!(ring.is_full());
        assert!(!ring.push(4)); // Should fail when full

        // Pop one and push one
        assert_eq!(ring.pop(), Some(1));
        assert!(ring.push(4));
        assert!(ring.is_full());

        // Verify order
        assert_eq!(ring.pop(), Some(2));
        assert_eq!(ring.pop(), Some(3));
        assert_eq!(ring.pop(), Some(4));
        assert_eq!(ring.pop(), None);
    }
}
