//! A small mock serial communication helper using `embedded-io-async`.
//!
//! This crate provides an asynchronous mock serial peripheral that can be used in tests to verify expected read/write/flush transactions.
//!
//! It attempts to provide the missing async serial functionality in [embedded-hal-mock](https://docs.rs/embedded-hal-mock/0.11.1/embedded_hal_mock/index.html).
//! As such, it orientates itself around the API, but differs in some respects.
//!
//! ## Features
//!
//! - [Mock] records expected serial transactions.
//! - Supports `Read` and `Write` semantics from `embedded-io-async`.
//! - Useful for unit testing async serial drivers.
//!
//! ## Example
//!
//! ```rust
//! use embedded_io_async_mock::{Mock as SerialAsyncMock, Transaction as SerialTransaction};
//! use embedded_io_async::{Read, Write};
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//!     let expectations = [
//!         SerialTransaction::write(b"VOL;"),
//!         SerialTransaction::flush(),
//!         SerialTransaction::read(b"42"),
//!     ];
//!
//!     let mut serial = SerialAsyncMock::new(&expectations);
//!
//!     assert!(serial.write(b"VOL;").await.is_ok());
//!     assert!(serial.flush().await.is_ok());
//!
//!     let mut buf = [0 as u8; 2];
//!     assert!(serial.read(&mut buf).await.is_ok());
//!
//!     serial.done();
//! # }
//! ```
//!

use std::collections::VecDeque;
use std::vec::Vec;
// TODO
// [ ] Check the formatting of the error messages against embedded-hal-mock
// [ ] Add badges to the README.

/// A mock for serial communication using the `embedded-io-async` traits Read and Write.
///

pub struct Mock {
    transactions: VecDeque<Transaction>,
    all_consumed: bool,
    transactions_aborted: bool,

    read_index: usize,
    write_index: usize,
}

impl Mock {
    /// Create a serial mock that will expect the provided transactions
    pub fn new(expected_transactions: &[Transaction]) -> Self {
        let transactions = VecDeque::from(expected_transactions.to_owned());
        Mock {
            transactions,
            all_consumed: false,
            transactions_aborted: false,
            read_index: 0,
            write_index: 0,
        }
    }

    /// Assert that all expectations on a given mock have been consumed.
    pub fn done(&mut self) {
        self.all_consumed = self.transactions.is_empty();
        assert!(
            self.all_consumed,
            "All transactions have not been consumed."
        );
    }
}

impl embedded_io_async::Read for Mock {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self.transactions.pop_front() {
            Some(Transaction::Read(data)) => {
                buf.copy_from_slice(&data);
                Ok(data.len())
            }
            Some(Transaction::ReadMany(data)) => {
                let available = data.len().saturating_sub(self.read_index);
                let n = buf.len().min(available);
                buf[..n].copy_from_slice(&data[self.read_index..self.read_index + n]);

                self.read_index += n;

                // If not finished reading, push the ReadMany transaction back onto the stack
                if self.read_index < data.len() {
                    self.transactions.push_front(Transaction::ReadMany(data));
                }
                Ok(n)
            }

            Some(other_transaction) => {
                self.transactions_aborted = true;
                panic!("Expected read, got {}", other_transaction);
            }
            None => {
                self.transactions_aborted = true;
                panic!("Transaction read not expected")
            }
        }
    }
}

impl embedded_io_async::Write for Mock {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        match self.transactions.pop_front() {
            Some(Transaction::Write(data)) => {
                assert_eq!(data.as_slice(), buf);
                Ok(buf.len())
            }
            Some(Transaction::WriteMany(data)) => {
                assert!(
                    self.write_index + buf.len() <= data.len(),
                    "write_many: expected to write {} bytes, instead writing {} bytes",
                    data.len(),
                    self.write_index + buf.len()
                );
                assert_eq!(
                    data[self.write_index..(self.write_index + buf.len())],
                    buf[..],
                    "Expected and written bytes differ"
                );
                self.write_index += buf.len();
                // If not finished writing, push the WriteMany transaction back onto the stack
                if self.write_index < data.len() {
                    self.transactions.push_front(Transaction::WriteMany(data));
                }
                Ok(buf.len())
            }

