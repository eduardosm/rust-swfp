use crate::simple_fp::Sfp;
use crate::traits::{FloatImpExt, SInt, UInt};

mod reduce_pi_2_large;
pub(crate) mod sfpm128e16;
pub(crate) mod sfpm192e16;
pub(crate) mod sfpm32e16;
pub(crate) mod sfpm64e16;

pub(crate) use reduce_pi_2_large::reduce_pi_2_large;

pub(crate) fn reduce_half_revs<F: FloatImpExt>(x: F) -> (u8, F) {
    let x_exp = x.exponent();
    if x_exp < F::Exp::from(-2i8) {
        // |x| < 0.25
        return (0, x);
    }

    let x_abs = x.abs();
    let x_scale = x_abs.scalbn(1);
    let x_int = x_scale.round_ties_even();

    if let Some(xi) = x_int.to_uint(F::MANT_BITS + 2) {
        let x_frac = x_scale - x_int;

        let n = xi as u8;
        let y = x_frac.scalbn(-1);

        if x.sign() {
            (n.wrapping_neg() & 3, -y)
        } else {
            (n & 3, y)
        }
    } else {
        (0, F::ZERO)
    }
}

pub(crate) fn reduce_half_revs_sfp<M: UInt, E: SInt>(x: Sfp<M, E>) -> (u8, Sfp<M, E>) {
    let x_exp = x.exponent();
    if x_exp < E::from(-2i8) {
        // |x| < 0.25
        return (0, x);
    }

    let x_abs = x.abs();
    let x_scale = x_abs.scalbn(E::ONE);

    if let Some(xi) = x_scale.to_int_round::<M>() {
        let x_int = Sfp::from_int(xi);
        let x_frac = x_scale - x_int;

        let n: u8 = xi.cast_into();
        let y = x_frac.scalbn(-E::ONE);

        if x.sign() {
            (n.wrapping_neg() & 3, -y)
        } else {
            (n & 3, y)
        }
    } else {
        (0, Sfp::ZERO)
    }
}
