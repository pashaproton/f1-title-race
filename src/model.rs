use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    ops::{Add, AddAssign, Mul},
    str::FromStr,
};

use serde::Deserialize;

#[derive(Debug)]
pub enum ValidationError {
    UnknownDriverInRace(Driver, String),
    EmptySeason,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::UnknownDriverInRace(driver, race_name) => {
                write!(f, "Unknown driver {driver} in race {race_name}")
            }
            ValidationError::EmptySeason => write!(f, "Season has no races"),
        }
    }
}

impl Error for ValidationError {}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Driver(pub String);

impl fmt::Display for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for Driver {
    fn from(s: &'static str) -> Self {
        Driver(s.to_string())
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Points(pub i32);

impl Points {
    pub fn new(points: i32) -> Self {
        Points(points)
    }
}

impl Add for Points {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Points(self.0 + rhs.0)
    }
}

impl Mul<i32> for Points {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Points(self.0 * rhs)
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AddAssign for Points {
    fn add_assign(&mut self, rhs: Points) {
        self.0 += rhs.0;
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Race {
    pub name: String,
    pub sprint: bool,
    #[allow(clippy::struct_field_names)]
    pub race_result: Option<Vec<Driver>>,
    pub sprint_result: Option<Vec<Driver>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Standing {
    pub driver: Driver,
    pub points: Points,
    pub max_points: Points,
}

impl Standing {
    pub fn can_catch(&self, leader: Points, max_left: Points) -> bool {
        self.points + max_left >= leader
    }
}

impl fmt::Display for Standing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let full = 100;
        let dots = (full * self.points.0) / self.max_points.0;
        let spaces = full - dots;
        write!(
            f,
            "{}{}{}{}({})",
            "•".repeat(dots as usize),
            " ".repeat(spaces as usize),
            self.driver,
            if self.driver.0.len() < 12 {
                "\t\t"
            } else {
                "\t"
            },
            self.points
        )
    }
}

#[derive(Deserialize)]
pub struct Season {
    races: Option<Vec<Race>>,
    drivers: Vec<Driver>,
    race_points: Vec<Points>,
    sprint_points: Vec<Points>,
}

impl Season {
    pub fn new(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(file_name)?;
        let season: Season = toml::from_str(&file_content)?;
        Ok(season)
    }

    fn get_races(&self) -> Result<&[Race], ValidationError> {
        self.races.as_deref().ok_or(ValidationError::EmptySeason)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let races = self.get_races()?;

        for race in races {
            for driver in race
                .race_result
                .iter()
                .flatten()
                .chain(race.sprint_result.iter().flatten())
            {
                if !self.drivers.contains(driver) {
                    return Err(ValidationError::UnknownDriverInRace(
                        driver.clone(),
                        race.name.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn top_point(&self) -> Points {
        self.race_points[0]
    }

    pub fn top_sprint_point(&self) -> Points {
        self.sprint_points[0]
    }

    pub fn total_races(&self) -> Result<i32, ValidationError> {
        let races = self.get_races()?;
        i32::try_from(races.len()).map_err(|_| ValidationError::EmptySeason)
    }

    pub fn total_sprints(&self) -> Result<i32, ValidationError> {
        let races = self.get_races()?;
        Ok(i32::try_from(races.iter().filter(|r| r.sprint).count()).unwrap_or(0))
    }

    pub fn completed_races(&self) -> Result<Vec<&[Driver]>, ValidationError> {
        let races = self.get_races()?;
        Ok(races
            .iter()
            .filter_map(|r| r.race_result.as_deref())
            .collect())
    }

    pub fn completed_sprints(&self) -> Result<Vec<&[Driver]>, ValidationError> {
        let races = self.get_races()?;
        Ok(races
            .iter()
            .filter_map(|r| r.sprint_result.as_deref())
            .collect())
    }

    pub fn max_points(&self) -> Result<Points, ValidationError> {
        let total_races = self.total_races()?;
        let total_sprints = self.total_sprints()?;
        Ok(self.top_point() * total_races + self.top_sprint_point() * total_sprints)
    }

    pub fn max_points_left(&self) -> Result<Points, ValidationError> {
        let total_races = self.total_races()?;
        let total_sprints = self.total_sprints()?;

        let completed_races = self.completed_races()?;
        let completed_sprints = self.completed_sprints()?;

        let completed_races_len = i32::try_from(completed_races.len()).unwrap_or(0);
        let completed_sprints_len = i32::try_from(completed_sprints.len()).unwrap_or(0);

        Ok(self.top_point() * (total_races - completed_races_len)
            + self.top_sprint_point() * (total_sprints - completed_sprints_len))
    }

    pub fn standings(&self) -> Result<Vec<Standing>, ValidationError> {
        let mut summary: HashMap<Driver, Points> = HashMap::new();

        let completed_races = self.completed_races()?;
        let completed_sprints = self.completed_sprints()?;

        Self::score(completed_races, &self.race_points, &mut summary);
        Self::score(completed_sprints, &self.sprint_points, &mut summary);

        let max_points = self.max_points()?;

        let mut summary_vec: Vec<Standing> = summary
            .into_iter()
            .map(|summary| Standing {
                driver: summary.0,
                points: summary.1,
                max_points,
            })
            .collect();

        summary_vec.sort_by(|a, b| b.points.cmp(&a.points).then(a.driver.cmp(&b.driver)));

        if summary_vec.is_empty() {
            return Err(ValidationError::EmptySeason);
        }

        Ok(summary_vec)
    }

    fn score<R, I>(results: I, table: &[Points], summary: &mut HashMap<Driver, Points>)
    where
        I: IntoIterator<Item = R>,
        R: AsRef<[Driver]>,
    {
        for result in results {
            for (driver, point) in result.as_ref().iter().zip(table) {
                *summary.entry(driver.clone()).or_insert(Points::new(0)) += *point;
            }
        }
    }
}

impl FromStr for Season {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let season: Season = toml::from_str(s)?;
        Ok(season)
    }
}

#[cfg(test)]
mod model_tests {
    use super::*;

    #[test]
    fn test_score() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 1"
            sprint = false
            race_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 2"
            sprint = false
            race_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 3"
            sprint = false
            "#
        .parse::<Season>()?;

        let standings = season.standings()?;
        let max_points = season.max_points()?;
        let max_points_left = season.max_points_left()?;

        assert_eq!(
            standings[0].points,
            Points(50),
            "Kimi Antonelli should have 50 points"
        );
        assert_eq!(
            standings[1].points,
            Points(36),
            "Lewis Hamilton should have 36 points"
        );
        assert_eq!(
            standings[2].points,
            Points(30),
            "Charles Leclerc should have 30 points"
        );
        assert_eq!(
            max_points,
            Points(75),
            "Maximum points a driver can earn in the season should be 75"
        );
        assert_eq!(
            max_points_left,
            Points(25),
            "Maximum points left in the season should be 25"
        );

        Ok(())
    }

    #[test]
    fn test_standings_can_catch() {
        let standing = Standing {
            driver: Driver::from("Kimi Antonelli"),
            points: Points(100),
            max_points: Points(0),
        };
        let leader_points = Points(150);
        let max_points_left = Points(60);

        assert!(
            standing.can_catch(leader_points, max_points_left),
            "Kimi Antonelli should be able to catch the leader"
        );

        let max_points_left = Points(40);
        assert!(
            !standing.can_catch(leader_points, max_points_left),
            "Kimi Antonelli should not be able to catch the leader"
        );
    }

    #[test]
    fn test_standing_sorting() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell"]

            [[races]]
            name = "Race 1"
            sprint = false
            race_result = ["Kimi Antonelli", "George Russell"]

            [[races]]
            name = "Race 2"
            sprint = false
            race_result = ["George Russell", "Kimi Antonelli"]

            [[races]]
            name = "Race 3"
            sprint = false
            "#
        .parse::<Season>()?;
        let standings = season.standings()?;

        assert_eq!(
            standings[0],
            Standing {
                driver: Driver::from("George Russell"),
                points: Points(43),
                max_points: Points(75),
            }
        );
        assert_eq!(
            standings[1],
            Standing {
                driver: Driver::from("Kimi Antonelli"),
                points: Points(43),
                max_points: Points(75),
            }
        );

        Ok(())
    }

    #[test]
    fn test_sprints_only() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell", "Lewis Hamilton", "Charles Leclerc", "Lando Norris", "Oscar Piastri", "Liam Lawson", "Oliver Bearman"]

            [[races]]
            name = "China"
            sprint = true
            sprint_result = [ "George Russell", "Charles Leclerc", "Lewis Hamilton", "Lando Norris", "Kimi Antonelli", "Oscar Piastri", "Liam Lawson", "Oliver Bearman"]
            "#
            .parse::<Season>()?;
        let standings = season.standings()?;

        assert_eq!(standings.len(), 8);
        assert_eq!(
            standings[0],
            Standing {
                driver: Driver::from("George Russell"),
                points: Points(8),
                max_points: Points(33),
            }
        );
        assert_eq!(
            standings[7],
            Standing {
                driver: Driver::from("Oliver Bearman"),
                points: Points(1),
                max_points: Points(33),
            }
        );

        Ok(())
    }

    #[test]
    fn test_empty_season() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell", "Lewis Hamilton"]
        "#
        .parse::<Season>()?;
        assert!(matches!(
            season.standings(),
            Err(ValidationError::EmptySeason)
        ));
        Ok(())
    }

    #[test]
    fn test_unknown_driver() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 1"
            sprint = false
            race_result = ["Michael Schumacher", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 2"
            sprint = false
            race_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]
        "#
        .parse::<Season>()?;

        assert!(matches!(
            season.validate(),
            Err(ValidationError::UnknownDriverInRace(_, _))
        ));

        Ok(())
    }

    #[test]
    fn test_validation_error_display() {
        let e = ValidationError::UnknownDriverInRace(
            Driver::from("Michael Schumacher"),
            "Race 1".into(),
        );
        assert_eq!(
            e.to_string(),
            "Unknown driver Michael Schumacher in race Race 1"
        );
        assert_eq!(
            ValidationError::EmptySeason.to_string(),
            "Season has no races"
        );
    }

    #[test]
    fn test_max_points() -> Result<(), Box<dyn std::error::Error>> {
        let season = r#"
            race_points = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1]

            sprint_points = [8, 7, 6, 5, 4, 3, 2, 1]

            drivers = [ "Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 1"
            sprint = false
            race_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]

            [[races]]
            name = "Race 2"
            sprint = true
            sprint_result = ["Kimi Antonelli", "George Russell", "Lewis Hamilton"]
        "#
        .parse::<Season>()?;

        let max_points = season.max_points()?;
        let max_points_left = season.max_points_left()?;
        assert_eq!(max_points, Points(58));
        assert_eq!(max_points_left, Points(25));

        Ok(())
    }

    // #[test]
    // fn test_points_operators() {
    //     assert_eq!(Points(25) * Points(2), Points(50));
    //     assert_eq!(Points(50) - Points(18), Points(32));
    //     assert_eq!(Points(25) + Points(25), Points(50));
    // }

    #[test]
    fn test_standing_display() {
        let s = Standing {
            driver: Driver::from("Kimi Antonelli"),
            points: Points(60),
            max_points: Points(100),
        };
        let out = s.to_string();
        assert_eq!(out.matches('•').count(), 60);
        assert_eq!(out.matches(' ').count(), 41); // plus one space between first and last name
    }
}
