use embedded_io_async::{Read, Write};
use mock_serial_async::{MockSerialAsync, SerialTransaction};

#[tokio::test]
async fn test_read_many_transaction() {
    let expectations = [
        SerialTransaction::read_many(b"abcd"),
        SerialTransaction::flush(),
    ];

    let mut serial = MockSerialAsync::new(&expectations);

    let mut buf = [0 as u8; 4];
    let n = serial.read(&mut buf).await.expect("Read error");

    assert_eq!(4, n);
    assert_eq!(b"abcd", &buf);

    //serial.flush();

    //serial.done();
}
