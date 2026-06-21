use crate::game::favor_for_reputation;

use serde::{Deserialize, Serialize};

const STARTING_MONEY: f64 = 1_024.0;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedPlayer {
    pub(crate) money: f64,
    pub(crate) charisma_experience: f64,
    pub(crate) charisma_skill: u8,
    pub(crate) hack_experience: f64,
    pub(crate) hack_skill: u8,
    #[serde(rename = "botanical_gardens")]
    pub(crate) exchange_marker: f64,
    pub(crate) company_standings: Vec<SavedCompanyStanding>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct SavedCompanyStanding {
    pub(crate) company_name: String,
    pub(crate) reputation: f64,
}

struct Stats {
    money: f64,
    charisma_experience: f64,
    charisma_skill: u8,
    hack_experience: f64,
    hack_skill: u8,
    exchange_marker: f64,
    company_standings: Vec<CompanyStanding>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            money: STARTING_MONEY,
            charisma_experience: 0.0,
            charisma_skill: 1,
            hack_experience: 0.0,
            hack_skill: 1,
            exchange_marker: 0.0,
            company_standings: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct CompanyStanding {
    company_name: String,
    reputation: f64,
}

#[derive(Default)]
pub(crate) struct Player {
    stats: Stats,
}

impl Player {
    pub(crate) fn from_save(saved: SavedPlayer) -> Self {
        Self {
            stats: Stats {
                money: saved.money,
                charisma_experience: saved.charisma_experience,
                charisma_skill: saved.charisma_skill,
                hack_experience: saved.hack_experience,
                hack_skill: saved.hack_skill,
                exchange_marker: saved.exchange_marker,
                company_standings: saved
                    .company_standings
                    .into_iter()
                    .map(|standing| CompanyStanding {
                        company_name: standing.company_name,
                        reputation: standing.reputation,
                    })
                    .collect(),
            },
        }
    }

    pub(crate) fn earn_money(&mut self, money: f64) {
        self.stats.money += money;
    }

    pub(crate) fn money(&self) -> f64 {
        self.stats.money
    }

    pub(crate) fn reset_money(&mut self) {
        self.stats.money = STARTING_MONEY;
    }

    pub(crate) fn gain_charisma_experience(&mut self, experience: f64) {
        self.stats.charisma_experience += experience;
    }

    pub(crate) fn charisma_experience(&self) -> f64 {
        self.stats.charisma_experience
    }

    pub(crate) fn charisma_skill(&self) -> u8 {
        self.stats.charisma_skill
    }

    pub(crate) fn clear_skill_experience(&mut self) {
        self.stats.charisma_experience = 0.0;
        self.stats.hack_experience = 0.0;
    }

    pub(crate) fn gain_hack_experience(&mut self, experience: f64) {
        self.stats.hack_experience += experience;
    }

    pub(crate) fn hack_experience(&self) -> f64 {
        self.stats.hack_experience
    }

    pub(crate) fn hack_skill(&self) -> u8 {
        self.stats.hack_skill
    }

    pub(crate) fn shift_exchange_marker(&mut self, exchange_marker: f64) {
        self.stats.exchange_marker += exchange_marker;
    }

    pub(crate) fn gain_company_reputation(&mut self, company_name: &str, reputation: f64) {
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
            company_name: company_name.to_string(),
            reputation,
        });
    }

    pub(crate) fn company_reputation(&self, company_name: &str) -> f64 {
        self.stats
            .company_standings
            .iter()
            .find(|standing| standing.company_name == company_name)
            .map_or(0.0, |standing| standing.reputation)
    }

    pub(crate) fn company_favor(&self, company_name: &str) -> f64 {
        favor_for_reputation(self.company_reputation(company_name))
    }

    pub(crate) fn clear_company_standings(&mut self) {
        self.stats.company_standings.clear();
    }

    pub(crate) fn to_save(&self) -> SavedPlayer {
        SavedPlayer {
            money: round_save_value(self.stats.money),
            charisma_experience: round_save_value(self.stats.charisma_experience),
            charisma_skill: self.stats.charisma_skill,
            hack_experience: round_save_value(self.stats.hack_experience),
            hack_skill: self.stats.hack_skill,
            exchange_marker: round_save_value(self.stats.exchange_marker),
            company_standings: self
                .stats
                .company_standings
                .iter()
                .map(|standing| SavedCompanyStanding {
                    company_name: standing.company_name.clone(),
                    reputation: round_save_value(standing.reputation),
                })
                .collect(),
        }
    }
}

