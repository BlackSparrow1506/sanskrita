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

/// `floor(a × 10^s / b)` with its remainder. A negative `s` scales the
/// DIVISOR instead, so every value stays an integer.
fn divide_shifted(a: &BigInt, b: &BigInt, s: i64) -> Option<(BigInt, BigInt)> {
    if s >= 0 {
        a.mul_pow10(s as usize).divmod_trunc(b)
    } else {
        a.divmod_trunc(&b.mul_pow10((-s) as usize))
    }
}

fn is_odd(v: &BigInt) -> bool {
    match v.divmod_trunc(&BigInt::from_i64(2)) {
        Some((_, r)) => !r.is_zero(),
        None => false,
    }
}

/// `(q, k)` with `a / b == q / 10^k` exactly, or `None` when `a / b` repeats.
/// `a` and `b` are magnitudes, `b` non-zero.
///
/// A quotient terminates exactly when, in lowest terms, its denominator has no
/// prime factor besides 2 and 5. Write `b = 2^i · 5^j · t` with `t` coprime to
/// ten; then the quotient terminates precisely when **t divides a** — a power
/// of ten can supply twos and fives, but never a factor of `t`. So no gcd is
/// needed: strip the twos and fives, test one division, and `k = max(i, j)` is
/// large enough for `a · 10^k` to be divisible by `b`.
fn exact_quotient(a: &BigInt, b: &BigInt) -> Option<(BigInt, i64)> {
    let two = BigInt::from_i64(2);
    let five = BigInt::from_i64(5);
    let mut t = b.clone();
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    loop {
        match t.divmod_trunc(&two) {
            Some((q, r)) if r.is_zero() => { t = q; i += 1; }
            _ => break,
        }
    }
    loop {
        match t.divmod_trunc(&five) {
            Some((q, r)) if r.is_zero() => { t = q; j += 1; }
            _ => break,
        }
    }
    let (_, rem) = a.divmod_trunc(&t)?;
    if !rem.is_zero() {
        return None;                      // the quotient repeats
    }
    let k = if i > j { i } else { j };
    let (q, r) = a.mul_pow10(k as usize).divmod_trunc(b)?;
    if !r.is_zero() {
        return None;                      // unreachable if the reasoning holds
    }
    Some((q, k))
}

