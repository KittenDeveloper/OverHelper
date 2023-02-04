use directories::ProjectDirs;

use crate::seasons;


pub struct OverHelperSettings
{
	pub battle_pass_level: u8,
	pub battle_pass_target: u8,

	pub tank_wins: u8,
	pub damage_wins: u8,
	pub support_wins: u8,

	pub theme: iced::Theme,
}

pub fn settings_to_appstate(settings: &OverHelperSettings) -> crate::OverHelperApp
{
	crate::OverHelperApp
	{
		battle_pass_level: settings.battle_pass_level,
		battle_pass_target: settings.battle_pass_target,

		tank_wins: settings.tank_wins,
		damage_wins: settings.damage_wins,
		support_wins: settings.support_wins,

		theme: settings.theme.clone(),

		settings_page: None,
	}
}

pub fn appstate_to_settings(appstate: &crate::OverHelperApp) -> OverHelperSettings
{
	OverHelperSettings
	{
		battle_pass_level: appstate.battle_pass_level,
		battle_pass_target: appstate.battle_pass_target,

		tank_wins: appstate.tank_wins,
		damage_wins: appstate.damage_wins,
		support_wins: appstate.support_wins,

		theme: appstate.theme.clone(),
	}
}

pub fn get_settings_path() -> std::path::PathBuf
{
	let project_dirs = ProjectDirs::from("games", "partypurr", "OverHelper").expect("Could not find project directories");
	// Assert that the config directory exists
	std::fs::create_dir_all(project_dirs.config_dir()).expect("Could not create config directory");

	let settings_path = project_dirs.config_dir().join("settings.json"); // TODO: consider using a different file format
	settings_path
}

impl Default for OverHelperSettings
{
	fn default() -> Self
	{
		Self
		{
			battle_pass_level: 0,
			battle_pass_target: crate::seasons::PRESTIGE_BATTLE_PASS_END,

			tank_wins: 0,
			damage_wins: 0,
			support_wins: 0,

			theme: iced::Theme::Dark,
		}
	}
}

// These allow statements are needed because the settings file is not written in debug mode
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn write_settings(settings: &OverHelperSettings)
{
	// Don't write settings if in debug profile
	#[cfg(debug_assertions)]
	{
		return;
	}
	let settings_path = get_settings_path();
	let mut file = std::fs::File::create(settings_path).expect("Could not create settings file");
	use serde_json::json;
	let battle_pass_target = if settings.battle_pass_target == crate::seasons::PRESTIGE_BATTLE_PASS_END { "prestige" }
	else if settings.battle_pass_target == crate::seasons::MYTHIC_BATTLE_PASS_END { "mythic" }
	else
	{
		// Custom level
		let battle_pass_target = settings.battle_pass_target.to_string();
		// Warning: this leaks memory
		let battle_pass_target: &'static str = Box::leak(battle_pass_target.into_boxed_str());
		battle_pass_target
	};
	// Construct JSON
	let settings_json = json!
	(
		{
			"battle_pass":
			{
				"level": settings.battle_pass_level,
				"target": battle_pass_target
			},
			"roll_mastery":
			{
				"tank": settings.tank_wins,
				"damage": settings.damage_wins,
				"support": settings.support_wins
			}
		}
	);
	// Write JSON
	serde_json::to_writer_pretty(&mut file, &settings_json).expect("Could not write settings file");
}

pub fn load_settings_or_default() -> OverHelperSettings
{
	let settings_path = get_settings_path();
	if settings_path.exists()
	{
		// Load file
		use serde_json::Value;
		let file = std::fs::File::open(settings_path).expect("Could not open settings file");
		let reader = std::io::BufReader::new(file);
		let settings: Value = serde_json::from_reader(reader).expect("Could not parse settings file");
		let mut result: OverHelperSettings = OverHelperSettings::default();
		/*
		Example file:
		{
			"battle_pass"
			{
				"level": 0,
				"target": "prestige"
			},
			"roll_mastery":
			{
				"tank": 0,
				"damage": 0,
				"support": 0
			}
		}
		*/
		result.battle_pass_level = settings["battle_pass"]["level"].as_u64().expect("Could not parse battle pass level") as u8;
		if let Some(battle_pass_target) = settings["battle_pass"]["target"].as_str()
		{
			result.battle_pass_target = if battle_pass_target == "prestige" { crate::seasons::PRESTIGE_BATTLE_PASS_END } else { crate::seasons::MYTHIC_BATTLE_PASS_END };
		}
		else
		{
			// Else assume its a custom value
			result.battle_pass_target = settings["battle_pass"]["target"].as_u64().expect("Could not parse battle pass target") as u8;
		}

		result.tank_wins = settings["roll_mastery"]["tank"].as_u64().expect("Could not parse tank wins") as u8;
		result.damage_wins = settings["roll_mastery"]["damage"].as_u64().expect("Could not parse damage wins") as u8;
		result.support_wins = settings["roll_mastery"]["support"].as_u64().expect("Could not parse support wins") as u8;

		result
	}
	else
	{
		let settings = OverHelperSettings::default();
		// Create file
		write_settings(&settings);
		settings
	}
}



