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

    /// Round to exactly `places` decimal places, **half away from zero**.
    ///
    /// This is the commercial convention (२.५ → ३, -२.५ → -३), not banker's
    /// rounding, because that is what an invoice and an auditor expect.
    /// Division still uses half-even; this is the explicit "make it money" step.
    /// The result always carries exactly `places` digits, so १०० becomes १००.००.
    pub fn round_half_up(&self, places: usize) -> Decimal {
        if self.scale == places {
            return self.clone();
        }
        if self.scale < places {
            // widen: 100 → 100.00, no value change
            return Decimal {
                unscaled: self.unscaled.mul_pow10(places - self.scale),
                scale: places,
            };
        }
        let drop = self.scale - places;
        let p = BigInt::from_i64(1).mul_pow10(drop);
        let neg = self.unscaled.is_negative();
        let mag = self.unscaled.abs();
        let (q, r) = match mag.divmod_trunc(&p) {
            Some(qr) => qr,
            None => return self.clone(),
        };
        // round up when the dropped part is >= half
        let twice = r.mul(&BigInt::from_i64(2));
        let mut out = q;
        if !twice.sub(&p).is_negative() {
            out = out.add(&BigInt::from_i64(1));
        }
        if neg {
            out = out.neg();
        }
        Decimal { unscaled: out, scale: places }
    }

    /// Exact floored remainder — the decimal twin of `BigInt::rem_floor`.
    ///
    /// Line both values up at a common scale and take the floored remainder of
    /// the unscaled integers. Doing it this way has no precision ceiling and
    /// no rounding step, so `(०-७.५) % ३` is `१.५` here exactly as it is in the
    /// reference engine. (Computing it as `a - floor(a/b)*b` would depend on
    /// how the division rounds, which is precisely the trap.)
    pub fn rem_floor(&self, other: &Decimal) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }
        let (ua, ub, scale) = Self::align(self, other);
        Some(Decimal { unscaled: ua.rem_floor(&ub)?, scale })
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

    // The reference engine's default decimal context rounds every operation to
    // 28 significant digits; ours must not. These were found by
    // यादृच्छिकपरीक्षा.py (property-based differential testing).
    #[test]
    fn add_is_exact_at_any_size() {
        let a = Decimal::parse("10592702790675413177270004.3333").unwrap();
        let b = Decimal::parse("0.0001").unwrap();
        assert_eq!(a.add(&b).to_plain_string(), "10592702790675413177270004.3334");
    }

    #[test]
    fn mul_is_exact_at_any_size() {
        let x = Decimal::parse("12345678901234567890.12345").unwrap();
        assert_eq!(x.mul(&x).to_plain_string(),
                   "152415787532388367504953347995733866912.0562399025");
    }

    #[test]
    fn round_half_up_is_the_commercial_convention() {
        let d = |s: &str| Decimal::parse(s).unwrap();
        let r = |s: &str, n: usize| d(s).round_half_up(n).to_plain_string();
        assert_eq!(r("112727.272727", 2), "112727.27");
        assert_eq!(r("2.5", 0), "3");          // half AWAY from zero, not to even
        assert_eq!(r("3.5", 0), "4");
        assert_eq!(r("-2.5", 0), "-3");
        assert_eq!(r("0.125", 2), "0.13");
        assert_eq!(r("0.135", 2), "0.14");
        assert_eq!(r("0.124", 2), "0.12");
        // widening keeps the value and adds the places a total should show
        assert_eq!(r("100", 2), "100.00");
        assert_eq!(r("1.5", 3), "1.500");
        assert_eq!(r("0", 2), "0.00");
        // exact at any size
        assert_eq!(r("123456789012345678901234567890.555", 2),
                   "123456789012345678901234567890.56");
    }

    #[test]
    fn rem_floor_is_exact_and_floored() {
        let d = |s: &str| Decimal::parse(s).unwrap();
        let m = |a: &str, b: &str| d(a).rem_floor(&d(b)).unwrap().to_plain_string();
        // huge numerator, small modulus — this raised DivisionImpossible in the
        // reference engine before the fix
        assert_eq!(m("10592702790675413177270004.3333", "112"), "84.3333");
        // floored, not truncated — the sign of the result follows the divisor
        assert_eq!(m("-7.5", "3"), "1.5");
        assert_eq!(m("7.5", "-3"), "-1.5");
        assert_eq!(m("10.25", "0.5"), "0.25");
        assert_eq!(m("-0.001", "0.3"), "0.299");
        assert!(d("1").rem_floor(&d("0")).is_none());
    }
}