            Some(other_transaction) => {
                self.transactions_aborted = true;
                panic!("Expected write, got {}", other_transaction)
            }
            None => {
                self.transactions_aborted = true;
                panic!("Transaction write not expected",)
            }
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        match self.transactions.pop_front() {
            Some(Transaction::Flush) => Ok(()),
            Some(other_transaction) => {
                self.transactions_aborted = true;
                panic!("Expected flush, got {}", other_transaction)
            }
            None => {
                self.transactions_aborted = true;
                panic!("Transaction flush not expected",)
            }
        }
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        if !self.all_consumed && !self.transactions_aborted && !std::thread::panicking() {
            panic!("MockSerialAsync::done was not called before it went out of scope");
        }
    }
}

impl embedded_io::ErrorType for Mock {
    // type Error = MockSerialError;
    type Error = embedded_io::ErrorKind;
}

/// A async serial transaction
///
/// Transactions can either be reads, writes, or flushes. A collection of transactions represent
/// the expected async operations that are performed on a serial device.
///
/// Instead of using the variants driectly, it is advised to use the implemented functions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Transaction {
    Write(Vec<u8>),
    WriteMany(Vec<u8>),
    Flush,
    Read(Vec<u8>),
    ReadMany(Vec<u8>),
}

impl Transaction {
    /// Expect serial read that returns the expected bytes.
    pub fn read(expected: &[u8]) -> Self {
        Transaction::Read(Vec::from(expected))
    }

    /// Expect a serial write that transmits the expected bytes.
    pub fn write(expected: &[u8]) -> Self {
        Transaction::Write(Vec::from(expected))
    }

    /// Expect one of more serial reads after one another.  
    /// This avoids specifing a transaction for each `read`` call.
    ///
    //
    /// ```
    /// use embedded_io_async::{Read, Write};
    /// use embedded_io_async_mock::{Mock as SerialAsyncMock, Transaction as SerialTransaction};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    ///
    /// let expectations = [
    ///     SerialTransaction::read_many(b"VOL:42;"),
    ///     SerialTransaction::flush(),
    /// ];
    ///
    /// let mut serial = SerialAsyncMock::new(&expectations);
    ///
    /// let mut buf = [0u8; 4];
    /// let n = serial.read(&mut buf).await.expect("Read error");
    /// assert_eq!(n, 4);
    /// assert_eq!(&buf, b"VOL:");
    ///
    /// let mut buf = [0u8; 3];
    /// let n = serial.read(&mut buf).await.expect("Read error");
    /// assert_eq!(n, 3);
    /// assert_eq!(&buf, b"42;");
    ///
    /// assert!(serial.flush().await.is_ok());
    ///
    /// serial.done();
    /// # }
    /// ```
    pub fn read_many(expected: &[u8]) -> Self {
        Transaction::ReadMany(Vec::from(expected))
    }

    /// Expect one of more serial writes after one another.
    /// This avoids specifing a transaction for each `write` call.
    /// ```
    ///
    /// use embedded_io_async::{Read, Write};
    /// use embedded_io_async_mock::{Mock as SerialAsyncMock, Transaction as SerialTransaction};
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let expectations = [SerialTransaction::write_many(b"VOL:42;")];
    ///
    /// let mut serial = SerialAsyncMock::new(&expectations);
    ///
    /// let mut n = serial.write(b"VOL:").await.expect("Write error");
    /// assert_eq!(n, 4);
    ///
    /// n = serial.write(b"42;").await.expect("Write error");
    /// assert_eq!(n, 3);
    ///
    /// serial.done();
    /// # }
    /// ```
    ///
    pub fn write_many(expected: &[u8]) -> Self {
        Transaction::WriteMany(Vec::from(expected))
    }

    /// Expect a serial flush to clear the serial buffers
    pub fn flush() -> Self {
        Transaction::Flush
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let transaction_type = match self {
            Self::Flush => "flush".to_string(),
            Self::Write(_items) => "write".to_string(),
            Self::Read(_items) => "read".to_string(),
            Self::ReadMany(_items) => "read_many".to_string(),
            Self::WriteMany(_items) => "write_many".to_string(),
        };

        write!(f, "{}", transaction_type)
    }
}

// // #[derive(Debug)]
// #[derive(Debug)]
// pub enum MockSerialError {
//     BufferEmpty,
//     // Add other variants as needed
// }

// impl std::error::Error for MockSerialError {}

// impl embedded_io::Error for MockSerialError {
//     fn kind(&self) -> embedded_io::ErrorKind {
//         match self {
//             MockSerialError::BufferEmpty => embedded_io_async::ErrorKind::Other,
//         }
//     }
// }
