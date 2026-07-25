use koralys_rust::Reader;

#[test]
fn test_reader_byte() {
    let data = vec![0x12, 0x34, 0x56];
    let mut reader = Reader::new(&data);
    assert_eq!(reader.pos(), 0);
    assert!(reader.can_read(3));
    assert_eq!(reader.next_byte(), 0x12);
    assert_eq!(reader.next_byte(), 0x34);
    assert_eq!(reader.next_byte(), 0x56);
    assert_eq!(reader.pos(), 3);
    assert!(!reader.can_read(1));
}

#[test]
fn test_reader_var_int_single_byte() {
    let data = vec![0x00, 0x01, 0x7F];
    let mut reader = Reader::new(&data);
    assert_eq!(reader.next_var_int(), 0);
    assert_eq!(reader.next_var_int(), 1);
    assert_eq!(reader.next_var_int(), 127);
}

#[test]
fn test_reader_var_int_multi_byte() {
    // 128 -> 0x80, 0x01
    // 300 -> 300 = 0x12C -> low 7 bits 0x2C | 0x80 = 0xAC, high bits = 0x02
    let data = vec![0x80, 0x01, 0xAC, 0x02];
    let mut reader = Reader::new(&data);
    assert_eq!(reader.next_var_int(), 128);
    assert_eq!(reader.next_var_int(), 300);
}

#[test]
fn test_reader_string() {
    // Length 5 ("hello")
    let mut data = vec![0x05];
    data.extend_from_slice(b"hello");
    let mut reader = Reader::new(&data);
    assert_eq!(reader.next_string(), "hello");
}

#[test]
fn test_reader_int_and_floats() {
    let mut data = Vec::new();
    // next_int (u32 LE): 0x12345678 -> 0x78, 0x56, 0x34, 0x12
    data.extend_from_slice(&[0x78, 0x56, 0x34, 0x12]);
    // f32 LE: 1.5f32 -> 0x3FC00000 -> [0x00, 0x00, 0xC0, 0x3F]
    data.extend_from_slice(&1.5f32.to_le_bytes());
    // f64 LE: 42.0f64
    data.extend_from_slice(&42.0f64.to_le_bytes());

    let mut reader = Reader::new(&data);
    assert_eq!(reader.next_int(), 0x12345678);
    assert_eq!(reader.next_float(), 1.5);
    assert_eq!(reader.next_double(), 42.0);
}

#[test]
#[should_panic]
fn test_reader_out_of_bounds() {
    let data = vec![0x01];
    let mut reader = Reader::new(&data);
    reader.next_byte();
    reader.next_byte(); // should panic
}
