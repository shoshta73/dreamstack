use crate::game::favor_for_reputation;

#[derive(Default)]
struct Stats {
    money: f64,
    charisma_experience: f64,
    company_standings: Vec<CompanyStanding>,
}

#[derive(Debug, PartialEq)]
struct CompanyStanding {
    company_name: &'static str,
    reputation: f64,
}

#[derive(Default)]
pub(crate) struct Player {
    stats: Stats,
}

impl Player {
    pub(crate) fn earn_money(&mut self, money: f64) {
        self.stats.money += money;
    }

    pub(crate) fn money(&self) -> f64 {
        self.stats.money
    }

    pub(crate) fn gain_charisma_experience(&mut self, experience: f64) {
        self.stats.charisma_experience += experience;
    }

    pub(crate) fn charisma_experience(&self) -> f64 {
        self.stats.charisma_experience
    }

    pub(crate) fn gain_company_reputation(&mut self, company_name: &'static str, reputation: f64) {
        if let Some(standing) = self
            .stats
            .company_standings
            .iter_mut()
            .find(|standing| standing.company_name == company_name)
        {
            standing.reputation += reputation;
            return;
        }

        self.stats.company_standings.push(CompanyStanding {
            company_name,
            reputation,
        });
    }

    pub(crate) fn company_reputation(&self, company_name: &'static str) -> f64 {
        self.stats
            .company_standings
            .iter()
            .find(|standing| standing.company_name == company_name)
            .map_or(0.0, |standing| standing.reputation)
    }

    pub(crate) fn company_favor(&self, company_name: &'static str) -> f64 {
        favor_for_reputation(self.company_reputation(company_name))
    }
}

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn gains_charisma_experience() {
        let mut player = Player::default();

        player.gain_charisma_experience(0.200);
        player.gain_charisma_experience(0.200);

        assert_eq!(player.charisma_experience(), 0.400);
    }

    #[test]
    fn earns_money() {
        let mut player = Player::default();

        player.earn_money(10.0);
        player.earn_money(0.5);

        assert_eq!(player.money(), 10.5);
    }

    #[test]
    fn gains_company_reputation_and_favor() {
        let mut player = Player::default();

        player.gain_company_reputation("employer0", 1.0 / 16.0);
        player.gain_company_reputation("employer0", 0.05);

        assert_eq!(player.company_reputation("employer0"), 0.1125);
        assert_eq!(player.company_favor("employer0"), 0.05);
        assert_eq!(player.company_reputation("unknown"), 0.0);
    }
}
