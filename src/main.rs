mod model;

use std::path::Path;

use crate::model::Season;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let usage_message = "Usage: f1-title-race <path/to/season.toml>";

    if args.len() != 2 {
        eprintln!("{usage_message}");
        std::process::exit(1);
    }

    let season_path = Path::new(args.get(1).expect(usage_message));
    run(season_path, &mut std::io::stdout())
}

fn run(season_path: &Path, out: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
    let season = Season::new(&season_path.display().to_string())?;
    season.validate()?;

    let max_points = season.max_points()?;
    let max_points_left = season.max_points_left()?;

    writeln!(
        out,
        "Maximum points a driver can earn in the season: {max_points}"
    )?;
    writeln!(out, "Maximum points left in the season: {max_points_left}")?;

    let standins = season.standings()?;
    let top_driver_points = standins[0].points;

    for (position, standing) in standins.iter().enumerate() {
        let can_catch = standing.can_catch(top_driver_points, max_points_left);
        writeln!(
            out,
            "{}{}:\t{},\tStill possible to win: {}\x1b[0m",
            if !can_catch { "\x1b[90m" } else { "\x1b[0m" },
            position + 1,
            standing,
            if can_catch { "yes" } else { "no" }
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_output() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let toml_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/2026-golden.toml"
        );
        run(Path::new(toml_path), &mut buf)?;

        let output = String::from_utf8(buf)?;
        assert!(output.starts_with("Maximum points a driver can earn in the season: 623"));
        assert!(output.contains("Kimi Antonelli\t(365)"));
        Ok(())
    }
}
