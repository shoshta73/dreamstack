use std::fmt;

use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub(crate) struct Level {
    pub(crate) number: u8,
    pub(crate) duration_seconds: u64,
    pub(crate) employers: Vec<Employer>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Employer {
    pub(crate) name: String,
    pub(crate) jobs: Vec<Job>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Job {
    pub(crate) name: String,
    pub(crate) hourly_pay: f64,
    pub(crate) company_reputation_per_second: f64,
    pub(crate) charisma_experience_per_second: f64,
}

impl Job {
    pub(crate) fn pay_for_seconds(&self, seconds: u64) -> f64 {
        self.hourly_pay / 3_600.0 * seconds as f64
    }

    pub(crate) fn company_reputation_for_seconds(&self, seconds: u64) -> f64 {
        self.company_reputation_per_second * seconds as f64
    }

    pub(crate) fn charisma_experience_for_seconds(&self, seconds: u64) -> f64 {
        self.charisma_experience_per_second * seconds as f64
    }
}

pub(crate) fn favor_for_reputation(reputation: f64) -> f64 {
    (1.0 + ((reputation + 25_000.0 / 25_500.0).log(1.02) + 1e-10).floor()) / 100.0
}

pub(crate) fn level_0() -> Level {
    Level {
        number: 0,
        duration_seconds: 8 * 60 * 60,
        employers: load_employers(),
    }
}

fn load_employers() -> Vec<Employer> {
    serde_json::from_str(include_str!("../data/employers.json"))
        .expect("data/employers.json should contain valid employers")
}

#[derive(Debug, Default)]
pub(crate) struct Clock {
    seconds: u64,
}

impl Clock {
    pub(crate) fn tick(&mut self) {
        self.seconds += 1;
    }

    pub(crate) fn advance_by(&mut self, seconds: u64) {
        self.seconds += seconds;
    }

    pub(crate) fn elapsed_seconds(&self) -> u64 {
        self.seconds
    }

    pub(crate) fn hour(&self) -> u64 {
        (self.seconds / 3_600) % 24
    }

    pub(crate) fn minute(&self) -> u64 {
        (self.seconds / 60) % 60
    }

    pub(crate) fn second(&self) -> u64 {
        self.seconds % 60
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hour(),
            self.minute(),
            self.second()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, level_0};
    use test_case::test_case;

    #[test]
    fn level_0_introduces_employee_job() {
        let level = level_0();

        assert_eq!(level.number, 0);
        assert_eq!(level.duration_seconds, 28_800);
        assert_eq!(level.employers.len(), 1);

        let employer = &level.employers[0];
        assert_eq!(employer.name, "employer0");
        assert_eq!(employer.jobs.len(), 1);

        let job = &employer.jobs[0];
        assert_eq!(job.name, "employee");
        assert_eq!(job.hourly_pay, 110.000);
        assert_eq!(job.company_reputation_per_second, 0.001);
        assert_eq!(job.charisma_experience_per_second, 0.200);
    }

    #[test]
    fn job_calculates_rewards_for_worked_seconds() {
        let level = level_0();
        let job = &level.employers[0].jobs[0];

        assert_eq!(job.pay_for_seconds(3_600), 110.000);
        assert_eq!(job.pay_for_seconds(level.duration_seconds), 880.000);
        assert_eq!(job.company_reputation_for_seconds(60), 0.060);
        assert_eq!(
            job.company_reputation_for_seconds(level.duration_seconds),
            28.800
        );
        assert_eq!(job.charisma_experience_for_seconds(60), 12.000);
        assert_eq!(
            job.charisma_experience_for_seconds(level.duration_seconds),
            5_760.000
        );
    }

    #[test_case(0.0, 0.0; "zero reputation")]
    #[test_case(0.0203921568627451, 0.01; "one favor threshold")]
    #[test_case(0.0403921568627451, 0.02; "two favor threshold")]
    fn calculates_favor_from_reputation(reputation: f64, favor: f64) {
        assert_eq!(super::favor_for_reputation(reputation), favor);
    }

    #[test_case(0, 0, 0, 0; "starts at midnight")]
    #[test_case(1, 0, 0, 1; "one second")]
    #[test_case(59, 0, 0, 59; "last second before minute")]
    #[test_case(60, 0, 1, 0; "one minute")]
    #[test_case(3_599, 0, 59, 59; "last second before hour")]
    #[test_case(3_600, 1, 0, 0; "one hour")]
    #[test_case(86_399, 23, 59, 59; "last second before day wraps")]
    #[test_case(86_400, 0, 0, 0; "day wraps")]
    fn reports_time_parts(seconds: u64, hour: u64, minute: u64, second: u64) {
        let clock = Clock { seconds };

        assert_eq!(clock.hour(), hour);
        assert_eq!(clock.minute(), minute);
        assert_eq!(clock.second(), second);
    }

    #[test]
    fn ticks_one_ingame_second() {
        let mut clock = Clock::default();

        clock.tick();

        assert_eq!(clock.elapsed_seconds(), 1);
        assert_eq!(clock.second(), 1);
    }

    #[test]
    fn advances_by_multiple_ingame_seconds() {
        let mut clock = Clock::default();

        clock.advance_by(120);

        assert_eq!(clock.elapsed_seconds(), 120);
        assert_eq!(clock.minute(), 2);
    }

    #[test]
    fn formats_time() {
        insta::assert_snapshot!(Clock { seconds: 3_661 }.to_string(), @"01:01:01");
    }
}
