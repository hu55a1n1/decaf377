// use super::{fiat, wrapper::Fp};
use super::super::u64::{fiat, wrapper::Fp};
use ark_ff::FftField;
use ark_ff::{BigInt, Field, PrimeField, SqrtPrecomputation};
use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, Compress, Flags, SerializationError, Valid, Validate,
};
use ark_std::{rand, str::FromStr, string::ToString, One, Zero};
use core::hash::Hash;
use core::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    iter,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use crate::fields::constants::{FP_MODULUS, FP_MODULUS_MINUS_ONE_DIV_TWO, FP_MODULUS_BIT_SIZE, FP_TRACE, 
    FP_TRACE_MINUS_ONE_DIV_TWO, FP_QUADRATIC_NONRESIDUE_TO_TRACE, FP_ZERO, FP_ONE, FP_TWO_ADIC_ROOT_OF_UNITY, 
    FP_TWO_ADICITY, FP_GENERATOR};

impl PrimeField for Fp {
    /// A `BigInteger` type that can represent elements of this field.
    type BigInt = BigInt<6>;

    /// The BLS12-377 base field modulus `p` (258664426012969094010652733694893533536393512754914660539884262666720468348340822774968888139573360124440321458177)
    const MODULUS: Self::BigInt = ark_ff::BigInt(FP_MODULUS);

    /// The value `(p - 1)/ 2`.
    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BigInt(FP_MODULUS_MINUS_ONE_DIV_TWO);

    /// The size of the modulus in bits.
    const MODULUS_BIT_SIZE: u32 = FP_MODULUS_BIT_SIZE;

    /// The trace of the field is defined as the smallest integer `t` such that by
    /// `2^s * t = p - 1`, and `t` is coprime to 2.
    const TRACE: Self::BigInt = BigInt(FP_TRACE);

    /// The value `(t - 1)/ 2`.
    const TRACE_MINUS_ONE_DIV_TWO: Self::BigInt = BigInt(FP_TRACE_MINUS_ONE_DIV_TWO);

    fn from_bigint(repr: Self::BigInt) -> Option<Self> {
        if repr >= Fp::MODULUS {
            None
        } else {
            Some(Fp(fiat::FpMontgomeryDomainFieldElement(repr.0)))
        }
    }

    fn into_bigint(self) -> Self::BigInt {
        BigInt(self.0 .0)
    }

    fn from_be_bytes_mod_order(bytes: &[u8]) -> Self {
        let mut bytes_copy = bytes.to_vec();
        bytes_copy.reverse();
        Self::from_le_bytes_mod_order(&bytes_copy)
    }

    fn from_le_bytes_mod_order(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 48);

        let mut t = [0u8; 48];
        t.copy_from_slice(&bytes[..48]);
        let modulus_field_montgomery = Fp::from_bytes(&t);

        modulus_field_montgomery
    }
}

impl Field for Fp {
    type BasePrimeField = Self;
    type BasePrimeFieldIter = iter::Once<Self::BasePrimeField>;

    const SQRT_PRECOMP: Option<SqrtPrecomputation<Self>> =
        Some(SqrtPrecomputation::TonelliShanks {
            two_adicity: FP_TWO_ADICITY,
            quadratic_nonresidue_to_trace: Fp(fiat::FpMontgomeryDomainFieldElement(FP_QUADRATIC_NONRESIDUE_TO_TRACE)),
            trace_of_modulus_minus_one_div_two: &FP_TRACE_MINUS_ONE_DIV_TWO,
        });

    const ZERO: Self = Fp(fiat::FpMontgomeryDomainFieldElement(FP_ZERO));

    // Montomgery representation of one
    const ONE: Self = Fp(fiat::FpMontgomeryDomainFieldElement(FP_ONE));

    fn extension_degree() -> u64 {
        1
    }

    fn to_base_prime_field_elements(&self) -> Self::BasePrimeFieldIter {
        iter::once(*self)
    }

    fn from_base_prime_field_elems(elems: &[Self::BasePrimeField]) -> Option<Self> {
        if elems.len() != (Self::extension_degree() as usize) {
            return None;
        }
        Some(elems[0])
    }

    fn from_base_prime_field(elem: Self::BasePrimeField) -> Self {
        elem
    }

    fn double(&self) -> Self {
        self.add(*self)
    }

    fn double_in_place(&mut self) -> &mut Self {
        *self = self.add(*self);
        self
    }

