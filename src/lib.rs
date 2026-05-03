use std::collections::VecDeque;
/// A mock for serial communication using the embedded-io-async traits Read and Write.
use std::vec::Vec;

use embedded_io::{Error, ErrorKind, ErrorType};
use embedded_io_async::{Read, Write};

pub struct MockSerialAsync {
    // transactions: Vec<SerialTransaction>,
    // current_transaction_index: usize,
    transactions: VecDeque<SerialTransaction>,
}

impl MockSerialAsync {
    pub fn new(expected_transactions: &[SerialTransaction]) -> Self {
        let transactions = VecDeque::from(expected_transactions.to_owned());
        MockSerialAsync { transactions }
    }

    /// Assert that all expectations on a given mock have been consumed.
    pub fn done(&self) {
        todo!()
    }
}

impl embedded_io_async::Read for MockSerialAsync {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self.transactions.pop_front() {
            Some(SerialTransaction::ReadMany(data)) => {
                buf.copy_from_slice(&data);
                Ok(data.len() as usize)
            }
            Some(other_transaction) => panic!("Expected read_many, got {}", other_transaction),
            None => panic!("Transaction read_many not expected",),
        }
    }

    async fn read_exact(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<(), embedded_io_async::ReadExactError<Self::Error>> {
        while !buf.is_empty() {
            match self.read(buf).await {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) => return Err(embedded_io_async::ReadExactError::Other(e)),
            }
        }
        if buf.is_empty() {
            Ok(())
        } else {
            Err(embedded_io_async::ReadExactError::UnexpectedEof)
        }
    }
}

impl embedded_io_async::Write for MockSerialAsync {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // capture output
        todo!()
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl embedded_io::ErrorType for MockSerialAsync {
    // type Error = MockSerialError;
    type Error = embedded_io::ErrorKind;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SerialTransaction {
    WriteMany(Vec<u8>),
    Flush,
    Read(Vec<u8>),
    ReadMany(Vec<u8>),
}

impl SerialTransaction {
    pub fn read_many(expectation: &[u8]) -> Self {
        SerialTransaction::ReadMany(Vec::from(expectation))
    }

    pub fn flush() -> Self {
        SerialTransaction::Flush
    }
}

impl std::fmt::Display for SerialTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let transaction_type = match self {
            SerialTransaction::WriteMany(_items) => "write_many".to_string(),

            SerialTransaction::Flush => "flush".to_string(),
            SerialTransaction::Read(_items) => "read".to_string(),
            SerialTransaction::ReadMany(_items) => "read_many".to_string(),
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

mod test {
    use super::*;

    #[test]
    fn test_new() {
        let expectations = [
            SerialTransaction::read_many(b"abcd"),
            SerialTransaction::Flush,
        ];

        let serial = MockSerialAsync::new(&expectations);

        assert_eq!(
            SerialTransaction::ReadMany(b"abcd".to_vec()),
            serial.transactions[0]
        );

        assert_eq!(SerialTransaction::Flush, serial.transactions[1]);
    }
}
