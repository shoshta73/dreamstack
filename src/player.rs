#[derive(Default)]
struct Stats {
    charisma_experience: f64,
}

#[derive(Default)]
pub(crate) struct Player {
    stats: Stats,
}

impl Player {
    pub(crate) fn gain_charisma_experience(&mut self, experience: f64) {
        self.stats.charisma_experience += experience;
    }

    pub(crate) fn charisma_experience(&self) -> f64 {
        self.stats.charisma_experience
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
}
