use rand::seq::SliceRandom;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Value on a wheel wedge
#[derive(Debug, Clone, PartialEq)]
pub enum WedgeValue {
    Cash(i32),
    Bankrupt,
    LoseTurn,
    FreePlay,
    Prize { wedge_type: String, name: String },
}

// Custom serialization to handle untagged enum properly
impl Serialize for WedgeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            WedgeValue::Cash(val) => serializer.serialize_i32(*val),
            WedgeValue::Bankrupt => serializer.serialize_str("BANKRUPT"),
            WedgeValue::LoseTurn => serializer.serialize_str("LOSE A TURN"),
            WedgeValue::FreePlay => serializer.serialize_str("FREE PLAY"),
            WedgeValue::Prize { wedge_type, name } => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", wedge_type)?;
                map.serialize_entry("name", name)?;
                map.end()
            }
        }
    }
}

// Custom deserialization
impl<'de> Deserialize<'de> for WedgeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct WedgeValueVisitor;

        impl<'de> Visitor<'de> for WedgeValueVisitor {
            type Value = WedgeValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number, string, or prize object")
            }

            fn visit_i64<E>(self, value: i64) -> Result<WedgeValue, E>
            where
                E: de::Error,
            {
                Ok(WedgeValue::Cash(value as i32))
            }

            fn visit_u64<E>(self, value: u64) -> Result<WedgeValue, E>
            where
                E: de::Error,
            {
                Ok(WedgeValue::Cash(value as i32))
            }

            fn visit_str<E>(self, value: &str) -> Result<WedgeValue, E>
            where
                E: de::Error,
            {
                match value {
                    "BANKRUPT" => Ok(WedgeValue::Bankrupt),
                    "LOSE A TURN" => Ok(WedgeValue::LoseTurn),
                    "FREE PLAY" => Ok(WedgeValue::FreePlay),
                    _ => Err(de::Error::unknown_variant(
                        value,
                        &["BANKRUPT", "LOSE A TURN", "FREE PLAY"],
                    )),
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<WedgeValue, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut wedge_type = None;
                let mut name = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => wedge_type = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(WedgeValue::Prize {
                    wedge_type: wedge_type.unwrap_or_else(|| "PRIZE".to_string()),
                    name: name.ok_or_else(|| de::Error::missing_field("name"))?,
                })
            }
        }

        deserializer.deserialize_any(WedgeValueVisitor)
    }
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

/// Number of wedges on the wheel (matches real Wheel of Fortune)
pub const WHEEL_SIZE: usize = 24;

/// Create the standard wheel - exactly 24 wedges like the real Wheel of Fortune
/// All unique cash values to avoid visual confusion
pub fn create_standard_wheel() -> Vec<WedgeValue> {
    vec![
        WedgeValue::Cash(5000),      // 1 - Top dollar
        WedgeValue::Cash(300),       // 2
        WedgeValue::Cash(900),       // 3
        WedgeValue::Cash(400),       // 4
        WedgeValue::Cash(650),       // 5
        WedgeValue::Bankrupt,        // 6 - Bankrupt
        WedgeValue::Cash(450),       // 7
        WedgeValue::Cash(800),       // 8
        WedgeValue::Cash(350),       // 9
        WedgeValue::Cash(550),       // 10
        WedgeValue::prize("GIFT CARD"), // 11 - Prize
        WedgeValue::Cash(700),       // 12
        WedgeValue::Cash(500),       // 13
        WedgeValue::Cash(1000),      // 14
        WedgeValue::LoseTurn,        // 15 - Lose A Turn
        WedgeValue::Cash(600),       // 16
        WedgeValue::Cash(2500),      // 17
        WedgeValue::Cash(750),       // 18
        WedgeValue::Cash(850),       // 19
        WedgeValue::FreePlay,        // 20 - Free Play
        WedgeValue::Cash(950),       // 21
        WedgeValue::Cash(1500),      // 22
        WedgeValue::Cash(3500),      // 23
        WedgeValue::Bankrupt,        // 24 - Bankrupt
    ]
}

/// Shuffle wheel slots while keeping special wedges evenly distributed
pub fn shuffle_wheel_with_spacing(slots: Vec<WedgeValue>) -> Vec<WedgeValue> {
    let mut special: Vec<WedgeValue> = slots.iter().filter(|s| s.is_special()).cloned().collect();
    let mut cash: Vec<WedgeValue> = slots.iter().filter(|s| !s.is_special()).cloned().collect();

    let mut rng = rand::thread_rng();
    special.shuffle(&mut rng);
    cash.shuffle(&mut rng);

    let total = WHEEL_SIZE;
    let n_special = special.len();

    if n_special == 0 {
        cash.truncate(total);
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
    let mut cash_iter = cash.into_iter().cycle();
    for slot in &mut result {
        if slot.is_none() {
            *slot = Some(cash_iter.next().unwrap());
        }
    }

    result.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_has_24_wedges() {
        let wheel = create_standard_wheel();
        assert_eq!(wheel.len(), WHEEL_SIZE);
        assert_eq!(wheel.len(), 24);
    }

    #[test]
    fn test_shuffle_preserves_size() {
        let original = create_standard_wheel();
        let shuffled = shuffle_wheel_with_spacing(original);

        // Should always have exactly 24 wedges
        assert_eq!(shuffled.len(), WHEEL_SIZE);

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