fn round_save_value(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
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

        assert_eq!(player.money(), 1_034.5);
    }

    #[test]
    fn resets_money() {
        let mut player = Player::default();

        player.earn_money(880.0);

        player.reset_money();

        assert_eq!(player.money(), 1_024.0);
    }

    #[test]
    fn starts_with_money() {
        let player = Player::default();

        assert_eq!(player.money(), 1_024.0);
    }

    #[test]
    fn starts_with_level_1_charisma_skill() {
        let player = Player::default();

        assert_eq!(player.charisma_skill(), 1);
        assert_eq!(player.charisma_experience(), 0.0);
    }

    #[test]
    fn starts_with_level_1_hack_skill() {
        let player = Player::default();

        assert_eq!(player.hack_skill(), 1);
        assert_eq!(player.hack_experience(), 0.0);
    }

    #[test]
    fn gains_hack_experience() {
        let mut player = Player::default();

        player.gain_hack_experience(10.0);
        player.gain_hack_experience(15.0);

        assert_eq!(player.hack_experience(), 25.0);
    }

    #[test]
    fn clears_skill_experience() {
        let mut player = Player::default();

        player.gain_charisma_experience(10.0);
        player.gain_hack_experience(25.0);

        player.clear_skill_experience();

        assert_eq!(player.charisma_experience(), 0.0);
        assert_eq!(player.hack_experience(), 0.0);
        assert_eq!(player.charisma_skill(), 1);
        assert_eq!(player.hack_skill(), 1);
    }

    #[test]
    fn shifts_exchange_marker() {
        let mut player = Player::default();

        player.shift_exchange_marker(-1.0);
        player.shift_exchange_marker(0.25);

        assert_eq!(player.to_save().exchange_marker, -0.75);
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

    #[test]
    fn clears_company_standings() {
        let mut player = Player::default();

        player.gain_company_reputation("employer0", 0.1125);

        player.clear_company_standings();

        assert_eq!(player.company_reputation("employer0"), 0.0);
        assert_eq!(player.to_save().company_standings.len(), 0);
    }

    #[test]
    fn converts_to_save_data() {
        let mut player = Player::default();

        player.earn_money(10.5);
        player.gain_charisma_experience(0.400);
        player.shift_exchange_marker(-1.0);
        player.gain_company_reputation("employer0", 0.1125);

        let saved = player.to_save();

        assert_eq!(saved.money, 1_034.5);
        assert_eq!(saved.charisma_experience, 0.400);
        assert_eq!(saved.charisma_skill, 1);
        assert_eq!(saved.hack_experience, 0.0);
        assert_eq!(saved.hack_skill, 1);
        assert_eq!(saved.exchange_marker, -1.0);
        assert_eq!(saved.company_standings.len(), 1);
        assert_eq!(saved.company_standings[0].company_name, "employer0");
        assert_eq!(saved.company_standings[0].reputation, 0.1125);
    }

    #[test]
    fn restores_from_save_data() {
        let player = Player::from_save(super::SavedPlayer {
            money: 880.0,
            charisma_experience: 576.0,
            charisma_skill: 2,
            hack_experience: 25.0,
            hack_skill: 3,
            exchange_marker: -1.0,
            company_standings: vec![super::SavedCompanyStanding {
                company_name: "employer0".to_string(),
                reputation: 28.8,
            }],
        });

        assert_eq!(player.money(), 880.0);
        assert_eq!(player.charisma_experience(), 576.0);
        assert_eq!(player.charisma_skill(), 2);
        assert_eq!(player.hack_experience(), 25.0);
        assert_eq!(player.hack_skill(), 3);
        assert_eq!(player.company_reputation("employer0"), 28.8);
        assert_eq!(player.to_save().exchange_marker, -1.0);
    }

    #[test]
    fn rounds_save_data() {
        let mut player = Player::default();

        player.earn_money(879.9999999999998);
        player.gain_charisma_experience(576.0000000000002);
        player.gain_hack_experience(24.999999999999996);

        let saved = player.to_save();

        assert_eq!(saved.money, 1_904.0);
        assert_eq!(saved.charisma_experience, 576.0);
        assert_eq!(saved.hack_experience, 25.0);
    }
}
