use std::fmt;

use serde::Deserialize;

const FAVOR_UNIT: f64 = 0.01;
const COMPANY_REPUTATION_RATE_BONUS_PER_FAVOR: f64 = 0.005;

#[derive(Debug, PartialEq)]
pub(crate) struct Tutorial {
    pub(crate) number: u8,
    pub(crate) duration_seconds: u64,
    pub(crate) employers: Vec<Employer>,
    pub(crate) servers: Vec<Server>,
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

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Server {
    pub(crate) name: String,
    pub(crate) hack_skill_needed: u8,
    pub(crate) min_security: f64,
    pub(crate) max_money: ServerMoney,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ServerMoney {
    quantifier: f64,
    multiplier: u64,
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

impl Server {
    pub(crate) fn max_money(&self) -> f64 {
        self.max_money.quantifier * self.max_money.multiplier as f64
    }

    pub(crate) fn hack_experience_reward(&self) -> f64 {
        self.min_security * 25.0
    }
}

pub(crate) fn favor_for_reputation(reputation: f64) -> f64 {
    (1.0 + ((reputation + 25_000.0 / 25_500.0).log(1.02) + 1e-10).floor()) / 100.0
}

pub(crate) fn company_reputation_rate_multiplier(favor: f64) -> f64 {
    1.0 + favor / FAVOR_UNIT * COMPANY_REPUTATION_RATE_BONUS_PER_FAVOR
}

pub(crate) fn tutorial_0() -> Tutorial {
    Tutorial {
        number: 0,
        duration_seconds: 8 * 60 * 60,
        employers: load_employers(),
        servers: Vec::new(),
    }
}

pub(crate) fn tutorial_1() -> Tutorial {
    Tutorial {
        number: 1,
        duration_seconds: 8 * 60 * 60,
        employers: load_employers(),
        servers: load_servers(),
    }
}

fn load_employers() -> Vec<Employer> {
    serde_json::from_str(include_str!("../data/employers.json"))
        .expect("data/employers.json should contain valid employers")
}

fn load_servers() -> Vec<Server> {
    serde_json::from_str(include_str!("../data/servers.json"))
        .expect("data/servers.json should contain valid servers")
}

#[derive(Debug, Default)]
pub(crate) struct Clock {
    seconds: u64,
}

impl Clock {
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
    use super::{Clock, tutorial_0, tutorial_1};
    use test_case::test_case;

    #[test]
    fn tutorial_0_introduces_employee_job() {
        let tutorial = tutorial_0();

        assert_eq!(tutorial.number, 0);
        assert_eq!(tutorial.duration_seconds, 28_800);
        assert_eq!(tutorial.employers.len(), 1);
        assert_eq!(tutorial.servers.len(), 0);

        let employer = &tutorial.employers[0];
        assert_eq!(employer.name, "employer0");
        assert_eq!(employer.jobs.len(), 1);

        let job = &employer.jobs[0];
        assert_eq!(job.name, "employee");
        assert_eq!(job.hourly_pay, 7.000);
        assert_eq!(job.company_reputation_per_second, 0.001);
        assert_eq!(job.charisma_experience_per_second, 0.020);
    }

    #[test]
    fn tutorial_1_introduces_hacking_server() {
        let tutorial = tutorial_1();

        assert_eq!(tutorial.number, 1);
        assert_eq!(tutorial.duration_seconds, 28_800);
        assert_eq!(tutorial.employers.len(), 1);
        assert_eq!(tutorial.servers.len(), 1);

        let server = &tutorial.servers[0];
        assert_eq!(server.name, "server0");
        assert_eq!(server.hack_skill_needed, 1);
        assert_eq!(server.min_security, 1.0);
        assert_eq!(server.max_money(), 100_000.0);
        assert_eq!(server.hack_experience_reward(), 25.0);
    }

    #[test]
    fn job_calculates_rewards_for_worked_seconds() {
        let tutorial = tutorial_0();
        let job = &tutorial.employers[0].jobs[0];

        assert_eq!(job.pay_for_seconds(3_600), 7.000);
        assert_eq!(job.pay_for_seconds(tutorial.duration_seconds), 56.000);
        assert_eq!(job.company_reputation_for_seconds(60), 0.060);
        assert_eq!(
            job.company_reputation_for_seconds(tutorial.duration_seconds),
            28.800
        );
        assert_eq!(job.charisma_experience_for_seconds(60), 1.200);
        assert_eq!(
            job.charisma_experience_for_seconds(tutorial.duration_seconds),
            576.000
        );
    }

    #[test_case(0.0, 0.0; "zero reputation")]
    #[test_case(0.0203921568627451, 0.01; "one favor threshold")]
    #[test_case(0.0403921568627451, 0.02; "two favor threshold")]
    fn calculates_favor_from_reputation(reputation: f64, favor: f64) {
        assert_eq!(super::favor_for_reputation(reputation), favor);
    }

    #[test_case(0.0, 1.0; "no favor")]
    #[test_case(0.01, 1.005; "one favor")]
    #[test_case(0.05, 1.025; "five favor")]
    fn calculates_company_reputation_rate_multiplier(favor: f64, multiplier: f64) {
        assert_eq!(super::company_reputation_rate_multiplier(favor), multiplier);
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

        clock.advance_by(1);

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
