// bigint.rs — arbitrary-precision signed integers, dependency-free.
//
// Why hand-rolled: संस्कृता's identity is exactness (०.१ + ०.२ = ०.३, १०० !
// computed exactly). The Python reference has bignums for free; वेगः must match
// or it is a different language. Keeping the engine dependency-free also keeps
// the supply chain trivial to audit — appropriate for a language meant for
// schools.
//
// Representation: sign + magnitude in base 1e9 limbs, least-significant first,
// normalized (no leading zero limbs; zero has an empty limb vector and sign 0).

use std::cmp::Ordering;

const BASE: u64 = 1_000_000_000;
const BASE_DIGITS: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInt {
    /// -1, 0, or 1
    sign: i8,
    /// base-1e9 limbs, little-endian; empty iff sign == 0
    mag: Vec<u32>,
}

impl BigInt {
    pub fn zero() -> Self {
        BigInt { sign: 0, mag: Vec::new() }
    }

    pub fn from_i64(v: i64) -> Self {
        if v == 0 {
            return BigInt::zero();
        }
        let sign = if v < 0 { -1 } else { 1 };
        // i128 avoids the i64::MIN negation trap
        let mut n = (v as i128).unsigned_abs();
        let mut mag = Vec::new();
        while n > 0 {
            mag.push((n % BASE as u128) as u32);
            n /= BASE as u128;
        }
        BigInt { sign, mag }
    }

    pub fn is_zero(&self) -> bool {
        self.sign == 0
    }

    pub fn is_negative(&self) -> bool {
        self.sign < 0
    }

    /// Parse a run of ASCII digits (no sign). Empty input yields zero.
    pub fn from_digits(s: &str) -> Self {
        let s = s.trim_start_matches('0');
        if s.is_empty() {
            return BigInt::zero();
        }
        let bytes = s.as_bytes();
        let mut mag = Vec::with_capacity(bytes.len() / BASE_DIGITS + 1);
        let mut i = bytes.len();
        while i > 0 {
            let start = i.saturating_sub(BASE_DIGITS);
            let chunk = &s[start..i];
            mag.push(chunk.parse::<u32>().unwrap_or(0));
            i = start;
        }
        let mut out = BigInt { sign: 1, mag };
        out.trim();
        out
    }

    fn trim(&mut self) {
        while let Some(&0) = self.mag.last() {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.sign = 0;
        }
    }

    pub fn neg(&self) -> Self {
        let mut out = self.clone();
        out.sign = -out.sign;
        out
    }

    pub fn abs(&self) -> Self {
        let mut out = self.clone();
        if out.sign < 0 {
            out.sign = 1;
        }
        out
    }

