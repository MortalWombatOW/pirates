use bevy::prelude::*;

/// In-game time constants.
/// At 60Hz FixedUpdate, 1 in-game hour = ~1 real second.
pub const TICKS_PER_HOUR: u32 = 60;

/// Resource tracking in-game time progression.
/// 
/// The world clock advances on each FixedUpdate tick:
/// - `tick` increments each FixedUpdate
/// - When tick reaches `TICKS_PER_HOUR`, hour increments
/// - When hour reaches 24, day increments
/// 
/// Used by:
/// - Contract expiry (Epic 4.5.6)
/// - Price dynamics (Epic 4.4)
/// - Intel expiry (Epic 6.1.4)
/// - Faction AI (Epic 5.2)
#[derive(Resource, Debug, Clone)]
pub struct WorldClock {
    /// Current in-game day (1-indexed).
    pub day: u32,
    /// Current hour of the day (0-23).
    pub hour: u32,
    /// Current tick within the hour (0 to TICKS_PER_HOUR-1).
    pub tick: u32,
}

impl Default for WorldClock {
    fn default() -> Self {
        Self {
            day: 1,
            hour: 0,
            tick: 0,
        }
    }
}

impl WorldClock {
    /// Returns a formatted string for HUD display.
    /// Format: "Day X, Hour Y"
    pub fn formatted_time(&self) -> String {
        format!("Day {}, Hour {}", self.day, self.hour)
    }

    /// Returns total elapsed ticks since the start of the game.
    pub fn total_ticks(&self) -> u32 {
        let hours_total = (self.day - 1) * 24 + self.hour;
        hours_total * TICKS_PER_HOUR + self.tick
    }

    /// Advances the clock by one tick.
    pub fn advance(&mut self) {
        self.tick += 1;
        if self.tick >= TICKS_PER_HOUR {
            self.tick = 0;
            self.hour += 1;
            if self.hour >= 24 {
                self.hour = 0;
                self.day += 1;
                info!("New day: Day {}", self.day);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_clock() {
        let clock = WorldClock::default();
        assert_eq!(clock.day, 1);
        assert_eq!(clock.hour, 0);
        assert_eq!(clock.tick, 0);
    }

    #[test]
    fn test_formatted_time() {
        let clock = WorldClock { day: 3, hour: 14, tick: 30 };
        assert_eq!(clock.formatted_time(), "Day 3, Hour 14");
    }

    #[test]
    fn test_total_ticks() {
        let clock = WorldClock { day: 1, hour: 0, tick: 0 };
        assert_eq!(clock.total_ticks(), 0);

        let clock2 = WorldClock { day: 1, hour: 1, tick: 0 };
        assert_eq!(clock2.total_ticks(), TICKS_PER_HOUR);

        let clock3 = WorldClock { day: 2, hour: 0, tick: 0 };
        assert_eq!(clock3.total_ticks(), 24 * TICKS_PER_HOUR);
    }

    #[test]
    fn test_advance_tick() {
        let mut clock = WorldClock::default();
        clock.advance();
        assert_eq!(clock.tick, 1);
        assert_eq!(clock.hour, 0);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn test_advance_hour_rollover() {
        let mut clock = WorldClock { day: 1, hour: 0, tick: TICKS_PER_HOUR - 1 };
        clock.advance();
        assert_eq!(clock.tick, 0);
        assert_eq!(clock.hour, 1);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn test_advance_day_rollover() {
        let mut clock = WorldClock { day: 1, hour: 23, tick: TICKS_PER_HOUR - 1 };
        clock.advance();
        assert_eq!(clock.tick, 0);
        assert_eq!(clock.hour, 0);
        assert_eq!(clock.day, 2);
    }
}
