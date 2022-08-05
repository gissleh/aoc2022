use num::Integer;
use std::ops::Neg;

pub fn byte(data: &[u8]) -> Option<(u8, &[u8])> {
    if !data.is_empty() {
        Some((data[0], &data[1..]))
    } else {
        None
    }
}

pub fn byte_array<const N: usize>(data: &[u8]) -> Option<([u8; N], &[u8])> {
    if data.len() >= N {
        let mut arr = [0u8; N];
        arr.copy_from_slice(&data[..N]);

        Some((arr, &data[N..]))
    } else {
        None
    }
}

pub fn expect_byte<const B: u8>(data: &[u8]) -> Option<((), &[u8])> {
    if let Some(_) = data.get(0).filter(|b| **b == B) {
        Some(((), &data[1..]))
    } else {
        None
    }
}

pub fn expect_bytes<'a, 'b>(pred: &'b [u8]) -> impl Fn(&'a [u8]) -> Option<((), &'a [u8])> + 'b {
    move |data| {
        if data.starts_with(pred) {
            Some(((), &data[pred.len()..]))
        } else {
            None
        }
    }
}

pub fn expect_str<'a, 'b>(pred: &'b str) -> impl Fn(&'a [u8]) -> Option<((), &'a [u8])> + 'b {
    move |data| {
        if data.starts_with(pred.as_bytes()) {
            Some(((), &data[pred.len()..]))
        } else {
            None
        }
    }
}

pub fn line(data: &[u8]) -> Option<(&[u8], &[u8])> {
    if data.is_empty() {
        Some((data, data))
    } else {
        let len = data.iter().take_while(|v| **v != b'\n').count();
        Some((&data[..len], &data[len+1..]))
    }
}

pub fn word(data: &[u8]) -> Option<(&[u8], &[u8])> {
    if data.is_empty() {
        None
    } else {
        let len = data.iter().take_while(|v| **v != b' ' && **v != b'\n').count();

        if len == data.len() {
            Some((data, &data[len..]))
        } else {
            Some((&data[..len], &data[len+1..]))
        }
    }
}

pub fn until_byte<const B: u8>(data: &[u8]) -> Option<(&[u8], &[u8])> {
    if data.is_empty() {
        None
    } else {
        let len = data.iter().take_while(|v| **v != B).count();
        Some((&data[..len], &data[len..]))
    }
}


pub fn int<T: Integer + Copy + From<u8> + Neg<Output=T>>(data: &[u8]) -> Option<(T, &[u8])> {
    if data.is_empty() {
        return None;
    }

    let mut sum = T::zero();
    let mut neg = false;
    let ten = T::from(10u8);

    for (i, b) in data.iter().enumerate() {
        match *b {
            b'0'..=b'9' => {
                sum = sum.mul(ten);
                sum = sum.add(T::from(*b - b'0'));
            }
            b'-' if !neg => {
                neg = true;
            }
            _ => {
                return if i > 0 {
                    if neg {
                        sum = sum.neg()
                    }
                    Some((sum, &data[i..]))
                } else {
                    None
                };
            }
        }
    }

    if neg {
        sum = sum.neg()
    }
    Some((sum, &data[data.len()..]))
}

pub fn uint<T: Integer + Copy + From<u8>>(data: &[u8]) -> Option<(T, &[u8])> {
    if data.is_empty() {
        return None;
    }

    let mut sum = T::zero();
    let ten = T::from(10u8);

    for (i, b) in data.iter().enumerate() {
        match *b {
            b'0'..=b'9' => {
                sum = sum.mul(ten);
                sum = sum.add(T::from(*b - b'0'));
            }
            _ => {
                return if i > 0 {
                    Some((sum, &data[i..]))
                } else {
                    None
                };
            }
        }
    }

    Some((sum, &data[data.len()..]))
}

pub fn hex_byte(data: &[u8]) -> Option<(u8, &[u8])> {
    if let Some((a, data)) = hex_number(data) {
        return if let Some((b, data)) = hex_number(&data) {
            Some(((a << 4) + b, data))
        } else {
            Some(((a << 4), data))
        }
    }

    None
}

pub fn hex_number(data: &[u8]) -> Option<(u8, &[u8])> {
    if data.is_empty() {
        return None;
    }

    let hex = data[0];
    match hex {
        b'0'..=b'9' => Some((hex - b'0', &data[1..])),
        b'A'..=b'F' => Some(((hex - b'A') + 10, &data[1..])),
        b'a'..=b'f' => Some(((hex - b'a') + 10, &data[1..])),
        _ => None,
    }
}

#[macro_export]
macro_rules! parse_all {
    ($input:ident, $p1:expr) => {
        if let Some((r1, input)) = $p1($input) {
            Some((r1, input))
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                Some((r1, r2, input))
            } else { None }
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr, $p3:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                if let Some((r3, input)) = $p3(input) {
                    Some((r1, r2, r3, input))
                } else { None }
            } else { None }
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr, $p3:expr, $p4:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                if let Some((r3, input)) = $p3(input) {
                    if let Some((r4, input)) = $p4(input) {
                        Some((r1, r2, r3, r4, input))
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                if let Some((r3, input)) = $p3(input) {
                    if let Some((r4, input)) = $p4(input) {
                        if let Some((r5, input)) = $p5(input) {
                            Some((r1, r2, r3, r4, r5, input))
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                if let Some((r3, input)) = $p3(input) {
                    if let Some((r4, input)) = $p4(input) {
                        if let Some((r5, input)) = $p5(input) {
                            if let Some((r6, input)) = $p6(input) {
                                Some((r1, r2, r3, r4, r5, r6, input))
                            } else { None }
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };

    ($input:ident, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr) => {
        if let Some((r1, input)) = $p1($input) {
            if let Some((r2, input)) = $p2(input) {
                if let Some((r3, input)) = $p3(input) {
                    if let Some((r4, input)) = $p4(input) {
                        if let Some((r5, input)) = $p5(input) {
                            if let Some((r6, input)) = $p6(input) {
                                if let Some((r7, input)) = $p7(input) {
                                    Some((r1, r2, r3, r4, r5, r6, r7, input))
                                } else { None }
                            } else { None }
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    };
}



