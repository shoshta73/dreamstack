use std::fmt;

#[derive(Debug, PartialEq)]
pub(crate) struct Level {
    pub(crate) number: u8,
    pub(crate) employers: Vec<Employer>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Employer {
    pub(crate) name: &'static str,
    pub(crate) jobs: Vec<Job>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Job {
    pub(crate) name: &'static str,
    pub(crate) hourly_pay: f64,
    pub(crate) company_reputation_per_second: f64,
    pub(crate) charisma_experience_per_second: f64,
}

pub(crate) fn level_0() -> Level {
    Level {
        number: 0,
        employers: vec![Employer {
            name: "employer0",
            jobs: vec![Job {
                name: "employee",
                hourly_pay: 110_000.0,
                company_reputation_per_second: 0.001,
                charisma_experience_per_second: 0.200,
            }],
        }],
    }
}

#[derive(Debug, Default)]
pub(crate) struct Clock {
    seconds: u64,
}

impl Clock {
    pub(crate) fn tick(&mut self) {
        self.seconds += 1;
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
        assert_eq!(level.employers.len(), 1);

        let employer = &level.employers[0];
        assert_eq!(employer.name, "employer0");
        assert_eq!(employer.jobs.len(), 1);

        let job = &employer.jobs[0];
        assert_eq!(job.name, "employee");
        assert_eq!(job.hourly_pay, 110_000.0);
        assert_eq!(job.company_reputation_per_second, 0.001);
        assert_eq!(job.charisma_experience_per_second, 0.200);
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

        assert_eq!(clock.second(), 1);
    }

    #[test]
    fn formats_time() {
        insta::assert_snapshot!(Clock { seconds: 3_661 }.to_string(), @"01:01:01");
    }
}
