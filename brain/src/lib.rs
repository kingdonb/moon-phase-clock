#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use core::panic::PanicInfo;

use core::f64::consts::PI;

// Julian Date of the UNIX epoch, and of the J2000.0 epoch
const UNIX_EPOCH_JD: f64 = 2440587.5;
const J2000_JD: f64 = 2451545.0;
const SECONDS_PER_DAY: f64 = 86400.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36525.0;

const RESULT_BUFFER_LEN: usize = 128;

// Shared buffer for returning the result to the host
static mut RESULT_BUFFER: [u8; RESULT_BUFFER_LEN] = [0; RESULT_BUFFER_LEN];

/// Entry point for every host. Takes a UNIX timestamp (seconds) and returns a
/// pointer to a NUL-terminated JSON string:
///
/// `{"phase_name": "Full Moon", "torment_multiplier": 3.0, "illumination": 0.9987}`
///
/// The buffer is overwritten by the next call.
#[no_mangle]
pub extern "C" fn calculate_moon_phase(timestamp: f64) -> *const u8 {
    // SAFETY: the brain runs single-threaded and no reference to the buffer
    // outlives a call, so this is the only live reference.
    let buf = unsafe { &mut *core::ptr::addr_of_mut!(RESULT_BUFFER) };
    render(timestamp, buf);
    buf.as_ptr()
}

/// Position within the lunar cycle: 0.0 = New Moon, 0.5 = Full Moon.
///
/// This is the Moon's true elongation from the Sun (0° to 360°) as a fraction
/// of a full turn. It follows Meeus, *Astronomical Algorithms* (2nd ed.),
/// ch. 48: the mean elongation D corrected by the six largest periodic terms
/// in D and the mean anomalies of the Sun (M) and Moon (M′). Phase instants
/// come out within about half an hour of the true ones, where the mean
/// synodic month used before could be off by up to ~14 hours.
///
/// Uses UTC in place of Terrestrial Time; the ~70 s difference is far below
/// the model's accuracy.
fn cycle_position(timestamp: f64) -> f64 {
    let t = (timestamp / SECONDS_PER_DAY + UNIX_EPOCH_JD - J2000_JD) / DAYS_PER_JULIAN_CENTURY;
    let t2 = t * t;

    // Mean elongation of the Moon, mean anomaly of the Sun, mean anomaly of the Moon (degrees)
    let d = 297.8501921 + 445267.1114034 * t - 0.0018819 * t2;
    let m = 357.5291092 + 35999.0502909 * t - 0.0001536 * t2;
    let mp = 134.9633964 + 477198.8675055 * t + 0.0087414 * t2;

    // True elongation = 180° − phase angle i (Meeus eq. 48.4)
    let elongation = d
        + 6.289 * sin_deg(mp)
        - 2.100 * sin_deg(m)
        + 1.274 * sin_deg(2.0 * d - mp)
        + 0.658 * sin_deg(2.0 * d)
        + 0.214 * sin_deg(2.0 * mp)
        + 0.110 * sin_deg(d);

    normalize_degrees(elongation) / 360.0
}

/// Largest integer not above x (no_std has no f64::floor).
fn floor(x: f64) -> f64 {
    let truncated = x as i64 as f64;
    if truncated > x {
        truncated - 1.0
    } else {
        truncated
    }
}

/// Angle in degrees, reduced to [0, 360).
fn normalize_degrees(x: f64) -> f64 {
    let r = x - 360.0 * floor(x / 360.0);
    if r >= 360.0 {
        0.0
    } else {
        r
    }
}

/// sin of an angle in degrees, any magnitude.
fn sin_deg(degrees: f64) -> f64 {
    // sin(x) = cos(x − 90°)
    cos_radians(normalize_degrees(degrees - 90.0) * PI / 180.0)
}