    // ---- magnitude helpers ----

    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        if a.len() != b.len() {
            return a.len().cmp(&b.len());
        }
        for i in (0..a.len()).rev() {
            if a[i] != b[i] {
                return a[i].cmp(&b[i]);
            }
        }
        Ordering::Equal
    }

    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry: u64 = 0;
        for i in 0..a.len().max(b.len()) {
            let x = *a.get(i).unwrap_or(&0) as u64;
            let y = *b.get(i).unwrap_or(&0) as u64;
            let s = x + y + carry;
            out.push((s % BASE) as u32);
            carry = s / BASE;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }

    /// a - b, requires |a| >= |b|
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow: i64 = 0;
        for i in 0..a.len() {
            let x = a[i] as i64;
            let y = *b.get(i).unwrap_or(&0) as i64;
            let mut d = x - y - borrow;
            if d < 0 {
                d += BASE as i64;
                borrow = 1;
            } else {
                borrow = 0;
            }
            out.push(d as u32);
        }
        out
    }

    fn mul_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        if a.is_empty() || b.is_empty() {
            return Vec::new();
        }
        let mut out = vec![0u64; a.len() + b.len()];
        for (i, &x) in a.iter().enumerate() {
            let mut carry: u64 = 0;
            for (j, &y) in b.iter().enumerate() {
                let cur = out[i + j] + (x as u64) * (y as u64) + carry;
                out[i + j] = cur % BASE;
                carry = cur / BASE;
            }
            let mut k = i + b.len();
            while carry > 0 {
                let cur = out[k] + carry;
                out[k] = cur % BASE;
                carry = cur / BASE;
                k += 1;
            }
        }
        // The buffer is allocated at a.len()+b.len() limbs, but the product
        // often needs one fewer. The leading zero MUST go: every magnitude in
        // this module is required to be trimmed, because cmp_mag compares
        // lengths before contents. An untrimmed value here made b*q look
        // larger than it is, which drove divmod_mag's binary search to
        // saturate at BASE-1 for every divisor of two limbs or more.
        while let Some(&0) = out.last() {
            out.pop();
        }
        out.into_iter().map(|v| v as u32).collect()
    }

    /// Divide magnitude by a single limb-sized divisor; returns (quotient, remainder).
    fn divmod_small(a: &[u32], d: u64) -> (Vec<u32>, u64) {
        let mut out = vec![0u32; a.len()];
        let mut rem: u64 = 0;
        for i in (0..a.len()).rev() {
            let cur = rem * BASE + a[i] as u64;
            out[i] = (cur / d) as u32;
            rem = cur % d;
        }
        (out, rem)
    }

    /// Schoolbook long division on magnitudes → (quotient, remainder).
    fn divmod_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        if Self::cmp_mag(a, b) == Ordering::Less {
            return (Vec::new(), a.to_vec());
        }
        if b.len() == 1 {
            let (q, r) = Self::divmod_small(a, b[0] as u64);
            return (q, if r == 0 { Vec::new() } else { vec![r as u32] });
        }
        // long division, one limb of quotient at a time, binary-searching each
        let mut quotient = vec![0u32; a.len()];
        let mut rem: Vec<u32> = Vec::new();
        for i in (0..a.len()).rev() {
            rem.insert(0, a[i]);
            while let Some(&0) = rem.last() {
                rem.pop();
            }
            // find the largest q with b*q <= rem  (0 <= q < BASE)
            let (mut lo, mut hi, mut best) = (0u64, BASE - 1, 0u64);
            while lo <= hi {
                let mid = (lo + hi) / 2;
                let t = Self::mul_mag(b, &[(mid % BASE) as u32]);
                if Self::cmp_mag(&t, &rem) != Ordering::Greater {
                    best = mid;
                    if mid == BASE - 1 { break; }
                    lo = mid + 1;
                } else {
                    if mid == 0 { break; }
                    hi = mid - 1;
                }
            }
            quotient[i] = best as u32;
            if best > 0 {
                let t = Self::mul_mag(b, &[best as u32]);
                rem = Self::sub_mag(&rem, &t);
                while let Some(&0) = rem.last() {
                    rem.pop();
                }
            }
        }
        (quotient, rem)
    }

    // ---- signed arithmetic ----

    pub fn add(&self, other: &BigInt) -> BigInt {
        if self.is_zero() {
            return other.clone();
        }
        if other.is_zero() {
            return self.clone();
        }
        if self.sign == other.sign {
            let mut out = BigInt { sign: self.sign, mag: Self::add_mag(&self.mag, &other.mag) };
            out.trim();
            return out;
        }
        match Self::cmp_mag(&self.mag, &other.mag) {
            Ordering::Equal => BigInt::zero(),
            Ordering::Greater => {
                let mut out = BigInt { sign: self.sign, mag: Self::sub_mag(&self.mag, &other.mag) };
                out.trim();
                out
            }
            Ordering::Less => {
                let mut out = BigInt { sign: other.sign, mag: Self::sub_mag(&other.mag, &self.mag) };
                out.trim();
                out
            }
        }
    }

    pub fn sub(&self, other: &BigInt) -> BigInt {
        self.add(&other.neg())
    }

    pub fn mul(&self, other: &BigInt) -> BigInt {
        if self.is_zero() || other.is_zero() {
            return BigInt::zero();
        }
        let mut out = BigInt {
            sign: self.sign * other.sign,
            mag: Self::mul_mag(&self.mag, &other.mag),
        };
        out.trim();
        out
    }

    /// Truncating division and its remainder (like Rust's / and %).
    pub fn divmod_trunc(&self, other: &BigInt) -> Option<(BigInt, BigInt)> {
        if other.is_zero() {
            return None;
        }
        let (q, r) = Self::divmod_mag(&self.mag, &other.mag);
        let mut quo = BigInt { sign: self.sign * other.sign, mag: q };
        let mut rem = BigInt { sign: self.sign, mag: r };
        quo.trim();
        rem.trim();
        Some((quo, rem))
    }

    /// Python-compatible floored modulo (-७ % ३ == २).
    pub fn rem_floor(&self, other: &BigInt) -> Option<BigInt> {
        let (_, r) = self.divmod_trunc(other)?;
        if r.is_zero() || (r.is_negative() == other.is_negative()) {
            Some(r)
        } else {
            Some(r.add(other))
        }
    }

    pub fn cmp_to(&self, other: &BigInt) -> Ordering {
        if self.sign != other.sign {
            return self.sign.cmp(&other.sign);
        }
        let m = Self::cmp_mag(&self.mag, &other.mag);
        if self.sign < 0 { m.reverse() } else { m }
    }

    /// Multiply by 10^n — used by the decimal layer to align scales.
    pub fn mul_pow10(&self, n: usize) -> BigInt {
        if self.is_zero() || n == 0 {
            return self.clone();
        }
        let mut out = self.clone();
        let mut left = n;
        while left >= BASE_DIGITS {
            out.mag.insert(0, 0);
            left -= BASE_DIGITS;
        }
        if left > 0 {
            let mut p = BigInt::from_i64(10i64.pow(left as u32));
            p = out.mul(&p);
            out = p;
        }
        out.trim();
        out
    }

    /// Decimal digit string of the magnitude (no sign).
    pub fn digits(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut s = String::new();
        for (i, limb) in self.mag.iter().enumerate().rev() {
            if i == self.mag.len() - 1 {
                s.push_str(&limb.to_string());
            } else {
                s.push_str(&format!("{:0width$}", limb, width = BASE_DIGITS));
            }
        }
        s
    }

    pub fn to_string_signed(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let d = self.digits();
        if self.sign < 0 { format!("-{}", d) } else { d }
    }

    #[allow(dead_code)] // used by later slices (list indexing, string ops)
    pub fn to_i64(&self) -> Option<i64> {
        let s = self.to_string_signed();
        s.parse::<i64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> BigInt {
        if let Some(rest) = s.strip_prefix('-') {
            BigInt::from_digits(rest).neg()
        } else {
            BigInt::from_digits(s)
        }
    }

    #[test]
    fn basics() {
        assert_eq!(b("0").to_string_signed(), "0");
        assert_eq!(b("123456789012345678901234567890").to_string_signed(),
                   "123456789012345678901234567890");
        assert_eq!(BigInt::from_i64(-42).to_string_signed(), "-42");
        assert_eq!(BigInt::from_i64(i64::MIN).to_string_signed(),
                   i64::MIN.to_string());
    }

    #[test]
    fn add_sub() {
        assert_eq!(b("999999999999").add(&b("1")).to_string_signed(), "1000000000000");
        assert_eq!(b("1000000000000").sub(&b("1")).to_string_signed(), "999999999999");
        assert_eq!(b("5").sub(&b("8")).to_string_signed(), "-3");
        assert_eq!(b("-5").add(&b("5")).to_string_signed(), "0");
    }

    #[test]
    fn mul_big() {
        // 20! = 2432902008176640000 (beyond i32, within i64)
        let mut f = BigInt::from_i64(1);
        for i in 1..=20 {
            f = f.mul(&BigInt::from_i64(i));
        }
        assert_eq!(f.to_string_signed(), "2432902008176640000");
        // 25! overflows i64 — the whole point of bignums
        for i in 21..=25 {
            f = f.mul(&BigInt::from_i64(i));
        }
        assert_eq!(f.to_string_signed(), "15511210043330985984000000");
    }

    #[test]
    fn div_mod() {
        let (q, r) = b("1000000000000").divmod_trunc(&b("7")).unwrap();
        assert_eq!(q.to_string_signed(), "142857142857");
        assert_eq!(r.to_string_signed(), "1");
        let (q2, r2) = b("15511210043330985984000000")
            .divmod_trunc(&b("123456789")).unwrap();
        assert_eq!(q2.mul(&b("123456789")).add(&r2).to_string_signed(),
                   "15511210043330985984000000");
    }

    #[test]
    fn floored_modulo_matches_python() {
        assert_eq!(b("-7").rem_floor(&b("3")).unwrap().to_string_signed(), "2");
        assert_eq!(b("7").rem_floor(&b("-3")).unwrap().to_string_signed(), "-2");
        assert_eq!(b("7").rem_floor(&b("3")).unwrap().to_string_signed(), "1");
    }

    #[test]
    fn compare() {
        assert_eq!(b("100").cmp_to(&b("99")), Ordering::Greater);
        assert_eq!(b("-100").cmp_to(&b("99")), Ordering::Less);
        assert_eq!(b("-100").cmp_to(&b("-99")), Ordering::Less);
        assert_eq!(b("42").cmp_to(&b("42")), Ordering::Equal);
    }

    #[test]
    fn pow10() {
        assert_eq!(b("123").mul_pow10(0).to_string_signed(), "123");
        assert_eq!(b("123").mul_pow10(3).to_string_signed(), "123000");
        assert_eq!(b("1").mul_pow10(20).to_string_signed(),
                   "100000000000000000000");
    }

    // Pin the primitives `Decimal::div` stands on. If one of these ever fails,
    // the culprit is named here instead of only showing up as a wrong quotient.
    #[test]
    fn powers_of_ten_round_trip_through_digits() {
        for k in 0..60usize {
            let v = BigInt::from_i64(1).mul_pow10(k);
            let mut want = String::from("1");
            want.push_str(&"0".repeat(k));
            assert_eq!(v.digits(), want, "10^{} rendered wrong", k);
            assert_eq!(v.digits().len(), k + 1, "10^{} has wrong digit count", k);
        }
    }

    #[test]
    fn shifted_division_matches_by_hand() {
        // the operands from the property-test failure, done exactly
        let a = BigInt::from_digits("113").mul_pow10(50);
        assert_eq!(a.digits().len(), 53);
        let b = BigInt::from_digits("751306816453898389858997");
        assert_eq!(b.digits().len(), 24);
        let (q, _) = a.divmod_trunc(&b).unwrap();
        assert_eq!(q.digits(), "15040459839476765103808918240");
        assert_eq!(q.digits().len(), 29);
    }

    #[test]
    fn digits_len_is_the_significant_digit_count() {
        let v = BigInt::from_digits("1504045983947676510380891824");
        assert_eq!(v.digits().len(), 28);
        assert_eq!(BigInt::from_digits("1000000000").digits().len(), 10);
        assert_eq!(BigInt::from_i64(0).digits(), "0");
    }

    // Every magnitude in this module must be trimmed, because cmp_mag compares
    // lengths before contents. mul_mag allocates a.len()+b.len() limbs and the
    // product usually needs one fewer, so it is the one place that can hand
    // back an untrimmed value. It did, and it cost three wrong rewrites of
    // Decimal::div before this test existed to say so.
    #[test]
    fn mul_mag_never_returns_a_leading_zero_limb() {
        let cases: [(&str, &str); 5] = [
            ("751306816453898389858997", "1"),
            ("751306816453898389858997", "999999999"),
            ("1000000000", "1000000000"),
            ("999999999", "999999999"),
            ("123456789012345678901234567890", "7"),
        ];
        for (x, y) in cases {
            let p = BigInt::mul_mag(&BigInt::from_digits(x).mag, &BigInt::from_digits(y).mag);
            assert_ne!(p.last(), Some(&0), "{} * {} left a zero limb on top", x, y);
        }
    }

    // divmod_small handles one-limb divisors, so anything below 10^9 took a
    // different path and always worked. Divisors of two limbs or more went
    // through the binary search — the region that was broken and untested.
    #[test]
    fn division_is_exact_for_multi_limb_divisors() {
        let cases: [(&str, &str, &str, &str); 6] = [
            // (dividend, divisor, quotient, remainder)
            ("11300000000000000000000000000000000000000000000000000",
             "751306816453898389858997",
             "15040459839476765103808918240", "419967613154735298594720"),
            ("1000000000000000000", "1000000000", "1000000000", "0"),
            ("1000000000000000000", "1000000001", "999999999", "1"),
            ("123456789012345678901234567890", "98765432109876543210",
             "1249999988", "60185185207253086410"),
            ("340282366920938463463374607431768211456", "18446744073709551616",
             "18446744073709551616", "0"),
            ("999999999999999999999999999999", "999999999999",
             "1000000000001000000", "999999"),
        ];
        for (n, d, wq, wr) in cases {
            let (q, r) = BigInt::from_digits(n).divmod_trunc(&BigInt::from_digits(d)).unwrap();
            assert_eq!(q.digits(), wq, "{} / {} quotient", n, d);
            assert_eq!(r.digits(), wr, "{} / {} remainder", n, d);
        }
    }
}