    fn neg_in_place(&mut self) -> &mut Self {
        *self = self.add(*self);
        self
    }

    fn from_random_bytes_with_flags<F: ark_serialize::Flags>(bytes: &[u8]) -> Option<(Self, F)> {
        Some((Self::from_le_bytes_mod_order(bytes), F::default()))
    }

    fn legendre(&self) -> ark_ff::LegendreSymbol {
        unimplemented!()
    }

    fn square(&self) -> Self {
        self.square()
    }

    fn square_in_place(&mut self) -> &mut Self {
        *self = self.square();
        self
    }

    fn inverse(&self) -> Option<Self> {
        self.inverse()
    }

    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        if let Some(inverse) = self.inverse() {
            *self = inverse;
            Some(self)
        } else {
            None
        }
    }

    fn frobenius_map_in_place(&mut self, _power: usize) {
        // Because this is a prime field, we don't need to do anything,
        // the automorphism is trivial.
    }
}

impl FftField for Fp {
    const GENERATOR: Self = Fp(fiat::FpMontgomeryDomainFieldElement(FP_GENERATOR));
    const TWO_ADICITY: u32 = FP_TWO_ADICITY;
    const TWO_ADIC_ROOT_OF_UNITY: Self = Fp(fiat::FpMontgomeryDomainFieldElement(FP_TWO_ADIC_ROOT_OF_UNITY));
    const SMALL_SUBGROUP_BASE: Option<u32> = None;
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = None;
}

impl From<u128> for Fp {
    fn from(other: u128) -> Self {
        let mut result = BigInt::default();
        result.0[0] = ((other << 64) >> 64) as u64;
        result.0[1] = (other >> 64) as u64;
        Self::from_bigint(result).unwrap()
    }
}

impl From<u64> for Fp {
    fn from(other: u64) -> Self {
        Self::from_bigint(BigInt::from(other)).unwrap()
    }
}

impl From<u32> for Fp {
    fn from(other: u32) -> Self {
        Self::from_bigint(BigInt::from(other)).unwrap()
    }
}

impl From<u16> for Fp {
    fn from(other: u16) -> Self {
        Self::from_bigint(BigInt::from(other)).unwrap()
    }
}

impl From<u8> for Fp {
    fn from(other: u8) -> Self {
        Self::from_bigint(BigInt::from(other)).unwrap()
    }
}

impl From<bool> for Fp {
    fn from(other: bool) -> Self {
        Self::from_bigint(BigInt::from(u64::from(other))).unwrap()
    }
}

impl Neg for Fp {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(self) -> Self {
        let neg = self.neg();
        neg
    }
}

