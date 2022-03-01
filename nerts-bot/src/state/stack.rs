use super::card::Card;

#[derive(Debug, Clone, Default)]
pub struct PlayedStack {
    pub cards: Vec<Card>,
}

impl PlayedStack {
    pub fn validate(&self) {
        // Check all cards are the right offset from each other
        // TODO: Check properly for flipped stacks
        // assert!(self
        //     .cards
        //     .iter()
        //     .zip(self.cards.iter().skip(1))
        //     .all(|(a, b)| a.position.y > b.position.y))
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn sort(&mut self, flipped: bool) {
        self.cards.sort_by_key(|c| c.position.y);
        if !flipped {
            self.cards.reverse();
        }
    }
}
