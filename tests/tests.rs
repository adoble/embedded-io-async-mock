use embedded_io_async::{Read, Write};
use embedded_io_async_mock::{Mock as SerialAsyncMock, Transaction as SerialTransaction};

#[tokio::test]
async fn test_read_transaction() {
    let expectations = [SerialTransaction::read(b"abcd"), SerialTransaction::flush()];

    let mut serial = SerialAsyncMock::new(&expectations);

    let mut buf = [0 as u8; 4];
    let n = serial.read(&mut buf).await.expect("Read error");

    assert_eq!(4, n);
    assert_eq!(b"abcd", &buf);

    let r = serial.flush().await;
    assert!(r.is_ok());

    serial.done();
}

#[tokio::test]
async fn test_read_single_transaction() {
    let expectations = [
        SerialTransaction::read(b"a"),
        SerialTransaction::read(b"b"),
        SerialTransaction::read(b"c"),
        SerialTransaction::read(b"d"),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    let mut buf = [0 as u8; 1];
    let mut n = serial.read(&mut buf).await.expect("Read error");

    assert_eq!(1, n);
    assert_eq!(b"a", &buf);

    n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(1, n);
    assert_eq!(b"b", &buf);

    n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(1, n);
    assert_eq!(b"c", &buf);

    n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(1, n);
    assert_eq!(b"d", &buf);

    serial.done();
}

#[tokio::test]
async fn test_write_transaction() {
    let expectations = [
        SerialTransaction::write(b"abcd"),
        SerialTransaction::flush(),
        SerialTransaction::write(b"efgh"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    let n = serial.write(b"abcd").await.expect("Write error");
    assert_eq!(4, n);

    let r = serial.flush().await;
    assert!(r.is_ok());

    let n = serial.write(b"efgh").await.expect("Write error");
    assert_eq!(4, n);

    let r = serial.flush().await;
    assert!(r.is_ok());

    serial.done();
}

#[tokio::test]
#[should_panic]
async fn test_not_all_transactions_consumed() {
    let expectations = [
        SerialTransaction::write(b"abcd"),
        SerialTransaction::flush(),
        SerialTransaction::write(b"efgh"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    let _ = serial.write(b"abcd").await.expect("Write error");

    serial.flush().await.expect("Flush error");

    let _ = serial.write(b"efgh").await.expect("Write error");

    // flush missing

    serial.done();
}

#[tokio::test]
#[should_panic]
async fn test_false_transaction() {
    let expectations = [
        SerialTransaction::write(b"abcd"),
        SerialTransaction::flush(),
        SerialTransaction::write(b"efgh"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    let _ = serial.write(b"abcd").await.expect("Write error");
    serial.flush().await.expect("Flush error");
    let mut buf = [0 as u8; 4];

    // False transaction
    let _ = serial.read(&mut buf).await.unwrap();

    serial.flush().await.expect("Flush error");

    serial.done();
}

#[tokio::test]
#[should_panic]
async fn test_unexpected_data_on_write() {
    let expectations = [SerialTransaction::write(b"abcd")];

    let mut serial = SerialAsyncMock::new(&expectations);

    let _ = serial.write(b"abxd").await.expect("Write error");

    serial.done();
}

#[tokio::test]
async fn test_readme_example() {
    let expectations = [
        SerialTransaction::write(b"VOL;"),
        SerialTransaction::flush(),
        SerialTransaction::read(b"42"),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    assert!(serial.write(b"VOL;").await.is_ok());
    assert!(serial.flush().await.is_ok());

    let mut buf = [0 as u8; 2];
    assert!(serial.read(&mut buf).await.is_ok());

    serial.done();
}

#[tokio::test]
async fn test_simple_read_many() {
    let expectations = [
        SerialTransaction::read_many(b"VOL:42;"),
        SerialTransaction::flush(),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    let mut buf = [0u8; 4];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 4);
    assert_eq!(&buf, b"VOL:");

    let mut buf = [0u8; 3];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 3);
    assert_eq!(&buf, b"42;");

    assert!(serial.flush().await.is_ok());

    serial.done();
}

#[tokio::test]
async fn test_read_many_no_transaction_boundary() {
    let expectations = [SerialTransaction::read_many(b"VOL:42;")];

    let mut serial = SerialAsyncMock::new(&expectations);
    let mut buf = [0u8; 4];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 4);
    assert_eq!(&buf, b"VOL:");

    let mut buf = [0u8; 3];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 3);
    assert_eq!(&buf, b"42;");

    serial.done();
}

#[tokio::test]
async fn test_read_many_but_not_enough_data() {
    let expectations = [SerialTransaction::read_many(b"VOL:;")];

    let mut serial = SerialAsyncMock::new(&expectations);

    let mut buf = [0u8; 4];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 4);
    assert_eq!(&buf, b"VOL:");

    let mut buf = [0u8; 3];
    let n = serial.read(&mut buf).await.expect("Read error");
    assert_eq!(n, 1);
    assert_eq!(&buf, &[b';', 0, 0]);

    serial.done();
}

#[tokio::test]
async fn test_write_many() {
    let expectations = [SerialTransaction::write_many(b"VOL:42;")];

    let mut serial = SerialAsyncMock::new(&expectations);

    let mut n = serial.write(b"VOL:").await.expect("Write error");
    assert_eq!(n, 4);

    n = serial.write(b"42;").await.expect("Write error");
    assert_eq!(n, 3);
}