impl<'a> AddAssign<&'a Self> for Fp {
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        *self = self.add(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::AddAssign<Self> for Fp {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = self.add(other);
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::AddAssign<&'a mut Self> for Fp {
    #[inline(always)]
    fn add_assign(&mut self, other: &'a mut Self) {
        *self = self.add(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::Add<Self> for Fp {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl<'a> Add<&'a Fp> for Fp {
    type Output = Self;

    #[inline]
    fn add(self, other: &Self) -> Self {
        self.add(*other)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::Add<&'a mut Self> for Fp {
    type Output = Self;

    #[inline]
    fn add(self, other: &'a mut Self) -> Self {
        self.add(*other)
    }
}

impl<'a> SubAssign<&'a Self> for Fp {
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        *self = self.sub(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::SubAssign<Self> for Fp {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = self.sub(other);
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::SubAssign<&'a mut Self> for Fp {
    #[inline(always)]
    fn sub_assign(&mut self, other: &'a mut Self) {
        *self = self.sub(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::Sub<Self> for Fp {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl<'a> Sub<&'a Fp> for Fp {
    type Output = Self;

    #[inline]
    fn sub(self, other: &Self) -> Self {
        self.sub(*other)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::Sub<&'a mut Self> for Fp {
    type Output = Self;

    #[inline]
    fn sub(self, other: &'a mut Self) -> Self {
        self.sub(*other)
    }
}

impl<'a> MulAssign<&'a Self> for Fp {
    fn mul_assign(&mut self, other: &Self) {
        *self = self.mul(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::MulAssign<Self> for Fp {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = self.mul(other);
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::MulAssign<&'a mut Self> for Fp {
    #[inline(always)]
    fn mul_assign(&mut self, other: &'a mut Self) {
        *self = self.mul(*other);
    }
}

#[allow(unused_qualifications)]
impl core::ops::Mul<Self> for Fp {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl<'a> Mul<&'a Fp> for Fp {
    type Output = Self;

    #[inline]
    fn mul(self, other: &Self) -> Self {
        self.mul(*other)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::Mul<&'a mut Self> for Fp {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: &'a mut Self) -> Self {
        self.mul(*other)
    }
}

impl<'a> DivAssign<&'a Self> for Fp {
    #[inline(always)]
    fn div_assign(&mut self, other: &Self) {
        self.mul_assign(&other.inverse().unwrap());
    }
}

#[allow(unused_qualifications)]
impl core::ops::DivAssign<Self> for Fp {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::DivAssign<&'a mut Self> for Fp {
    #[inline(always)]
    fn div_assign(&mut self, other: &'a mut Self) {
        self.div_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl core::ops::Div<Self> for Fp {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: Self) -> Self {
        self.div_assign(&other);
        self
    }
}

impl<'a> Div<&'a Fp> for Fp {
    type Output = Self;

    #[inline]
    fn div(mut self, other: &Self) -> Self {
        self.mul_assign(&other.inverse().unwrap());
        self
    }
}

#[allow(unused_qualifications)]
impl<'a> core::ops::Div<&'a mut Self> for Fp {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: &'a mut Self) -> Self {
        self.div_assign(&*other);
        self
    }
}

impl Zero for Fp {
    #[inline]
    fn zero() -> Self {
        Fp::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == Fp::ZERO
    }
}

#[allow(unused_qualifications)]
impl core::iter::Sum<Self> for Fp {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::iter::Sum<&'a Self> for Fp {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

impl One for Fp {
    #[inline]
    fn one() -> Self {
        Fp::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == Fp::ONE
    }
}

#[allow(unused_qualifications)]
impl core::iter::Product<Self> for Fp {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a> core::iter::Product<&'a Self> for Fp {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

impl CanonicalDeserializeWithFlags for Fp {
    fn deserialize_with_flags<R: ark_std::io::Read, F: Flags>(
        _reader: R,
    ) -> Result<(Self, F), SerializationError> {
        unimplemented!()
    }
}

impl Valid for Fp {
    fn check(&self) -> Result<(), SerializationError> {
        unimplemented!()
    }
}

impl CanonicalDeserialize for Fp {
    fn deserialize_with_mode<R: ark_std::io::Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        unimplemented!()
    }
}

impl CanonicalSerialize for Fp {
    #[inline]
    fn serialize_with_mode<W: ark_std::io::Write>(
        &self,
        _writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        unimplemented!()
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        unimplemented!()
    }
}

impl CanonicalSerializeWithFlags for Fp {
    fn serialize_with_flags<W: ark_std::io::Write, F: Flags>(
        &self,
        _writer: W,
        _flags: F,
    ) -> Result<(), SerializationError> {
        unimplemented!()
    }

    fn serialized_size_with_flags<F: Flags>(&self) -> usize {
        unimplemented!()
    }
}

impl ark_std::rand::distributions::Distribution<Fp> for ark_std::rand::distributions::Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, _rng: &mut R) -> Fp {
        todo!()
    }
}

impl Display for Fp {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let string = self.into_bigint().to_string();
        write!(f, "{}", string.trim_start_matches('0'))
    }
}

impl zeroize::Zeroize for Fp {
    fn zeroize(&mut self) {
        unimplemented!()
    }
}

impl Ord for Fp {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bigint().cmp(&other.into_bigint())
    }
}

impl PartialOrd for Fp {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Fp {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

impl From<num_bigint::BigUint> for Fp {
    #[inline]
    fn from(val: num_bigint::BigUint) -> Fp {
        Fp::from_le_bytes_mod_order(&val.to_bytes_le())
    }
}

impl From<Fp> for num_bigint::BigUint {
    #[inline(always)]
    fn from(other: Fp) -> Self {
        other.into_bigint().into()
    }
}

impl From<Fp> for BigInt<6> {
    #[inline(always)]
    fn from(fp: Fp) -> Self {
        fp.into_bigint()
    }
}

impl From<BigInt<6>> for Fp {
    /// Converts `Self::BigInteger` into `Self`
    #[inline(always)]
    fn from(int: BigInt<6>) -> Self {
        Fp(fiat::FpMontgomeryDomainFieldElement(int.0))
    }
}

impl Hash for Fp {
    fn hash<H: core::hash::Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

impl PartialEq for Fp {
    fn eq(&self, other: &Fp) -> bool {
        let self_bytes = self.to_bytes();
        let other_bytes = other.to_bytes();
        self_bytes[..] == other_bytes[..]
    }
}

impl Eq for Fp {}

impl Default for Fp {
    fn default() -> Self {
        Fp(fiat::FpMontgomeryDomainFieldElement([0, 0, 0, 0, 0, 0]))
    }
}

impl core::fmt::Debug for Fp {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let bytes = self.to_bytes();
        write!(f, "Fp(0x{})", hex::encode(bytes))
    }
}

#[cfg(test)]
#[cfg(feature = "u64")]
mod tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_bls377()(
            z0 in 0..u64::MAX,
            z1 in 0..u64::MAX,
            z2 in 0..u64::MAX,
            z3 in 0..u64::MAX,
            z4 in 0..u64::MAX,
            z5 in 0..(1u64 << 57)
        ) -> [u64; 6] {
            [z0, z1, z2, z3, z4, z5]
        }
    }

    prop_compose! {
        fn arb_fp()(a in arb_bls377()) -> Fp {
            let a = fiat::FpNonMontgomeryDomainFieldElement(a);

            let mut a_to_mont = fiat::FpMontgomeryDomainFieldElement([0; 6]);
            fiat::fp_to_montgomery(&mut a_to_mont, &a);

            Fp(a_to_mont)
        }
    }

    proptest! {
        #[test]
        fn test_addition_commutative(a in arb_fp(), b in arb_fp()) {
            assert_eq!(a + b, b + a);
        }
    }

    proptest! {
        #[test]
        fn test_addition_associative(a in arb_fp(), b in arb_fp(), c in arb_fp()) {
            assert_eq!(a + (b + c), (a + b) + c);
        }
    }

    proptest! {
        #[test]
        fn test_add_zero_identity(a in arb_fp()) {
            let zero = Fp::zero();

            assert_eq!(a + zero, a);
            assert_eq!(zero + a, a);
        }
    }

    proptest! {
        #[test]
        fn test_subtract_self_is_zero(a in arb_fp()) {
            let zero = Fp::zero();

            assert_eq!(a - a, zero);
        }
    }

    proptest! {
        #[test]
        fn test_doubling_is_just_addition(a in arb_fp()) {
            let two = fiat::FpNonMontgomeryDomainFieldElement([2, 0, 0, 0, 0, 0]);

            let mut two_to_mont = fiat::FpMontgomeryDomainFieldElement([0; 6]);
            fiat::fp_to_montgomery(&mut two_to_mont, &two);

            assert_eq!(Fp(two_to_mont) * a, a + a);
            assert_eq!(a.double(), a + a);
            assert_eq!(*(a.clone().double_in_place()), a + a);
        }
    }

    proptest! {
        #[test]
        fn test_adding_negation(a in arb_bls377()) {
            let a = Fp(fiat::FpMontgomeryDomainFieldElement(a));

            assert_eq!(a + -a, Fp::ZERO)
        }
    }

    proptest! {
        #[test]
        fn test_multiplication_commutative(a in arb_fp(), b in arb_fp()) {
            assert_eq!(a * b, b * a);
        }
    }

    proptest! {
        #[test]
        fn test_multiplication_associative(a in arb_fp(), b in arb_fp(), c in arb_fp()) {
            assert_eq!(a * (b * c), (a * b) * c);
        }
    }

    proptest! {
        #[test]
        fn test_multiplication_distributive(a in arb_fp(), b in arb_fp(), c in arb_fp()) {
            assert_eq!(a * (b + c), a * b + a * c);
        }
    }

    proptest! {
        #[test]
        fn test_multiply_one_identity(a in arb_fp()) {
            assert_eq!(a * Fp::ONE, a);
            assert_eq!(Fp::ONE * a, a);
        }
    }

    proptest! {
        #[test]
        fn test_multiply_minus_one_is_negation(a in arb_fp()) {
            let minus_one = -Fp::ONE;

            assert_eq!(a * minus_one, a.neg());
        }
    }

    proptest! {
        #[test]
        fn test_square_is_multiply(a in arb_fp()) {
            assert_eq!(a.square(), a * a);
            assert_eq!(*(a.clone().square_in_place()), a * a);
        }
    }

    proptest! {
        #[test]
        fn test_inverse(a in arb_fp()) {
            assert_eq!(a * a.inverse().unwrap(), Fp::ONE);
            assert_eq!(a * *(a.clone().inverse_in_place().unwrap()), Fp::ONE);
        }
    }

    proptest! {
        #[test]
        fn test_sqrt(a in arb_fp()) {
            match a.sqrt() {
                Some(x) => assert_eq!(x * x, a),
                None => {}
            }
        }
    }

    #[test]
    fn test_addition_examples() {
        let z1 = Fp(fiat::FpMontgomeryDomainFieldElement([1, 1, 1, 1, 1, 1]));
        let z2 = Fp(fiat::FpMontgomeryDomainFieldElement([2, 2, 2, 2, 2, 2]));
        let z3 = Fp(fiat::FpMontgomeryDomainFieldElement([3, 3, 3, 3, 3, 3]));

        assert_eq!(z3, z1 + z2);
    }

    #[test]
    fn test_subtraction_examples() {
        let mut z1 = Fp(fiat::FpMontgomeryDomainFieldElement([1, 1, 1, 1, 1, 1]));
        z1 -= z1;

        assert_eq!(z1, Fp::ZERO);
    }

    #[test]
    fn test_small_multiplication_examples() {
        let z1 = fiat::FpNonMontgomeryDomainFieldElement([1, 0, 0, 0, 0, 0]);
        let z2 = fiat::FpNonMontgomeryDomainFieldElement([2, 0, 0, 0, 0, 0]);
        let z3 = fiat::FpNonMontgomeryDomainFieldElement([3, 0, 0, 0, 0, 0]);

        let mut z1_mont = fiat::FpMontgomeryDomainFieldElement([0; 6]);
        let mut z2_mont = fiat::FpMontgomeryDomainFieldElement([0; 6]);
        let mut z3_mont = fiat::FpMontgomeryDomainFieldElement([0; 6]);
        fiat::fp_to_montgomery(&mut z1_mont, &z1);
        fiat::fp_to_montgomery(&mut z2_mont, &z2);
        fiat::fp_to_montgomery(&mut z3_mont, &z3);

        assert_eq!(Fp(z1_mont) + Fp(z1_mont), Fp(z1_mont) * Fp(z2_mont));
        assert_eq!(
            Fp(z1_mont) + Fp(z1_mont) + Fp(z1_mont),
            Fp(z1_mont) * Fp(z3_mont)
        );
    }

    #[test]
    fn test_2192_times_zero() {
        let two192 = Fp(fiat::FpMontgomeryDomainFieldElement([0, 0, 0, 0, 0, 1]));
        assert_eq!(two192 * Fp::zero(), Fp::ZERO);
    }

    #[test]
    fn test_minus_one_squared() {
        let minus_one = Fp::zero() - Fp::one();
        assert_eq!(minus_one.square(), Fp::ONE);
    }

    #[test]
    fn test_modulus_from_le_bytes_mod_order() {
        // Field modulus - 1 in non-montgomery form that satisfies the fiat-crypto preconditions (< m)
        let modulus_minus_one = fiat::FpNonMontgomeryDomainFieldElement([
            9586122913090633728,
            1660523435060625408,
            2230234197602682880,
            1883307231910630287,
            14284016967150029115,
            121098312706494698,
        ]);

        // Convert bytes into montgomery domain
        let mut modulus_minus_one_montgomery = fiat::FpMontgomeryDomainFieldElement([0; 6]);
        fiat::fp_to_montgomery(&mut modulus_minus_one_montgomery, &modulus_minus_one);

        // Convert to [u8; 48] bytes out of montgomery domain
        let modulus_non_montgomery_bytes = Fp::to_bytes(&Fp(modulus_minus_one_montgomery));

        // Convert [u8; 48] bytes into field element in montgomery domain
        let modulus_field_montgomery = Fp::from_le_bytes_mod_order(&modulus_non_montgomery_bytes);

        // Convert field element out of montgomery domain
        let mut x_non_montgomery = fiat::FpNonMontgomeryDomainFieldElement([0; 6]);
        fiat::fp_from_montgomery(&mut x_non_montgomery, &modulus_field_montgomery.0);

        // Assertion check against original `FpNonMontgomeryDomainFieldElement`
        assert_eq!(x_non_montgomery.0, modulus_minus_one.0);
    }
}