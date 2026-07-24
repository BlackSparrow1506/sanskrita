// decimal.rs — exact decimal numbers: value = unscaled × 10^(−scale).
//
// This is संस्कृता's correctness promise made real in Rust: ०.१ + ०.२ is
// EXACTLY ०.३, because the digits you typed are kept as digits (a big integer
// plus a decimal-point position) and never converted to a binary fraction.
// Mirrors the Python reference's `decimal.Decimal`.

use std::cmp::Ordering;

use crate::bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    unscaled: BigInt,
    scale: usize, // digits after the decimal point
}

impl Decimal {
    pub fn from_bigint(v: BigInt) -> Self {
        Decimal { unscaled: v, scale: 0 }
    }

    pub fn from_i64(v: i64) -> Self {
        Decimal { unscaled: BigInt::from_i64(v), scale: 0 }
    }

    /// Parse "123" or "123.456" (ASCII digits, optional leading '-').
    pub fn parse(s: &str) -> Option<Self> {
        let (neg, body) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return None;
        }
        let mut parts = body.splitn(2, '.');
        let int_part = parts.next().unwrap_or("");
        let frac_part = parts.next().unwrap_or("");
        if body.matches('.').count() > 1 {
            return None;
        }
        let digits = format!("{}{}", int_part, frac_part);
        let mut unscaled = BigInt::from_digits(&digits);
        if neg && !unscaled.is_zero() {
            unscaled = unscaled.neg();
        }
        Some(Decimal { unscaled, scale: frac_part.len() })
    }

    pub fn is_integer(&self) -> bool {
        self.scale == 0
    }

    pub fn is_zero(&self) -> bool {
        self.unscaled.is_zero()
    }

    pub fn to_bigint_if_integral(&self) -> Option<BigInt> {
        if self.scale == 0 {
            return Some(self.unscaled.clone());
        }
        let p = BigInt::from_i64(1).mul_pow10(self.scale);
        let (q, r) = self.unscaled.divmod_trunc(&p)?;
        if r.is_zero() { Some(q) } else { None }
    }

    /// Rescale both operands to a common scale.
    fn align(a: &Decimal, b: &Decimal) -> (BigInt, BigInt, usize) {
        let scale = a.scale.max(b.scale);
        let ua = a.unscaled.mul_pow10(scale - a.scale);
        let ub = b.unscaled.mul_pow10(scale - b.scale);
        (ua, ub, scale)
    }

    pub fn add(&self, other: &Decimal) -> Decimal {
        let (ua, ub, scale) = Self::align(self, other);
        Decimal { unscaled: ua.add(&ub), scale }
    }

    pub fn sub(&self, other: &Decimal) -> Decimal {
        let (ua, ub, scale) = Self::align(self, other);
        Decimal { unscaled: ua.sub(&ub), scale }
    }

    pub fn mul(&self, other: &Decimal) -> Decimal {
        Decimal {
            unscaled: self.unscaled.mul(&other.unscaled),
            scale: self.scale + other.scale,
        }
    }

    /// Exact division where possible; otherwise rounded to `max_digits`
    /// significant fractional digits (the reference uses 28-digit precision).
    pub fn div(&self, other: &Decimal, max_digits: usize) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }
        // value = (ua × 10^-sa) / (ub × 10^-sb) = (ua / ub) × 10^(sb-sa)
        // Scale the numerator up by max_digits so the quotient carries enough
        // fractional digits, then trim trailing zeros for an exact result.
        let extra = max_digits;
        let num = self.unscaled.mul_pow10(extra + other.scale);
        let den = other.unscaled.mul_pow10(self.scale);
        let (q, r) = num.divmod_trunc(&den)?;
        let mut out = Decimal { unscaled: q, scale: extra };
        if r.is_zero() {
            out.trim_zeros();
        }
        Some(out)
    }

    /// Remove trailing fractional zeros (०.३० → ०.३, ५.० → ५).
    fn trim_zeros(&mut self) {
        while self.scale > 0 {
            let ten = BigInt::from_i64(10);
            match self.unscaled.divmod_trunc(&ten) {
                Some((q, r)) if r.is_zero() => {
                    self.unscaled = q;
                    self.scale -= 1;
                }
                _ => break,
            }
        }
    }

    pub fn cmp_to(&self, other: &Decimal) -> Ordering {
        let (ua, ub, _) = Self::align(self, other);
        ua.cmp_to(&ub)
    }

    pub fn eq_value(&self, other: &Decimal) -> bool {
        self.cmp_to(other) == Ordering::Equal
    }

    pub fn neg(&self) -> Decimal {
        Decimal { unscaled: self.unscaled.neg(), scale: self.scale }
    }

    /// Plain decimal text, never scientific notation (matches the reference's
    /// format(v, "f")).
    pub fn to_plain_string(&self) -> String {
        let neg = self.unscaled.is_negative();
        let digits = self.unscaled.abs().digits();
        let s = if self.scale == 0 {
            digits
        } else if digits.len() > self.scale {
            let split = digits.len() - self.scale;
            format!("{}.{}", &digits[..split], &digits[split..])
        } else {
            let zeros = "0".repeat(self.scale - digits.len());
            format!("0.{}{}", zeros, digits)
        };
        if neg { format!("-{}", s) } else { s }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::parse(s).unwrap()
    }

    // THE promise: this is false in Python/Java floats, true here.
    #[test]
    fn point_one_plus_point_two_is_exactly_point_three() {
        let r = d("0.1").add(&d("0.2"));
        assert_eq!(r.to_plain_string(), "0.3");
        assert!(r.eq_value(&d("0.3")));
    }

    #[test]
    fn parse_and_print() {
        assert_eq!(d("3.14159").to_plain_string(), "3.14159");
        assert_eq!(d("0.001").to_plain_string(), "0.001");
        assert_eq!(d("-2.5").to_plain_string(), "-2.5");
        assert_eq!(d("42").to_plain_string(), "42");
    }

    #[test]
    fn arithmetic_is_exact() {
        assert_eq!(d("1.005").mul(&d("100")).to_plain_string(), "100.500");
        assert_eq!(d("10").sub(&d("9.99")).to_plain_string(), "0.01");
        // money: 0.1 added ten times is exactly 1
        let mut acc = d("0");
        for _ in 0..10 {
            acc = acc.add(&d("0.1"));
        }
        assert!(acc.eq_value(&d("1")));
    }

    #[test]
    fn division_exact_and_rounded() {
        assert_eq!(d("1").div(&d("4"), 28).unwrap().to_plain_string(), "0.25");
        assert_eq!(d("10").div(&d("5"), 28).unwrap().to_plain_string(), "2");
        let third = d("1").div(&d("3"), 28).unwrap().to_plain_string();
        assert!(third.starts_with("0.3333333333"));
        assert!(d("1").div(&d("0"), 28).is_none());
    }

    #[test]
    fn comparison() {
        assert_eq!(d("0.30").cmp_to(&d("0.3")), Ordering::Equal);
        assert_eq!(d("0.1").cmp_to(&d("0.2")), Ordering::Less);
        assert_eq!(d("-1.5").cmp_to(&d("-2.5")), Ordering::Greater);
    }
}
