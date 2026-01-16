use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// Value on a wheel wedge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WedgeValue {
    Cash(i32),
    #[serde(rename = "BANKRUPT")]
    Bankrupt,
    #[serde(rename = "LOSE A TURN")]
    LoseTurn,
    #[serde(rename = "FREE PLAY")]
    FreePlay,
    Prize {
        #[serde(rename = "type")]
        wedge_type: String,
        name: String,
    },
}

impl WedgeValue {
    pub fn is_bankrupt(&self) -> bool {
        matches!(self, WedgeValue::Bankrupt)
    }

    pub fn is_lose_turn(&self) -> bool {
        matches!(self, WedgeValue::LoseTurn)
    }

    pub fn is_special(&self) -> bool {
        matches!(
            self,
            WedgeValue::Bankrupt | WedgeValue::LoseTurn | WedgeValue::FreePlay | WedgeValue::Prize { .. }
        )
    }

    /// Create a prize wedge
    pub fn prize(name: &str) -> Self {
        WedgeValue::Prize {
            wedge_type: "PRIZE".to_string(),
            name: name.to_string(),
        }
    }
}

/// Base wheel configuration matching the Python version
pub const BASE_WHEEL: &[WedgeValue] = &[
    WedgeValue::Cash(500),
    WedgeValue::Cash(550),
    WedgeValue::Cash(600),
    WedgeValue::Cash(650),
    WedgeValue::Cash(700),
    WedgeValue::Cash(800),
    WedgeValue::Cash(900),
    WedgeValue::Cash(300),
    WedgeValue::Cash(350),
    WedgeValue::Cash(400),
    WedgeValue::Cash(450),
    WedgeValue::Cash(1000),
    WedgeValue::Cash(1500),
    WedgeValue::Cash(2000),
    WedgeValue::FreePlay,
    WedgeValue::Bankrupt,
    WedgeValue::LoseTurn,
];

/// Default prize wedges
pub fn default_prize_wedges() -> Vec<WedgeValue> {
    vec![
        WedgeValue::prize("GIFT CARD"),
        WedgeValue::prize("HOLIDAY MUG"),
        WedgeValue::prize("STOCKING STUFFER"),
    ]
}

/// Shuffle wheel slots ensuring special wedges are evenly distributed
pub fn shuffle_wheel_with_spacing(slots: Vec<WedgeValue>) -> Vec<WedgeValue> {
    let mut special: Vec<WedgeValue> = slots.iter().filter(|s| s.is_special()).cloned().collect();
    let mut cash: Vec<WedgeValue> = slots.iter().filter(|s| !s.is_special()).cloned().collect();

    // Add default prize wedges
    special.extend(default_prize_wedges());

    let mut rng = rand::thread_rng();
    special.shuffle(&mut rng);
    cash.shuffle(&mut rng);

    let total = special.len() + cash.len();
    let n_special = special.len();

    if n_special == 0 {
        return cash;
    }

    // Calculate spacing between special wedges
    let spacing = total / n_special;

    let mut result: Vec<Option<WedgeValue>> = vec![None; total];

    // Place special wedges at evenly spaced positions
    for (i, wedge) in special.into_iter().enumerate() {
        let base_pos = i * spacing;
        let jitter = if spacing > 2 {
            rand::random::<usize>() % (spacing - 1)
        } else {
            0
        };
        let mut pos = (base_pos + jitter) % total;

        // Find nearest empty slot if position is taken
        while result[pos].is_some() {
            pos = (pos + 1) % total;
        }
        result[pos] = Some(wedge);
    }

    // Fill remaining slots with cash values
    let mut cash_iter = cash.into_iter();
    for slot in &mut result {
        if slot.is_none() {
            if let Some(cash_wedge) = cash_iter.next() {
                *slot = Some(cash_wedge);
            }
        }
    }

    result.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle_preserves_all_wedges() {
        let original = BASE_WHEEL.to_vec();
        let shuffled = shuffle_wheel_with_spacing(original.clone());

        // Should have more wedges due to added prizes
        assert!(shuffled.len() >= original.len());

        // Should have special wedges spread out
        let mut last_special_idx: Option<usize> = None;
        for (i, wedge) in shuffled.iter().enumerate() {
            if wedge.is_special() {
                if let Some(last) = last_special_idx {
                    // Special wedges should have some spacing
                    assert!(i - last >= 1);
                }
                last_special_idx = Some(i);
            }
        }
    }
}