/// cos for any x in [0, 2π), via symmetry onto [−π/2, π/2].
fn cos_radians(x: f64) -> f64 {
    // Shift to [−π, π)
    let x = if x >= PI { x - 2.0 * PI } else { x };
    let a = if x < 0.0 { -x } else { x };
    if a <= PI / 2.0 {
        cos_within_half_pi(a)
    } else {
        -cos_within_half_pi(PI - a)
    }
}

fn phase_for(cycle_position: f64) -> (&'static str, f64) {
    if cycle_position < 0.03 || cycle_position > 0.97 {
        ("New Moon", 1.0)
    } else if cycle_position < 0.22 {
        ("Waxing Crescent", 1.2)
    } else if cycle_position < 0.28 {
        ("First Quarter", 1.5)
    } else if cycle_position < 0.47 {
        ("Waxing Gibbous", 1.8)
    } else if cycle_position < 0.53 {
        ("Full Moon", 3.0)
    } else if cycle_position < 0.72 {
        ("Waning Gibbous", 1.8)
    } else if cycle_position < 0.78 {
        ("Last Quarter", 1.5)
    } else {
        ("Waning Crescent", 1.2)
    }
}

/// Illuminated fraction of the lunar disc (0.0 to 1.0).
///
/// For phase angle θ = 2π·position the lit fraction is (1 − cos θ) / 2, which
/// equals cos²(π·(position − 0.5)). That form keeps the argument inside
/// [−π/2, π/2], where the short series below is accurate.
fn illumination(cycle_position: f64) -> f64 {
    let c = cos_within_half_pi(PI * (cycle_position - 0.5));
    let k = c * c;
    if k < 0.0 {
        0.0
    } else if k > 1.0 {
        1.0
    } else {
        k
    }
}

/// cos(x) for |x| <= π/2, by Taylor series through x^18 (error below 1e-14).
/// no_std has no f64::cos, and the brain takes no dependencies.
fn cos_within_half_pi(x: f64) -> f64 {
    let x2 = x * x;
    let mut term = 1.0;
    let mut sum = 1.0;
    let mut n = 1;
    while n <= 9 {
        term *= -x2 / (((2 * n - 1) * (2 * n)) as f64);
        sum += term;
        n += 1;
    }
    sum
}

/// Writes the JSON result plus a NUL terminator into `buf`; returns the JSON
/// length (excluding the terminator).
fn render(timestamp: f64, buf: &mut [u8]) -> usize {
    render_position(cycle_position(timestamp), buf)
}

fn render_position(position: f64, buf: &mut [u8]) -> usize {
    let (phase_name, torment_multiplier) = phase_for(position);

    // Built by hand to avoid dependencies
    let mut out = Out { buf, pos: 0 };
    out.str("{\"phase_name\": \"");
    out.str(phase_name);
    out.str("\", \"torment_multiplier\": ");
    out.fixed(torment_multiplier, 1);
    out.str(", \"illumination\": ");
    out.fixed(illumination(position), 4);
    out.str("}");
    out.terminate()
}

