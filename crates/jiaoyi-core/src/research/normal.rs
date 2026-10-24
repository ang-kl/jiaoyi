//! The standard normal distribution: Φ from the complementary error function
//! (libm), and Φ⁻¹ by P. J. Acklam's rational approximation refined by one
//! Halley step against that Φ (relative error near machine precision).

const SQRT_2: f64 = std::f64::consts::SQRT_2;

pub fn cdf(x: f64) -> f64 {
    0.5 * libm::erfc(-x / SQRT_2)
}

pub fn inv_cdf(p: f64) -> f64 {
    assert!(p > 0.0 && p < 1.0, "inv_cdf({p})");
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383_577_518_672_69e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    let p_low = 0.02425;
    let x = if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - p_low {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    // One Halley step.
    let e = cdf(x) - p;
    let u = e * (2.0 * std::f64::consts::PI).sqrt() * (x * x / 2.0).exp();
    x - u / (1.0 + x * u / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        // Reference values from Python's statistics.NormalDist (independent).
        assert!((cdf(0.0) - 0.5).abs() < 1e-16);
        assert!((cdf(1.959963984540054) - 0.975).abs() < 1e-14);
        assert!((cdf(-1.0) - 0.15865525393145707).abs() < 1e-15);
        assert!((inv_cdf(0.975) - 1.959963984540054).abs() < 1e-12);
        assert!((inv_cdf(0.01) - -2.3263478740408408).abs() < 1e-12);
        assert!((inv_cdf(0.999) - 3.090232306167813).abs() < 1e-11);
        assert!((inv_cdf(0.5)).abs() < 1e-15);
    }
}