/// Write a quotient the way the reference writes it. Both paths through `div`
/// end here, so they cannot drift apart.
///
/// `ideal` is exp(dividend) − exp(divisor). Two rules, both observable:
///
/// 1. An exact quotient sheds trailing zeros, but only down to `ideal`. So
///    २४४.२० / २ is १२२.१०, not १२२.१ — that zero is the precision the
///    operands claimed, and a money column depends on keeping it. An inexact
///    quotient (`shed == false`) keeps all of its significant digits.
/// 2. A whole number is written as a whole number, exact or not — the
///    reference's `quantize(Decimal(1))`. This is what turns an inexact
///    १.०००…० back into १. It is all-or-nothing on purpose: १२२.१० is not a
///    whole number, so it must not lose its zero here.
///
/// The result never carries a positive exponent; those are materialised into
/// digits, because वेगः has no way to represent one and the reference must not
/// let a later multiplication inherit it.
fn write_quotient(mut qi: BigInt, mut exp: i64, ideal: i64, shed: bool, negative: bool) -> Decimal {
    if shed {
        let ten = BigInt::from_i64(10);
        while exp < ideal && !qi.is_zero() {
            let next = match qi.divmod_trunc(&ten) {
                Some((q, r)) if r.is_zero() => Some(q),
                _ => None,
            };
            match next {
                Some(q) => { qi = q; exp += 1; }
                None => break,
            }
        }
    }
    if exp < 0 {
        let p = BigInt::from_i64(1).mul_pow10((-exp) as usize);
        let whole = match qi.divmod_trunc(&p) {
            Some((q, r)) if r.is_zero() => Some(q),
            _ => None,
        };
        if let Some(q) = whole {
            qi = q;
            exp = 0;
        }
    }
    if negative {
        qi = qi.neg();
    }
    if exp >= 0 {
        Decimal { unscaled: qi.mul_pow10(exp as usize), scale: 0 }
    } else {
        Decimal { unscaled: qi, scale: (-exp) as usize }
    }
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
    /// `self / other` — **exact when it divides evenly**, at any size; and
    /// otherwise rounded to `prec` SIGNIFICANT digits, half-even.
    ///
    /// The exact case is not an optimisation, it is the promise: a language
    /// that advertises exact decimals must not round ३०६५०९…/२२५ just because
    /// the answer needs 55 digits. Only a quotient that repeats forever — 1/3
    /// and its kind — has to be cut, and that is the only place `prec` applies.
    ///
    /// Two earlier versions of this got the digit count wrong: one counted as
    /// it grew the numerator, the other corrected a starting estimate with a
    /// loop. Both could overshoot, and both shipped a quotient with 29 or 30
    /// significant digits instead of 28.
    ///
    /// This version does not *count* at all. It divides once with five guard
    /// digits, then takes the first `prec` characters of the quotient and
    /// rounds on the ones it dropped — no loop that can run an extra time. The
    /// whole thing, string handling included, was transcribed into Python and
    /// checked against the reference on 20,000 random operand pairs before it
    /// was written here.
    pub fn div(&self, other: &Decimal, prec: usize) -> Option<Decimal> {
        if other.is_zero() {
            return None;
        }
        if self.is_zero() {
            return Some(Decimal { unscaled: BigInt::zero(), scale: 0 });
        }
        let prec = prec.max(1);
        let a = self.unscaled.abs();
        let b = other.unscaled.abs();
        let negative = self.unscaled.is_negative() != other.unscaled.is_negative();
        let e: i64 = other.scale as i64 - self.scale as i64;   // value = (a/b)×10^e

        // §4.1: `/` is EXACT when it divides evenly — at any size, with no
        // digit ceiling. Only a quotient that would repeat forever is rounded.
        // The reference used to cap this at 28 digits too, which quietly broke
        // its own promise on a division that came out to 55 exact digits.
        if let Some((c, k)) = exact_quotient(&a, &b) {
            return Some(write_quotient(c, e - k, e, true, negative));
        }

        // Divide with guard digits so the quotient certainly has more than
        // `prec` digits; then we only ever cut, never extend.
        let la = a.digits().len() as i64;
        let lb = b.digits().len() as i64;
        let mut s: i64 = (lb - la) + prec as i64 + 5;
        let (mut q, mut r) = divide_shifted(&a, &b, s)?;
        let mut text = q.digits();
        while text.len() <= prec {
            s += 5;
            let t = divide_shifted(&a, &b, s)?;
            q = t.0; r = t.1;
            text = q.digits();
        }

        let mut extra = (text.len() - prec) as i64;
        let (keep, rest) = text.split_at(prec);
        let mut qi = BigInt::from_digits(keep);

        // Round on what we dropped: > half up, < half down, exactly half to even.
        //
        // The "exactly half" arm is unreachable now that `exact_quotient` has
        // already returned for every terminating quotient: a quotient that
        // repeats leaves a non-zero remainder at every shift, so the dropped
        // tail is never exactly ५०००…. It is kept because it costs nothing and
        // it is what makes this function correct on its own terms, independent
        // of what the caller above it happens to have filtered out.
        let first = rest.as_bytes()[0];
        let rest_nonzero = rest[1..].bytes().any(|c| c != b'0') || !r.is_zero();
        let round_up = first > b'5'
            || (first == b'5' && rest_nonzero)
            || (first == b'5' && !rest_nonzero && is_odd(&qi));
        if round_up {
            qi = qi.add(&BigInt::from_i64(1));
            if qi.digits().len() > prec {          // ९९९ → १००० : one digit too many
                let (q2, _) = qi.divmod_trunc(&BigInt::from_i64(10))?;
                qi = q2;
                extra += 1;
            }
        }

        // Was the ORIGINAL division exact? Only if every digit we dropped was a
        // zero and the shifted division left no remainder. (`exact_quotient`
        // above has already returned for every terminating quotient, so this
        // should now always be false — it is kept because it costs nothing and
        // it is the behaviour 20,000 validated cases were checked against.)
        let exact = !rest_nonzero && first == b'0';

        Some(write_quotient(qi, e - s + extra, e, exact, negative))
    }

    // NOTE: there is deliberately no general `trim_zeros` here. Trailing zeros
    // are not noise in this language — ०.३० and ०.३ are the same number but not
    // the same *statement* about precision, and a money column depends on the
    // difference. Only `div` may drop them, under the two narrow rules written
    // out above. A blanket trim was the bug that made २४४.२० / २ print १२२.१.

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

    // The five programs यादृच्छिकपरीक्षा.py found in 8 seconds, reduced to their
    // divisions. Expected values come from the reference engine.
    #[test]
    // How a quotient is WRITTEN is part of the answer. Every expectation below
    // was produced by running the reference engine, not typed from memory.
    #[test]
    fn division_writes_the_result_the_way_the_reference_does() {
        let d = |s: &str| Decimal::parse(s).unwrap();
        let q = |a: &str, b: &str| d(a).div(&d(b), 28).unwrap().to_plain_string();

        // Exact division sheds trailing zeros only down to the ideal exponent,
        // exp(dividend) − exp(divisor). The zero in १२२.१० is the operands'
        // claimed precision, and a money column depends on keeping it.
        assert_eq!(q("244.20", "2"), "122.10");   // ideal exp −2
        assert_eq!(q("1221.0", "10"), "122.1");   // ideal exp −1
        assert_eq!(q("0.30", "3"), "0.10");       // ideal exp −2
        assert_eq!(q("1.000", "8"), "0.125");
        assert_eq!(q("2.50", "2"), "1.25");

        // …but a whole number is written as one, which can go *past* the ideal
        // exponent: ६.०० / ३ is २, not २.००.
        assert_eq!(q("6.00", "3"), "2");
        assert_eq!(q("10", "5"), "2");
        assert_eq!(q("7", "0.5"), "14");
        assert_eq!(q("100", "8"), "12.5");

        // And the collapse applies to INEXACT results too. ब / (ब+१) is
        // 0.999… rounded to 28 digits — exactly १.०००…० — and prints as १.
        // This is the seed-151 divergence.
        let big = "537222945737715537323242393024304946666700270829461708";
        let big1 = "537222945737715537323242393024304946666700270829461709";
        assert_eq!(q(big, big1), "1");

        // Zero over anything is zero, at scale 0 (seed 297).
        assert_eq!(q("0", "968.005"), "0");
        assert_eq!(q("0", "-5"), "0");
    }

    // `/` is exact when it divides evenly — with NO digit ceiling. The 28-digit
    // rule applies only to quotients that repeat forever. Both engines used to
    // cap this, which silently contradicted the language's headline promise.
    #[test]
    fn division_that_divides_evenly_is_exact_at_any_size() {
        let d = |s: &str| Decimal::parse(s).unwrap();
        let q = |a: &str, b: &str| d(a).div(&d(b), 28).unwrap().to_plain_string();

        // seed 1171: ग*ग / २२५ divides evenly into 55 digits. वेगः had this
        // right through its integer fast path; the reference did not.
        assert_eq!(
            q("306509434762526828044877323004762441933346472680260250000", "225"),
            "1362264154500119235755010324465610853037095434134490000");

        // A power of two always terminates, however long the answer runs.
        assert_eq!(q("1", "1024"), "0.0009765625");
        assert_eq!(q("1", "512"), "0.001953125");
        assert_eq!(q("3", "4096"), "0.000732421875");
        // …and so does a power of five.
        assert_eq!(q("1", "390625"), "0.00000256");

        // 60 digits over 8 — exact, and far past 28 significant digits.
        assert_eq!(
            q("1000000000000000000000000000000000000000000000000000000000000", "8"),
            "125000000000000000000000000000000000000000000000000000000000");

        // A repeating quotient still stops at 28 significant digits.
        assert_eq!(q("1", "3"), "0.3333333333333333333333333333");
        assert_eq!(q("1", "6"), "0.1666666666666666666666666667");
        assert_eq!(q("1", "7"), "0.1428571428571428571428571429");
    }

    #[test]
    fn division_matches_the_reference_on_the_property_failures() {
        let d = |s: &str| Decimal::parse(s).unwrap();
        let q = |a: &str, b: &str| d(a).div(&d(b), 28).unwrap().to_plain_string();

        // seed 12 — वेगः used to return 30 significant digits, not 28.
        // Both spellings of the divisor are pinned, because getting one of them
        // wrong in an *expectation* is how a green test hides a red engine.
        assert_eq!(q("113", "751306816453898389858997"),          // 24 digits
                   "0.0000000000000000000001504045983947676510380891824");
        assert_eq!(q("113", "7513068164538983898589997"),         // 25 digits
                   "0.00000000000000000000001504045983947676510380886419");
        // seed 25 — a very small quotient, where the shift is large
        assert_eq!(q("25", "10229998015538679119744060684500.1"),
                   "0.000000000000000000000000000002443793240431394121674023703");
        // a plain case, to pin the significant-digit rule itself
        assert_eq!(q("1", "3"), "0.3333333333333333333333333333");
        assert_eq!(q("275", "3"), "91.66666666666666666666666667");
        assert_eq!(q("2", "3"), "0.6666666666666666666666666667");
        // exact divisions keep their exact (short) form
        assert_eq!(q("1", "4"), "0.25");
        assert_eq!(q("10", "5"), "2");
        // `prec` applies ONLY to a quotient that repeats. १/८ terminates, so it
        // comes back exact even when a single significant digit was asked for —
        // this assertion used to expect ०.१ and was the last thing still
        // written to the old "everything gets rounded" rule.
        assert_eq!(d("1").div(&d("8"), 1).unwrap().to_plain_string(), "0.125");
        assert_eq!(d("1").div(&d("2"), 1).unwrap().to_plain_string(), "0.5");
        // A repeating quotient is what `prec` actually cuts, rounding on the
        // digits it drops.
        assert_eq!(d("1").div(&d("3"), 1).unwrap().to_plain_string(), "0.3");
        assert_eq!(d("2").div(&d("3"), 1).unwrap().to_plain_string(), "0.7");
        assert_eq!(d("1").div(&d("3"), 2).unwrap().to_plain_string(), "0.33");
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
