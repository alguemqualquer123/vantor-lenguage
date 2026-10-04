// Portable SIMD abstraction (Spec §33).
//
// API: explicit `f32x4` vector type with a scalar fallback plus x86-64
// SSE2 acceleration chosen at runtime. ARM NEON and AVX-512/AVX2 hooks
// follow the same dispatch shape. `simd_width()` reports the active lane
// count so binaries select kernels without executing unsupported
// instructions.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct F32x4 {
    pub v: [f32; 4],
}

impl F32x4 {
    pub fn splat(x: f32) -> Self {
        F32x4 { v: [x; 4] }
    }

    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        F32x4 { v: [a, b, c, d] }
    }

    pub fn add(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("sse2") {
                return sse2_add(self, rhs);
            }
        }
        let [a0, a1, a2, a3] = self.v;
        let [b0, b1, b2, b3] = rhs.v;
        F32x4 { v: [a0 + b0, a1 + b1, a2 + b2, a3 + b3] }
    }

    pub fn mul(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("sse2") {
                return sse2_mul(self, rhs);
            }
        }
        let [a0, a1, a2, a3] = self.v;
        let [b0, b1, b2, b3] = rhs.v;
        F32x4 { v: [a0 * b0, a1 * b1, a2 * b2, a3 * b3] }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        let p = self.mul(rhs);
        p.v[0] + p.v[1] + p.v[2] + p.v[3]
    }

    pub fn sum(self) -> f32 {
        self.v[0] + self.v[1] + self.v[2] + self.v[3]
    }
}

#[cfg(target_arch = "x86_64")]
fn sse2_add(a: F32x4, b: F32x4) -> F32x4 {
    use std::arch::x86_64::*;
    unsafe {
        let va = _mm_loadu_ps(a.v.as_ptr());
        let vb = _mm_loadu_ps(b.v.as_ptr());
        let mut out = [0f32; 4];
        _mm_storeu_ps(out.as_mut_ptr(), _mm_add_ps(va, vb));
        F32x4 { v: out }
    }
}

#[cfg(target_arch = "x86_64")]
fn sse2_mul(a: F32x4, b: F32x4) -> F32x4 {
    use std::arch::x86_64::*;
    unsafe {
        let va = _mm_loadu_ps(a.v.as_ptr());
        let vb = _mm_loadu_ps(b.v.as_ptr());
        let mut out = [0f32; 4];
        _mm_storeu_ps(out.as_mut_ptr(), _mm_mul_ps(va, vb));
        F32x4 { v: out }
    }
}

/// Part X native vector alias: 4-lane f32 (Spec §33 + §60).
pub type Vec4 = F32x4;

/// Part X native 8-lane f32 vector (Spec §33 + §60).
///
/// Portable implementation: arithmetic dispatches through two [`F32x4`]
/// halves (which themselves use SSE2 on x86-64 when available) with a
/// scalar tail-free loop. No unsupported instructions are ever
/// executed: wider ISAs (AVX/AVX2/AVX-512/NEON) are selected only via
/// [`supports_feature`] / [`preferred_width`] queries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec8 {
    pub v: [f32; 8],
}

impl Vec8 {
    pub fn splat(x: f32) -> Self {
        Vec8 { v: [x; 8] }
    }

    pub fn new(a: [f32; 8]) -> Self {
        Vec8 { v: a }
    }

    pub fn from_halves(lo: F32x4, hi: F32x4) -> Self {
        let mut v = [0.0f32; 8];
        v[..4].copy_from_slice(&lo.v);
        v[4..].copy_from_slice(&hi.v);
        Vec8 { v }
    }

    pub fn halves(self) -> (F32x4, F32x4) {
        (
            F32x4::new(self.v[0], self.v[1], self.v[2], self.v[3]),
            F32x4::new(self.v[4], self.v[5], self.v[6], self.v[7]),
        )
    }

    pub fn add(self, rhs: Self) -> Self {
        let (a_lo, a_hi) = self.halves();
        let (b_lo, b_hi) = rhs.halves();
        Self::from_halves(a_lo.add(b_lo), a_hi.add(b_hi))
    }

    pub fn mul(self, rhs: Self) -> Self {
        let (a_lo, a_hi) = self.halves();
        let (b_lo, b_hi) = rhs.halves();
        Self::from_halves(a_lo.mul(b_lo), a_hi.mul(b_hi))
    }

    pub fn dot(self, rhs: Self) -> f32 {
        let p = self.mul(rhs);
        p.sum()
    }

    pub fn sum(self) -> f32 {
        self.v.iter().sum()
    }

    pub fn to_array(self) -> [f32; 8] {
        self.v
    }
}

