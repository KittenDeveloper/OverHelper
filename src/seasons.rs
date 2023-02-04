use chrono::TimeZone;

pub const SEASON_LENGTH: u8 = 63;
pub const PRESTIGE_BATTLE_PASS_END: u8 = 200;
pub const MYTHIC_BATTLE_PASS_END: u8 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlePassTargets
{
	Mythic,
	Prestige,
	Custom
}
pub const BATTLE_PASS_TARGETS: [BattlePassTargets; 3] = [BattlePassTargets::Mythic, BattlePassTargets::Prestige, BattlePassTargets::Custom];

impl std::fmt::Display for BattlePassTargets
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		match self
		{
			BattlePassTargets::Mythic => write!(f, "Mythic"),
			BattlePassTargets::Prestige => write!(f, "Prestige"),
			BattlePassTargets::Custom => write!(f, "Custom")
		}
	}
}

lazy_static::lazy_static!
{
	pub static ref SEASON_ONE_START: chrono::DateTime<chrono::Utc> = chrono::Utc.with_ymd_and_hms(2022, 10, 4, 0, 0, 0).unwrap();
	// pub static ref SEASON_TWO_START: chrono::DateTime<chrono::Utc> = chrono::Utc.with_ymd_and_hms(2022, 12, 6, 0, 0, 0).unwrap();
	// pub static ref SEASON_THREE_START: chrono::DateTime<chrono::Utc> = chrono::Utc.with_ymd_and_hms(2023, 2, 7, 0, 0, 0).unwrap();

	pub static ref CURRENT_SEASON_START: chrono::DateTime<chrono::Utc> = get_current_season_start();
	pub static ref CURRENT_SEASON_NUMBER: u8 = get_current_season_number();
}

fn get_current_season_start() -> chrono::DateTime<chrono::Utc>
{
	let mut season_start = *SEASON_ONE_START;
	while chrono::Utc::now() > season_start && season_start < chrono::Utc::now() + chrono::Duration::days(SEASON_LENGTH as i64)
	{
		season_start = season_start + chrono::Duration::days(SEASON_LENGTH as i64);
	}
	season_start = season_start - chrono::Duration::days(SEASON_LENGTH as i64); // Subtract one season length to get the start of the current season
	// println!("Current season start: {}", season_start);
	season_start
}

fn get_current_season_number() -> u8
{
	let mut season_number = 1;
	let mut season_start = *SEASON_ONE_START;
	while chrono::Utc::now() > season_start && season_start < chrono::Utc::now() + chrono::Duration::days(SEASON_LENGTH as i64)
	{
		season_start = season_start + chrono::Duration::days(SEASON_LENGTH as i64);
		season_number += 1;
	}
	season_number
}

pub fn get_levels_required_per_day(battle_pass_level: u8, battle_pass_target: u8) -> f64
{
	let remaining_days = (SEASON_LENGTH as i64) + (get_current_season_start() - chrono::Utc::now()).num_days();
	let remaining_levels = battle_pass_target - battle_pass_level;
	let levels_required_per_day = remaining_levels as f64 / remaining_days as f64;
	if levels_required_per_day < 0.0
	{
		0.0
	}
	else
	{
		levels_required_per_day
	}
}
