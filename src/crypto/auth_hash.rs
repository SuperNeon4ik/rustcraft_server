use sha1::{Digest, Sha1};

// SOURCE: https://gist.github.com/roccodev/8fa130f1946f89702f799f89b8469bc9?permalink_comment_id=4561673#gistcomment-4561673
pub fn calc_hash_from_str(name: &str) -> String {
    let sha = Sha1::new().chain_update(name);
    calc_hash(sha)
}

pub fn calc_hash(sha: Sha1) -> String {
    let mut hash: [u8; 20] = sha.finalize().into();
    let negative = (hash[0] & 0x80) == 0x80;

    // Digest is 20 bytes, so 40 hex digits plus the minus sign if necessary.
    let mut hex = String::with_capacity(40 + negative as usize);
    if negative {
        hex.push('-');

        // two's complement
        let mut carry = true;
        for b in hash.iter_mut().rev() {
            (*b, carry) = (!*b).overflowing_add(carry as u8);
        }
    }
    hex.extend(
        hash.into_iter()
            // extract hex digits
            .flat_map(|x| [x >> 4, x & 0xf])
            // skip leading zeroes
            .skip_while(|&x| x == 0)
            .map(|x| char::from_digit(x as u32, 16).expect("x is always valid base16")),
    );
    hex
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cases() {
        let pairs = &[
            ("Notch", "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48"),
            ("jeb_", "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1"),
            ("simon", "88e16a1019277b15d58faf0541e11910eb756f6"),
        ];
        for (input, output) in pairs {
            assert_eq!(&calc_hash_from_str(input), output);
        }
    }
}