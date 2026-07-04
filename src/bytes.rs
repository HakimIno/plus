pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn parse_size(value: &str) -> anyhow::Result<u64> {
    let trimmed = value.trim();
    let number_len = trimmed
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(trimmed.len());
    let number = trimmed[..number_len].trim().parse::<f64>()?;
    let unit = trimmed[number_len..].trim().to_ascii_lowercase();

    let multiplier = match unit.as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => anyhow::bail!("unknown size unit `{unit}`"),
    };

    Ok((number * multiplier) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    }

    #[test]
    fn parses_size_units() {
        assert_eq!(parse_size("5GiB").unwrap(), 5 * 1024 * 1024 * 1024);
        assert_eq!(parse_size("1.5 MiB").unwrap(), 1_572_864);
        assert!(parse_size("7 goats").is_err());
    }
}
