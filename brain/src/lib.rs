#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use core::panic::PanicInfo;

// Known New Moon: 2000-01-06 18:14:00 UTC
const KNOWN_NEW_MOON_UNIX: f64 = 947182440.0;
const SYNODIC_MONTH_DAYS: f64 = 29.53058770576;
const SYNODIC_MONTH_SECONDS: f64 = SYNODIC_MONTH_DAYS * 24.0 * 60.0 * 60.0;

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

/// Position within the synodic month: 0.0 = New Moon, 0.5 = Full Moon.
fn cycle_position(timestamp: f64) -> f64 {
    let cycles = (timestamp - KNOWN_NEW_MOON_UNIX) / SYNODIC_MONTH_SECONDS;

    // Manual floor (no_std has no f64::floor); valid for timestamps after the
    // reference New Moon.
    cycles - (cycles as u64) as f64
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
    let c = cos_within_half_pi(core::f64::consts::PI * (cycle_position - 0.5));
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
    let position = cycle_position(timestamp);
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

    fn json_at(timestamp: f64) -> String {
        let mut buf = [0xFFu8; RESULT_BUFFER_LEN];
        let len = render(timestamp, &mut buf);
        assert_eq!(buf[len], 0, "result must be NUL-terminated");
        String::from_utf8(buf[..len].to_vec()).unwrap()
    }

    fn at_position(position: f64) -> f64 {
        // 300 lunations after the reference New Moon (early 2024)
        KNOWN_NEW_MOON_UNIX + (300.0 + position) * SYNODIC_MONTH_SECONDS
    }

    fn fixed(f: f64, decimals: u32) -> String {
        let mut buf = [0u8; 32];
        let mut out = Out { buf: &mut buf, pos: 0 };
        out.fixed(f, decimals);
        let len = out.terminate();
        String::from_utf8(buf[..len].to_vec()).unwrap()
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

    #[test]
    fn cycle_position_wraps_each_synodic_month() {
        assert!(cycle_position(KNOWN_NEW_MOON_UNIX).abs() < 1e-9);
        for position in [0.1, 0.25, 0.5, 0.9] {
            let diff = (cycle_position(at_position(position)) - position).abs();
            assert!(diff < 1e-9, "position {position} off by {diff}");
        }
    }

    #[test]
    fn position_resolves_far_finer_than_a_tenth_of_a_cycle() {
        // The position within the cycle is a full f64 and is never rounded
        // before it picks the phase or the illumination.
        let start = at_position(0.2);
        let one_minute_of_cycle = 60.0 / SYNODIC_MONTH_SECONDS;
        let moved = cycle_position(start + 60.0) - cycle_position(start);
        assert!((moved - one_minute_of_cycle).abs() < 1e-9);

        // A phase boundary is honoured to within a minute either side...
        let boundary = at_position(0.22);
        assert!(json_at(boundary - 60.0).contains("Waxing Crescent"));
        assert!(json_at(boundary + 60.0).contains("First Quarter"));

        // ...and the reported illumination moves from one hour to the next.
        let mut previous = json_at(start);
        for hour in 1..=24 {
            let current = json_at(start + hour as f64 * 3600.0);
            assert_ne!(current, previous, "no change during hour {hour}");
            previous = current;
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
            json_at(at_position(0.0001)),
            "{\"phase_name\": \"New Moon\", \"torment_multiplier\": 1.0, \"illumination\": 0.0000}"
        );
        assert_eq!(
            json_at(at_position(0.125)),
            "{\"phase_name\": \"Waxing Crescent\", \"torment_multiplier\": 1.2, \"illumination\": 0.1464}"
        );
        assert_eq!(
            json_at(at_position(0.25)),
            "{\"phase_name\": \"First Quarter\", \"torment_multiplier\": 1.5, \"illumination\": 0.5000}"
        );
        assert_eq!(
            json_at(at_position(0.5)),
            "{\"phase_name\": \"Full Moon\", \"torment_multiplier\": 3.0, \"illumination\": 1.0000}"
        );
        assert_eq!(
            json_at(at_position(0.875)),
            "{\"phase_name\": \"Waning Crescent\", \"torment_multiplier\": 1.2, \"illumination\": 0.1464}"
        );
    }

    #[test]
    fn longest_result_fits_the_buffer() {
        // Longest phase name; must not be truncated
        let json = json_at(at_position(0.1));
        assert!(json.ends_with('}'));
        assert!(json.len() < RESULT_BUFFER_LEN - 1);
    }

    #[test]
    fn output_is_truncated_safely_in_a_small_buffer() {
        let mut buf = [0xFFu8; 16];
        let len = render(at_position(0.5), &mut buf);
        assert_eq!(len, 15);
        assert_eq!(buf[15], 0);
    }

    #[test]
    fn exported_entry_point_returns_the_rendered_json() {
        let timestamp = at_position(0.5);
        let ptr = calculate_moon_phase(timestamp);
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr.cast()) };
        assert_eq!(c_str.to_str().unwrap(), json_at(timestamp));
    }

    #[test]
    fn real_full_moons_of_2026_land_in_the_full_moon_window() {
        // Approximate UTC instants of astronomical full moons. The mean-moon
        // model drifts by hours from the true moon, well inside the window.
        let full_moons = [
            1767434580.0, // 2026-01-03 10:03
            1775095920.0, // 2026-04-02 02:12
            1782777420.0, // 2026-06-29 23:57
            1790441340.0, // 2026-09-26 16:49
            1795531980.0, // 2026-11-24 14:53
        ];
        for timestamp in full_moons {
            let json = json_at(timestamp);
            assert!(json.contains("\"Full Moon\""), "{timestamp}: {json}");
        }
    }
}
