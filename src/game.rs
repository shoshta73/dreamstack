use std::fmt;

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
    use super::Clock;
    use test_case::test_case;

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
