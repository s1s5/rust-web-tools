use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[graphql(shareable)]
pub struct Month {
    pub year: i32,
    pub month: i32,
}

impl Month {
    pub fn from_i32(month: i32) -> Self {
        Self {
            year: month / 12 + 2000,
            month: month % 12 + 1,
        }
    }
    pub fn to_i32(&self) -> i32 {
        (self.year - 2000) * 12 + self.month - 1
    }
}

#[derive(InputObject, Debug, Clone)]
pub struct MonthInput {
    pub year: i32,
    pub month: i32,
}

impl From<MonthInput> for Month {
    fn from(value: MonthInput) -> Self {
        Self {
            year: value.year,
            month: value.month,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let month = Month {
            year: 2024,
            month: 10,
        };
        let m_i32 = month.to_i32();
        assert_eq!(m_i32, 24 * 12 + 10 - 1);
        let d = Month::from_i32(m_i32);
        assert_eq!(d.year, 2024);
        assert_eq!(d.month, 10);

        Ok(())
    }
}