/// Active SIMD lane width for f32 on this host.
pub fn simd_width() -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx") {
            return 8;
        }
        if std::arch::is_x86_feature_detected!("sse2") {
            return 4;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        return 4; // NEON mandatory
    }
    1
}

/// Feature detection for portable SIMD dispatch (Spec §33 + §68).
///
/// Returns enabled feature names for this host (`sse2`, `avx`, `avx2`,
/// `avx512f` on x86-64; `neon` on aarch64). Binaries select kernels via
/// [`supports_feature`] without executing unsupported instructions.
pub fn simd_features() -> Vec<&'static str> {
    let mut out = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("sse2") {
            out.push("sse2");
        }
        if std::arch::is_x86_feature_detected!("avx") {
            out.push("avx");
        }
        if std::arch::is_x86_feature_detected!("avx2") {
            out.push("avx2");
        }
        if std::arch::is_x86_feature_detected!("avx512f") {
            out.push("avx512f");
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        out.push("neon");
    }
    out
}

/// Whether a named SIMD feature is enabled on this host.
pub fn supports_feature(name: &str) -> bool {
    simd_features().iter().any(|f| *f == name)
}

/// Preferred native lane count: 8 when AVX is present, else 4 when a
/// 128-bit vector ISA is present, else 1 (scalar fallback).
pub fn preferred_width() -> usize {
    if supports_feature("avx") || supports_feature("avx2") || supports_feature("avx512f") {
        8
    } else if supports_feature("sse2") || supports_feature("neon") {
        4
    } else {
        simd_width().max(1)
    }
}

/// Dot product over slices using 8-wide chunks when AVX is available,
/// else 4-wide chunks, with a scalar tail.
pub fn dot_product_wide(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    if supports_feature("avx") || supports_feature("avx2") {
        let mut total = 0.0f32;
        let mut i = 0;
        while i + 8 <= a.len() {
            let va = Vec8::new([
                a[i], a[i + 1], a[i + 2], a[i + 3], a[i + 4], a[i + 5], a[i + 6], a[i + 7],
            ]);
            let vb = Vec8::new([
                b[i], b[i + 1], b[i + 2], b[i + 3], b[i + 4], b[i + 5], b[i + 6], b[i + 7],
            ]);
            total += va.dot(vb);
            i += 8;
        }
        while i < a.len() {
            total += a[i] * b[i];
            i += 1;
        }
        Some(total)
    } else {
        dot_product(a, b)
    }
}

/// Dot product over slices using 4-wide chunks with scalar tail.
pub fn dot_product(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    let mut acc = F32x4::splat(0.0);
    let mut i = 0;
    while i + 4 <= a.len() {
        let va = F32x4::new(a[i], a[i + 1], a[i + 2], a[i + 3]);
        let vb = F32x4::new(b[i], b[i + 1], b[i + 2], b[i + 3]);
        acc = acc.add(va.mul(vb));
        i += 4;
    }
    let mut total = acc.sum();
    while i < a.len() {
        total += a[i] * b[i];
        i += 1;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lanes_math() {
        let a = F32x4::new(1.0, 2.0, 3.0, 4.0);
        let b = F32x4::splat(2.0);
        assert_eq!(a.add(b), F32x4::new(3.0, 4.0, 5.0, 6.0));
        assert_eq!(a.mul(b), F32x4::new(2.0, 4.0, 6.0, 8.0));
        assert_eq!(a.dot(b), 20.0);
    }

    #[test]
    fn dot_with_tail() {
        let a: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let b = vec![1.0f32; 10];
        assert_eq!(dot_product(&a, &b), Some(45.0));
        assert_eq!(dot_product(&a, &b[..5]), None);
    }

    #[test]
    fn width_sane() {
        assert!(simd_width() >= 1);
    }

    #[test]
    fn vec8_math_matches_scalar() {
        let a = Vec8::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let b = Vec8::splat(2.0);
        assert_eq!(a.add(b).to_array(), [3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        assert_eq!(a.mul(b).to_array(), [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);
        assert_eq!(a.dot(b), 72.0);
        let alias: Vec4 = F32x4::splat(1.0);
        assert_eq!(alias.sum(), 4.0);
    }

    #[test]
    fn feature_detection_consistent() {
        let feats = simd_features();
        #[cfg(target_arch = "x86_64")]
        assert!(feats.contains(&"sse2"));
        #[cfg(target_arch = "aarch64")]
        assert!(feats.contains(&"neon"));
        assert!(preferred_width() >= 1);
        assert_eq!(supports_feature("definitely-not-a-feature"), false);
        let a: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let b = vec![1.0f32; 16];
        assert_eq!(dot_product_wide(&a, &b), Some(120.0));
        assert_eq!(dot_product_wide(&a, &b[..4]), None);
    }
}
