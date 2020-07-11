//! Singular-value decomposition

use crate::{error::*, layout::MatrixLayout};
use cauchy::*;
use num_traits::{ToPrimitive, Zero};

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum FlagSVD {
    All = b'A',
    // OverWrite = b'O',
    // Separately = b'S',
    No = b'N',
}

impl FlagSVD {
    fn from_bool(calc_uv: bool) -> Self {
        if calc_uv {
            FlagSVD::All
        } else {
            FlagSVD::No
        }
    }
}

/// Result of SVD
pub struct SVDOutput<A: Scalar> {
    /// diagonal values
    pub s: Vec<A::Real>,
    /// Unitary matrix for destination space
    pub u: Option<Vec<A>>,
    /// Unitary matrix for departure space
    pub vt: Option<Vec<A>>,
}

/// Wraps `*gesvd`
pub trait SVD_: Scalar {
    /// Calculate singular value decomposition $ A = U \Sigma V^T $
    unsafe fn svd(
        l: MatrixLayout,
        calc_u: bool,
        calc_vt: bool,
        a: &mut [Self],
    ) -> Result<SVDOutput<Self>>;
}

macro_rules! impl_svd_real {
    ($scalar:ty, $gesvd:path) => {
        impl SVD_ for $scalar {
            unsafe fn svd(
                l: MatrixLayout,
                calc_u: bool,
                calc_vt: bool,
                mut a: &mut [Self],
            ) -> Result<SVDOutput<Self>> {
                let ju = match l {
                    MatrixLayout::F { .. } => FlagSVD::from_bool(calc_u),
                    MatrixLayout::C { .. } => FlagSVD::from_bool(calc_vt),
                };
                let jvt = match l {
                    MatrixLayout::F { .. } => FlagSVD::from_bool(calc_vt),
                    MatrixLayout::C { .. } => FlagSVD::from_bool(calc_u),
                };

                let m = l.lda();
                let mut u = match ju {
                    FlagSVD::All => Some(vec![Self::zero(); (m * m) as usize]),
                    FlagSVD::No => None,
                };

                let n = l.len();
                let mut vt = match jvt {
                    FlagSVD::All => Some(vec![Self::zero(); (n * n) as usize]),
                    FlagSVD::No => None,
                };

                let k = std::cmp::min(m, n);
                let mut s = vec![Self::Real::zero(); k as usize];

                // eval work size
                let mut info = 0;
                let mut work_size = [Self::zero()];
                $gesvd(
                    ju as u8,
                    jvt as u8,
                    m,
                    n,
                    &mut a,
                    m,
                    &mut s,
                    u.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    m,
                    vt.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    n,
                    &mut work_size,
                    -1,
                    &mut info,
                );
                info.as_lapack_result()?;

                // calc
                let lwork = work_size[0].to_usize().unwrap();
                let mut work = vec![Self::zero(); lwork];
                $gesvd(
                    ju as u8,
                    jvt as u8,
                    m,
                    n,
                    &mut a,
                    m,
                    &mut s,
                    u.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    m,
                    vt.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    n,
                    &mut work,
                    lwork as i32,
                    &mut info,
                );
                info.as_lapack_result()?;
                match l {
                    MatrixLayout::F { .. } => Ok(SVDOutput { s, u, vt }),
                    MatrixLayout::C { .. } => Ok(SVDOutput { s, u: vt, vt: u }),
                }
            }
        }
    };
} // impl_svd_real!

impl_svd_real!(f64, lapack::dgesvd);
impl_svd_real!(f32, lapack::sgesvd);

macro_rules! impl_svd_complex {
    ($scalar:ty, $gesvd:path) => {
        impl SVD_ for $scalar {
            unsafe fn svd(
                l: MatrixLayout,
                calc_u: bool,
                calc_vt: bool,
                mut a: &mut [Self],
            ) -> Result<SVDOutput<Self>> {
                let ju = match l {
                    MatrixLayout::F { .. } => FlagSVD::from_bool(calc_u),
                    MatrixLayout::C { .. } => FlagSVD::from_bool(calc_vt),
                };
                let jvt = match l {
                    MatrixLayout::F { .. } => FlagSVD::from_bool(calc_vt),
                    MatrixLayout::C { .. } => FlagSVD::from_bool(calc_u),
                };

                let m = l.lda();
                let mut u = match ju {
                    FlagSVD::All => Some(vec![Self::zero(); (m * m) as usize]),
                    FlagSVD::No => None,
                };

                let n = l.len();
                let mut vt = match jvt {
                    FlagSVD::All => Some(vec![Self::zero(); (n * n) as usize]),
                    FlagSVD::No => None,
                };

                let k = std::cmp::min(m, n);
                let mut s = vec![Self::Real::zero(); k as usize];

                let mut rwork = vec![Self::Real::zero(); 5 * k as usize];

                // eval work size
                let mut info = 0;
                let mut work_size = [Self::zero()];
                $gesvd(
                    ju as u8,
                    jvt as u8,
                    m,
                    n,
                    &mut a,
                    m,
                    &mut s,
                    u.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    m,
                    vt.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    n,
                    &mut work_size,
                    -1,
                    &mut rwork,
                    &mut info,
                );
                info.as_lapack_result()?;

                // calc
                let lwork = work_size[0].to_usize().unwrap();
                let mut work = vec![Self::zero(); lwork];
                $gesvd(
                    ju as u8,
                    jvt as u8,
                    m,
                    n,
                    &mut a,
                    m,
                    &mut s,
                    u.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    m,
                    vt.as_mut().map(|x| x.as_mut_slice()).unwrap_or(&mut []),
                    n,
                    &mut work,
                    lwork as i32,
                    &mut rwork,
                    &mut info,
                );
                info.as_lapack_result()?;
                match l {
                    MatrixLayout::F { .. } => Ok(SVDOutput { s, u, vt }),
                    MatrixLayout::C { .. } => Ok(SVDOutput { s, u: vt, vt: u }),
                }
            }
        }
    };
} // impl_svd_real!

impl_svd_complex!(c64, lapack::zgesvd);
impl_svd_complex!(c32, lapack::cgesvd);
