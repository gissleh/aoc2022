use md5::Digest;

common::day!(parse, part1, part2, 10000, 10, 10);

fn parse(data: &[u8]) -> Vec<u8> {
    data.iter().take_while(|b| **b != b'\n').cloned().collect()
}

fn part1(input: &[u8]) -> usize {
    let mut buf = [0u8; 32];
    (&mut buf[..input.len()]).copy_from_slice(&input);

    let input_len = input.len();
    for n in 0.. {
        let number_len = append_number(&mut buf[input_len..], n);
        let Digest(digest) = md5::compute(&buf[..number_len+input_len]);

        if digest[0] == 0 && digest[1] == 0 && digest[2] & 0xf0 == 0 {
            return n;
        }
    }

    0
}

fn part2(input: &[u8]) -> usize {
    let mut buf = [0u8; 32];
    (&mut buf[..input.len()]).copy_from_slice(&input);

    let input_len = input.len();
    for n in 0.. {
        let number_len = append_number(&mut buf[input_len..], n);
        let Digest(digest) = md5::compute(&buf[..number_len+input_len]);

        if digest[0] == 0 && digest[1] == 0 && digest[2] == 0 {
            return n;
        }
    }

    0
}

fn append_number(buf: &mut [u8], num: usize) -> usize {
    let mut temp = [0u8; 32];
    let mut pos = 32;
    let mut current = num;
    while current > 0 {
        pos -= 1;
        temp[pos] = (current % 10) as u8 + b'0';
        current /= 10;
    }

    let num_len = 32 - pos;
    buf[..num_len].copy_from_slice(&temp[pos..]);

    num_len
}