#[derive(Debug, Clone)]
pub struct SettingsPage
{
	pub battle_pass_custom_target: u8,
	pub battle_pass_target: seasons::BattlePassTargets,
	pub theme: iced::Theme
}

#[derive(Debug, Clone)]
pub enum Message
{
	BattlePassTargetChanged(seasons::BattlePassTargets),
	BattlePassCustomLevelChanged(String),
	ThemeChanged(SupportedThemes),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedThemes
{
	Dark,
	Light,
}

impl Into<iced::Theme> for SupportedThemes
{
	fn into(self) -> iced::Theme
	{
		match self
		{
			SupportedThemes::Dark => iced::Theme::Dark,
			SupportedThemes::Light => iced::Theme::Light,
		}
	}
}

const SUPPORTED_THEMES: [SupportedThemes; 2] = [SupportedThemes::Dark, SupportedThemes::Light];

impl std::fmt::Display for SupportedThemes
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		match self
		{
			SupportedThemes::Dark => write!(f, "Dark"),
			SupportedThemes::Light => write!(f, "Light"),
		}
	}
}

impl SettingsPage
{
	pub fn new() -> Self
	{
		let settings = crate::settings::load_settings_or_default();
		Self
		{
			battle_pass_custom_target: settings.battle_pass_target,
			battle_pass_target:
				if settings.battle_pass_target == crate::seasons::PRESTIGE_BATTLE_PASS_END
					{ seasons::BattlePassTargets::Prestige }
				else if settings.battle_pass_target == crate::seasons::MYTHIC_BATTLE_PASS_END
					{ seasons::BattlePassTargets::Mythic }
				else
					{ seasons::BattlePassTargets::Custom },
			theme: settings.theme
		}
	}

	pub fn view(&self) -> iced::Element<Message>
	{
		let current_battle_pass_target = self.battle_pass_target;
		let battle_pass_target_picker = iced::widget::pick_list::PickList::new
			(
				&seasons::BATTLE_PASS_TARGETS[..],
				Some(current_battle_pass_target),
				Message::BattlePassTargetChanged
			)
			.width(iced::Length::Fill)
			;
		let battle_pass_target_picker = iced::widget::Row::new()
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
			.push(battle_pass_target_picker)
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
			;
		let battle_pass_custom_level_picker: iced::Element<_> = match current_battle_pass_target
		{
			seasons::BattlePassTargets::Prestige | seasons::BattlePassTargets::Mythic => iced::widget::Space::with_width(iced::Length::Fill).into(), // Empty space
			seasons::BattlePassTargets::Custom => iced::widget::TextInput::new("Custom level (ex: 120)", &self.battle_pass_custom_target.to_string(), Message::BattlePassCustomLevelChanged)
				.width(iced::Length::Fill)
				.size(32)
				.into()
		};
		let battle_pass_custom_level_picker = iced::widget::Row::new()
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(2)))
			.push(battle_pass_custom_level_picker)
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(2)))
			;
		let current_theme = match self.theme
		{
			iced::Theme::Dark => SupportedThemes::Dark,
			iced::Theme::Light => SupportedThemes::Light,
			_ => unreachable!("Unsupported theme")
		};
		let theme_picker = iced::widget::pick_list::PickList::new
			(
				&SUPPORTED_THEMES[..],
				Some(current_theme),
				Message::ThemeChanged
			)
			.width(iced::Length::Fill)
			;
		let theme_picker = iced::widget::Row::new()
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
			.push(theme_picker)
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
			;
		iced::widget::Column::new()
			.push
			(
				iced::widget::Text::new("Battle Pass Target")
					.size(32)
					.horizontal_alignment(iced::alignment::Horizontal::Center)
					.vertical_alignment(iced::alignment::Vertical::Center)
					.width(iced::Length::Fill)
			)
			.push(battle_pass_target_picker)
			.push(battle_pass_custom_level_picker)
			.push
			(
				iced::widget::Text::new("Theme")
					.size(32)
					.horizontal_alignment(iced::alignment::Horizontal::Center)
					.vertical_alignment(iced::alignment::Vertical::Center)
					.width(iced::Length::Fill)
			)
			.push(theme_picker)
			.into()
	}

	pub fn update(&mut self, message: Message)
	{
		match message
		{
			Message::BattlePassTargetChanged(battle_pass_target) =>
			{
				match battle_pass_target
				{
					seasons::BattlePassTargets::Prestige => self.battle_pass_custom_target = crate::seasons::PRESTIGE_BATTLE_PASS_END,
					seasons::BattlePassTargets::Mythic => self.battle_pass_custom_target = crate::seasons::MYTHIC_BATTLE_PASS_END,
					seasons::BattlePassTargets::Custom => ()
				}
				self.battle_pass_target = battle_pass_target;
			},
			Message::BattlePassCustomLevelChanged(battle_pass_target) => self.battle_pass_custom_target = battle_pass_target.parse().unwrap_or_else(|e| {eprintln!("Invalid battle pass target: {}", e); 0}),
			Message::ThemeChanged(theme) => self.theme = match theme
			{
				SupportedThemes::Dark => iced::Theme::Dark,
				SupportedThemes::Light => iced::Theme::Light
			}
		}
	}
}
