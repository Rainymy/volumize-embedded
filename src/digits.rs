use heapless::Vec;

pub fn number_to_vec<const N: usize, T: Into<usize>>(number: T) -> Vec<u8, N> {
    let mut digits = Vec::new();
    let mut n = number.into() as usize;

    // Edge case 1: Vec is empty if number is 0.
    if n == 0 {
        let _ = digits.push(0);
        return digits;
    }

    while n > 0 {
        let current = n % 10;
        let _ = digits.push(current as u8);
        n /= 10;
    }

    // Edge case 2: Appending leading 0 into Vec.
    if n != 0 {
        let _ = digits.push(n as u8);
    }
    digits.reverse();
    digits
}

pub fn digit_to_str(digit: u8) -> &'static str {
    match digit {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => "X",
    }
}
