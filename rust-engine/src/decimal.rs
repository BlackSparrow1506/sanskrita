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

    #[allow(dead_code)] // used by later slices and by external callers
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

    #[allow(dead_code)] // used by the reference-parity checks and later slices
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

    /// Division with the reference's semantics: exact when the quotient fits,
    /// otherwise rounded to `prec` SIGNIFICANT digits, half-even.
    ///
    /// (Python's `decimal` defaults to 28 significant digits — not 28
    /// fractional digits — so २७५/३ is ९१.६६…६७, with the final digit rounded.
    /// Truncating instead would silently disagree with the reference.)
    pub fn div(&self, other: &Decimal, prec: usize) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }
        if self.is_zero() {
            return Some(Decimal { unscaled: BigInt::zero(), scale: 0 });
        }
        let prec = prec.max(1);
        // value = (ua / ub) × 10^(sb − sa)
        let n0 = self.unscaled.abs();
        let d = other.unscaled.abs();
        let e: i64 = other.scale as i64 - self.scale as i64;
        let negative = self.unscaled.is_negative() != other.unscaled.is_negative();

        // Scale the numerator until the quotient carries `prec` digits.
        let mut k: usize = 0;
        let mut n = n0;
        let (mut q, mut r) = n.divmod_trunc(&d)?;
        while q.digits().len() < prec {
            n = n.mul_pow10(1);
            k += 1;
            let (q2, r2) = n.divmod_trunc(&d)?;
            q = q2;
            r = r2;
            if k > 10_000 {
                break; // safety valve; unreachable for sane inputs
            }
        }

        // Round half-even on the discarded remainder.
        let exact = r.is_zero();
        if !exact {
            let twice = r.mul(&BigInt::from_i64(2));
            let cmp = twice.cmp_to(&d);
            let round_up = match cmp {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Equal => {
                    // tie → round to even
                    let last = q.digits().chars().last().unwrap_or('0');
                    (last as u8 - b'0') % 2 == 1
                }
                std::cmp::Ordering::Less => false,
            };
            if round_up {
                q = q.add(&BigInt::from_i64(1));
                // rounding may add a digit (९९९ → १०००): drop it again
                if q.digits().len() > prec {
                    let (q2, _) = q.divmod_trunc(&BigInt::from_i64(10))?;
                    q = q2;
                    k = k.saturating_sub(1);
                }
            }
        }

        if negative {
            q = q.neg();
        }
        // value = q × 10^(e − k)
        let shift = e - k as i64;
        let mut out = if shift >= 0 {
            Decimal { unscaled: q.mul_pow10(shift as usize), scale: 0 }
        } else {
            Decimal { unscaled: q, scale: (-shift) as usize }
        };
        if exact {
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
        assert!(d("1").div(&d("0"), 28).is_none());
        // 28 SIGNIFICANT digits, half-even — byte-identical to the reference
        assert_eq!(d("1").div(&d("3"), 28).unwrap().to_plain_string(),
                   "0.3333333333333333333333333333");
        assert_eq!(d("275").div(&d("3"), 28).unwrap().to_plain_string(),
                   "91.66666666666666666666666667");
        assert_eq!(d("2").div(&d("3"), 28).unwrap().to_plain_string(),
                   "0.6666666666666666666666666667");
        assert_eq!(d("-275").div(&d("3"), 28).unwrap().to_plain_string(),
                   "-91.66666666666666666666666667");
        assert_eq!(d("0").div(&d("7"), 28).unwrap().to_plain_string(), "0");
        assert_eq!(d("0.1").div(&d("0.4"), 28).unwrap().to_plain_string(), "0.25");
    }

    #[test]
    fn comparison() {
        assert_eq!(d("0.30").cmp_to(&d("0.3")), Ordering::Equal);
        assert_eq!(d("0.1").cmp_to(&d("0.2")), Ordering::Less);
        assert_eq!(d("-1.5").cmp_to(&d("-2.5")), Ordering::Greater);
    }
}
