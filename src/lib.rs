/// A mock for serial communication using the embedded-io-async traits Read and Write.
///
// TODO Change the name of the crate to embedded-io-async-mock
// TODO Add some module documentation here including examples
// TODO Check the formatting of the error meesages againt embedded-hal-mock
use std::vec::Vec;
use std::{collections::VecDeque, os::unix::thread};

use embedded_io::{Error, ErrorKind, ErrorType};
use embedded_io_async::{Read, Write};

pub struct Mock {
    // transactions: Vec<SerialTransaction>,
    // current_transaction_index: usize,
    transactions: VecDeque<Transaction>,
    all_consumed: bool,
    transactions_aborted: bool,
}

impl Mock {
    pub fn new(expected_transactions: &[Transaction]) -> Self {
        let transactions = VecDeque::from(expected_transactions.to_owned());
        Mock {
            transactions,
            all_consumed: false,
            transactions_aborted: false,
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
                Ok(data.len() as usize)
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Transaction {
    Write(Vec<u8>),
    Flush,
    Read(Vec<u8>),
}

impl Transaction {
    pub fn read(expected: &[u8]) -> Self {
        Transaction::Read(Vec::from(expected))
    }

    pub fn write(expected: &[u8]) -> Self {
        Transaction::Write(Vec::from(expected))
    }

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
