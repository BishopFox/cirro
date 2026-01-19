use std::time::Duration;

pub fn fmt_duration(d: Duration) -> String {
    // total seconds as a float
    let secs_f = d.as_secs() as f64 + f64::from(d.subsec_nanos()) * 1e-9;

    if secs_f >= 3600.0 {
        // hours
        return format!("{:.1}h", secs_f / 3600.0);
    }
    if secs_f >= 60.0 {
        // minutes
        return format!("{:.1}m", secs_f / 60.0);
    }
    if secs_f >= 1.0 {
        // seconds
        return format!("{:.1}s", secs_f);
    }

    // now we're under 1 second:
    let ms_f = d.as_millis() as f64 + f64::from(d.subsec_nanos()) * 1e-6 % 1e3;
    if ms_f >= 1.0 {
        return format!("{:.1}ms", ms_f);
    }

    let us_f = d.as_micros() as f64 + f64::from(d.subsec_nanos()) * 1e-3 % 1e3;
    if us_f >= 1.0 {
        return format!("{:.1}µs", us_f);
    }

    // fallback to nanoseconds
    format!("{}ns", d.as_nanos())
}
