//! String buffer implementation for buffering heapless::String messages
//!
//! This module provides a simple ring buffer for byte streams,
//! with convenience methods for reading and writing heapless::String objects.

use heapless::String;

/// A ring buffer for storing byte streams
///
/// The buffer stores bytes continuously without message boundaries.
/// Strings are written as their raw bytes and can be read back
/// up to the available data.
///
/// # Type Parameters
/// * `CAPACITY` - Total size of the byte buffer in bytes (must be a power of two)
pub struct StringBuffer<const CAPACITY: usize> {
    buffer: [u8; CAPACITY],
    read_pos: usize,
    write_pos: usize,
}

impl<const CAPACITY: usize> StringBuffer<CAPACITY> {
    /// Create a new empty string buffer
    pub const fn new() -> Self {
        // Compile-time validation: CAPACITY must be a power of two and > 1
        assert!(CAPACITY.is_power_of_two());
        assert!(CAPACITY > 1);

        Self {
            buffer: [0; CAPACITY],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Write a string into the buffer
    ///
    /// Returns `Ok(())` if the string was successfully written,
    /// or `Err(())` if there's not enough space.
    pub fn write<const N: usize>(&mut self, s: &String<N>) -> Result<(), ()> {
        let bytes = s.as_bytes();

        if self.free_space() < bytes.len() {
            return Err(());
        }

        for &byte in bytes {
            let idx = self.write_pos & (CAPACITY - 1);
            self.buffer[idx] = byte;
            self.write_pos = self.write_pos.wrapping_add(1);
        }

        Ok(())
    }

    /// Write a string slice into the buffer
    ///
    /// This is a convenience method that accepts a &str.
    /// Returns `Ok(())` if successful, or `Err(())` if the buffer is full or
    /// the string is too long.
    pub fn write_str<const N: usize>(&mut self, s: &str) -> Result<(), ()> {
        let string: String<N> = String::try_from(s).map_err(|_| ())?;
        self.write(&string)
    }

    /// Write raw bytes into the buffer
    ///
    /// Returns `Ok(())` if the bytes were successfully written,
    /// or `Err(())` if there's not enough space.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if self.free_space() < bytes.len() {
            return Err(());
        }

        for &byte in bytes {
            let idx = self.write_pos & (CAPACITY - 1);
            self.buffer[idx] = byte;
            self.write_pos = self.write_pos.wrapping_add(1);
        }

        Ok(())
    }

    /// Read up to N bytes from the buffer into a String
    ///
    /// Reads as much data as available (up to N bytes) and converts it to a string.
    /// Returns `None` if the buffer is empty or the data isn't valid UTF-8.
    pub fn read<const N: usize>(&mut self) -> Option<String<N>> {
        let available = self.available_bytes();
        if available == 0 {
            return None;
        }

        let to_read = available.min(N);
        let mut result = String::new();

        for _ in 0..to_read {
            let idx = self.read_pos & (CAPACITY - 1);
            let byte = self.buffer[idx];
            self.read_pos = self.read_pos.wrapping_add(1);

            if result.push(byte as char).is_err() {
                break;
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Read exactly the requested number of bytes into a String
    ///
    /// Returns `None` if there aren't enough bytes available or if the data isn't valid UTF-8.
    pub fn read_exact<const N: usize>(&mut self) -> Option<String<N>> {
        let available = self.available_bytes();
        if available < N {
            return None;
        }

        let mut result = String::new();

        for _ in 0..N {
            let idx = self.read_pos & (CAPACITY - 1);
            let byte = self.buffer[idx];
            self.read_pos = self.read_pos.wrapping_add(1);

            if result.push(byte as char).is_err() {
                return None;
            }
        }

        Some(result)
    }

    /// Read raw bytes from the buffer
    ///
    /// Reads up to `buf.len()` bytes into the provided buffer.
    /// Returns the number of bytes actually read.
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> usize {
        let available = self.available_bytes();
        let to_read = available.min(buf.len());

        for i in 0..to_read {
            let idx = self.read_pos & (CAPACITY - 1);
            buf[i] = self.buffer[idx];
            self.read_pos = self.read_pos.wrapping_add(1);
        }

        to_read
    }

    // Empty the buffer. We don't actually need to overwrite anything.
    // TODO: Fix possible race condition if clear occurs while reading/writing
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.free_space() == 0
    }

    /// Get the number of bytes available to read
    pub fn available(&self) -> usize {
        self.available_bytes()
    }

    /// Free space in bytes (one slot kept free to distinguish full vs empty)
    fn free_space(&self) -> usize {
        let used = self.write_pos.wrapping_sub(self.read_pos);
        CAPACITY - 1 - used
    }

    /// Bytes available to read
    fn available_bytes(&self) -> usize {
        self.write_pos.wrapping_sub(self.read_pos)
    }
}