struct Out<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl Out<'_> {
    fn push(&mut self, b: u8) {
        // Always leave room for the NUL terminator
        if self.pos + 1 < self.buf.len() {
            self.buf[self.pos] = b;
            self.pos += 1;
        }
    }

    fn str(&mut self, s: &str) {
        for b in s.as_bytes() {
            self.push(*b);
        }
    }

    /// Writes a non-negative number with exactly `decimals` (>= 1) fractional
    /// digits, rounded half up.
    fn fixed(&mut self, f: f64, decimals: u32) {
        let mut scale: u64 = 1;
        let mut i = 0;
        while i < decimals {
            scale *= 10;
            i += 1;
        }
        let scaled = (f * scale as f64 + 0.5) as u64;

        self.uint(scaled / scale);
        self.push(b'.');
        let fraction = scaled % scale;
        let mut divisor = scale / 10;
        while divisor > 0 {
            self.push(b'0' + (fraction / divisor % 10) as u8);
            divisor /= 10;
        }
    }

    fn uint(&mut self, mut n: u64) {
        let mut digits = [0u8; 20];
        let mut len = 0;
        loop {
            digits[len] = b'0' + (n % 10) as u8;
            len += 1;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        while len > 0 {
            len -= 1;
            self.push(digits[len]);
        }
    }

    fn terminate(self) -> usize {
        if !self.buf.is_empty() {
            self.buf[self.pos] = 0;
        }
        self.pos
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: f64 = 3600.0;
    const MINUTE: f64 = 60.0;

    fn json_at(timestamp: f64) -> String {
        let mut buf = [0xFFu8; RESULT_BUFFER_LEN];
        let len = render(timestamp, &mut buf);
        assert_eq!(buf[len], 0, "result must be NUL-terminated");
        String::from_utf8(buf[..len].to_vec()).unwrap()
    }

    fn json_at_position(position: f64) -> String {
        let mut buf = [0xFFu8; RESULT_BUFFER_LEN];
        let len = render_position(position, &mut buf);
        assert_eq!(buf[len], 0, "result must be NUL-terminated");
        String::from_utf8(buf[..len].to_vec()).unwrap()
    }

    fn fixed(f: f64, decimals: u32) -> String {
        let mut buf = [0u8; 32];
        let mut out = Out { buf: &mut buf, pos: 0 };
        out.fixed(f, decimals);
        let len = out.terminate();
        String::from_utf8(buf[..len].to_vec()).unwrap()
    }

    /// UNIX timestamp for a UTC date and time (proleptic Gregorian).
    fn utc(y: i64, mo: i64, d: i64, h: i64, mi: i64) -> f64 {
        // Days from civil, after Howard Hinnant
        let y = if mo <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (mo + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let days = era * 146097 + doe - 719468;
        (days * 86400 + h * 3600 + mi * 60) as f64
    }

    /// Signed distance from `position` to `target` around the cycle, in degrees of elongation.
    fn degrees_off(position: f64, target: f64) -> f64 {
        let mut diff = (position - target) * 360.0;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }
        diff
    }

    #[test]
    fn utc_helper_matches_known_timestamps() {
        assert_eq!(utc(1970, 1, 1, 0, 0), 0.0);
        assert_eq!(utc(2000, 1, 6, 18, 14), 947182440.0);
        assert_eq!(utc(2026, 11, 17, 0, 0), 1794873600.0);
    }

    #[test]
    fn cosine_matches_std_across_its_domain() {
        let half_pi = core::f64::consts::FRAC_PI_2;
        for i in 0..=2000 {
            let x = -half_pi + (i as f64 / 2000.0) * 2.0 * half_pi;
            let diff = (cos_within_half_pi(x) - x.cos()).abs();
            assert!(diff < 1e-12, "cos({x}) off by {diff}");
        }
    }

    #[test]
    fn sine_in_degrees_matches_std_for_large_and_negative_angles() {
        // The mean arguments run to ~10^5 degrees for 2026 and beyond
        for i in -2000..=2000 {
            let degrees = i as f64 * 97.31;
            let expected = (degrees.to_radians()).sin();
            let diff = (sin_deg(degrees) - expected).abs();
            assert!(diff < 1e-9, "sin({degrees}°) off by {diff}");
        }
    }

    #[test]
    fn floor_and_normalization() {
        assert_eq!(floor(2.5), 2.0);
        assert_eq!(floor(-2.5), -3.0);
        assert_eq!(floor(-3.0), -3.0);
        assert_eq!(normalize_degrees(370.0), 10.0);
        assert_eq!(normalize_degrees(-10.0), 350.0);
        assert_eq!(normalize_degrees(720.0), 0.0);
    }

    #[test]
    fn illumination_follows_the_cosine_law() {
        for i in 0..=1000 {
            let p = i as f64 / 1000.0;
            let expected = (1.0 - (2.0 * core::f64::consts::PI * p).cos()) / 2.0;
            let diff = (illumination(p) - expected).abs();
            assert!(diff < 1e-12, "illumination({p}) off by {diff}");
        }
    }

    #[test]
    fn illumination_at_the_principal_phases() {
        assert!(illumination(0.0) < 1e-12);
        assert!((illumination(0.25) - 0.5).abs() < 1e-12);
        assert!((illumination(0.5) - 1.0).abs() < 1e-12);
        assert!((illumination(0.75) - 0.5).abs() < 1e-12);
        // A mid-crescent is ~15% lit, not the 25% a linear ramp would give
        assert!((illumination(0.125) - 0.146447).abs() < 1e-6);
    }

    #[test]
    fn illumination_is_symmetric_and_bounded() {
        for i in 0..=500 {
            let p = i as f64 / 1000.0;
            let waxing = illumination(p);
            let waning = illumination(1.0 - p);
            assert!((waxing - waning).abs() < 1e-12);
            assert!((0.0..=1.0).contains(&waxing));
        }
    }

    #[test]
    fn phase_buckets_and_multipliers() {
        let cases = [
            (0.00, "New Moon", 1.0),
            (0.02, "New Moon", 1.0),
            (0.98, "New Moon", 1.0),
            (0.10, "Waxing Crescent", 1.2),
            (0.25, "First Quarter", 1.5),
            (0.40, "Waxing Gibbous", 1.8),
            (0.50, "Full Moon", 3.0),
            (0.60, "Waning Gibbous", 1.8),
            (0.75, "Last Quarter", 1.5),
            (0.90, "Waning Crescent", 1.2),
        ];
        for (position, name, multiplier) in cases {
            assert_eq!(phase_for(position), (name, multiplier), "at {position}");
        }
    }

    /// Published instants of 2026 new and full moons (UTC, to the minute),
    /// including the year's four eclipses.
    const NEW_MOONS_2026: [(i64, i64, i64, i64, i64, &str); 3] = [
        (2026, 1, 18, 19, 52, "new moon"),
        (2026, 2, 17, 12, 1, "new moon, annular solar eclipse"),
        (2026, 8, 12, 17, 37, "new moon, total solar eclipse"),
    ];
    const FULL_MOONS_2026: [(i64, i64, i64, i64, i64, &str); 4] = [
        (2026, 1, 3, 10, 3, "full moon"),
        (2026, 3, 3, 11, 38, "full moon, total lunar eclipse"),
        (2026, 8, 28, 4, 18, "full moon, partial lunar eclipse"),
        (2026, 11, 24, 14, 53, "full moon"),
    ];

    #[test]
    fn true_moon_matches_published_new_and_full_moons() {
        // The Moon moves ~0.5° of elongation per hour, so 0.5° is ~1 hour.
        let tolerance_degrees = 0.5;
        for (cases, target, name) in [
            (&NEW_MOONS_2026[..], 0.0, "New Moon"),
            (&FULL_MOONS_2026[..], 0.5, "Full Moon"),
        ] {
            for &(y, mo, d, h, mi, label) in cases {
                let ts = utc(y, mo, d, h, mi);
                let off = degrees_off(cycle_position(ts), target);
                println!("{y}-{mo:02}-{d:02} {h:02}:{mi:02} {label}: {off:+.3}° (~{:+.0} min)", off / 0.508 * 60.0);
                assert!(off.abs() < tolerance_degrees, "{label} {y}-{mo}-{d}: off by {off}°");
                assert!(json_at(ts).contains(name), "{label}: {}", json_at(ts));
            }
        }
    }

    #[test]
    fn eclipses_are_exact_extremes_of_illumination() {
        let annular = utc(2026, 2, 17, 12, 1);
        let total_lunar = utc(2026, 3, 3, 11, 38);
        assert!(illumination(cycle_position(annular)) < 0.0001);
        assert!(illumination(cycle_position(total_lunar)) > 0.9999);
    }

    #[test]
    fn a_year_has_the_right_number_of_new_and_full_moons() {
        // 2026 has 12 new moons and 13 full moons (Jan 3 to Dec 24)
        let (mut news, mut fulls) = (0, 0);
        let mut t = utc(2026, 1, 1, 0, 0);
        let end = utc(2027, 1, 1, 0, 0);
        let mut previous = cycle_position(t);
        while t < end {
            t += HOUR;
            let p = cycle_position(t);
            if p < previous {
                news += 1;
            }
            if previous < 0.5 && p >= 0.5 {
                fulls += 1;
            }
            previous = p;
        }
        assert_eq!((news, fulls), (12, 13));
    }

    #[test]
    fn position_advances_smoothly_minute_by_minute() {
        // Within a lunation the elongation only grows, at roughly 0.4°–0.7°
        // per hour, and the printed result changes every hour.
        let start = utc(2026, 11, 10, 0, 0); // just after the Nov 9 new moon
        let mut previous = cycle_position(start);
        for minute in 1..(27 * 24 * 60) {
            let p = cycle_position(start + minute as f64 * MINUTE);
            let step_degrees = (p - previous) * 360.0;
            assert!(step_degrees > 0.0, "went backwards at minute {minute}");
            assert!(step_degrees < 0.7 / 60.0 * 1.2, "jumped at minute {minute}");
            previous = p;
        }
        let mut previous_json = json_at(start);
        for hour in 1..=48 {
            let json = json_at(start + hour as f64 * HOUR);
            assert_ne!(json, previous_json, "no change during hour {hour}");
            previous_json = json;
        }
    }

    #[test]
    fn fixed_point_formatting_rounds_and_pads() {
        assert_eq!(fixed(0.0, 4), "0.0000");
        assert_eq!(fixed(0.04, 4), "0.0400");
        assert_eq!(fixed(0.14645, 4), "0.1465");
        assert_eq!(fixed(0.99994, 4), "0.9999");
        assert_eq!(fixed(0.99996, 4), "1.0000");
        assert_eq!(fixed(1.0, 4), "1.0000");
        assert_eq!(fixed(1.2, 1), "1.2");
        assert_eq!(fixed(1.8, 1), "1.8");
        assert_eq!(fixed(3.0, 1), "3.0");
        assert_eq!(fixed(12.34, 1), "12.3");
    }

    #[test]
    fn json_contract() {
        assert_eq!(
            json_at_position(0.0001),
            "{\"phase_name\": \"New Moon\", \"torment_multiplier\": 1.0, \"illumination\": 0.0000}"
        );
        assert_eq!(
            json_at_position(0.125),
            "{\"phase_name\": \"Waxing Crescent\", \"torment_multiplier\": 1.2, \"illumination\": 0.1464}"
        );
        assert_eq!(
            json_at_position(0.25),
            "{\"phase_name\": \"First Quarter\", \"torment_multiplier\": 1.5, \"illumination\": 0.5000}"
        );
        assert_eq!(
            json_at_position(0.5),
            "{\"phase_name\": \"Full Moon\", \"torment_multiplier\": 3.0, \"illumination\": 1.0000}"
        );
        assert_eq!(
            json_at_position(0.875),
            "{\"phase_name\": \"Waning Crescent\", \"torment_multiplier\": 1.2, \"illumination\": 0.1464}"
        );
    }

    #[test]
    fn longest_result_fits_the_buffer() {
        // Longest phase name; must not be truncated
        let json = json_at_position(0.1);
        assert!(json.ends_with('}'));
        assert!(json.len() < RESULT_BUFFER_LEN - 1);
    }

    #[test]
    fn output_is_truncated_safely_in_a_small_buffer() {
        let mut buf = [0xFFu8; 16];
        let len = render_position(0.5, &mut buf);
        assert_eq!(len, 15);
        assert_eq!(buf[15], 0);
    }

    #[test]
    fn exported_entry_point_returns_the_rendered_json() {
        let timestamp = utc(2026, 11, 24, 14, 53);
        let ptr = calculate_moon_phase(timestamp);
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr.cast()) };
        assert_eq!(c_str.to_str().unwrap(), json_at(timestamp));
    }
}
