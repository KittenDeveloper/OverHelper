use iced::Application;

mod seasons;
mod settings;

fn main()
{
	let settings = iced::settings::Settings
	{
		exit_on_close_request: false, // save on exit
		..Default::default()
	};
	// Start the iced application
	OverHelperApp::run(settings)
		.unwrap();
}

#[derive(Debug, Clone)]
pub enum Message
{
	UpdateBattlePassLevel(u8),

	UpdateTankWins(i8),
	UpdateDamageWins(i8),
	UpdateSupportWins(i8),
	ResetWins,

	EventOccurred(iced_native::event::Event),

	OpenSettings,
	ExitSettings,
	SettingsMessage(settings::Message),
}

#[derive(Debug, Clone)]
pub struct OverHelperApp
{
	pub battle_pass_level: u8,
	pub battle_pass_target: u8, // Should be prestige or 80
	// Roll mastery
	pub tank_wins: u8,
	pub damage_wins: u8,
	pub support_wins: u8,

	pub theme: iced::Theme,

	pub settings_page: Option<settings::SettingsPage>,
}

impl Application for OverHelperApp
{
	type Executor = iced::executor::Default;
	type Flags = ();
	type Message = Message;
	type Theme = iced::theme::Theme;

	fn new(_flags: ()) -> (Self, iced::Command<Self::Message>)
	{
		(
			settings::settings_to_appstate(&settings::load_settings_or_default()),
			iced::Command::none()
		)
	}

	fn title(&self) -> String
	{
		String::from("OverHelper")
	}

	fn subscription(&self) -> iced::Subscription<Self::Message>
	{
		iced_native::subscription::events().map(Message::EventOccurred)
	}

	fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message>
	{
		match message
		{
			Message::UpdateBattlePassLevel(new_level) =>
			{
				self.battle_pass_level = new_level;
			},
			Message::UpdateTankWins(delta) =>
			{
				self.tank_wins = (self.tank_wins as i8 + delta) as u8;
			},
			Message::UpdateDamageWins(delta) =>
			{
				self.damage_wins = (self.damage_wins as i8 + delta) as u8;
			},
			Message::UpdateSupportWins(delta) =>
			{
				self.support_wins = (self.support_wins as i8 + delta) as u8;
			},
			Message::ResetWins =>
			{
				self.tank_wins = 0;
				self.damage_wins = 0;
				self.support_wins = 0;
			},
			Message::EventOccurred(event) =>
			{
				match event
				{
					iced_native::event::Event::Window(iced_native::window::Event::CloseRequested) => // Write settings to file on close
					{
						let settings = settings::appstate_to_settings(&self);
						settings::write_settings(&settings);
						return iced::window::close();
					},
					_ => (),
				}
			},
			Message::ExitSettings =>
			{
				// Use the new settings
				let settings = self.settings_page.clone().unwrap();
				self.battle_pass_target = settings.battle_pass_custom_target;
				self.battle_pass_level = std::cmp::min(self.battle_pass_level, self.battle_pass_target);
				self.theme = settings.theme.into();
				self.settings_page = None;
			},
			Message::OpenSettings =>
			{
				self.settings_page = Some(settings::SettingsPage::new());
			},
			Message::SettingsMessage(settings_message) =>
			{
				self.settings_page.as_mut().unwrap().update(settings_message);
			},
		}
		iced::Command::none()
	}


	fn view(&self) -> iced::Element<Self::Message>
	{
		if self.settings_page.is_some()
		{
			let page = self.settings_page
				.as_ref()
				.unwrap()
				.view()
				.map(Message::SettingsMessage)
				;
			let page = iced::widget::Container::new(page)
				.width(iced::Length::Fill)
				.height(iced::Length::FillPortion(6))
				;
			let back_button = iced::widget::Button::new(iced::widget::Text::new("Back"))
				.on_press(Message::ExitSettings)
				.width(iced::Length::FillPortion(1))
				;
			let back_button = iced::widget::Row::new()
				.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
				.push(back_button)
				.push(iced::widget::Space::with_width(iced::Length::FillPortion(14)))
				;
			let back_button = iced::widget::Container::new(back_button)
				.width(iced::Length::Fill)
				.height(iced::Length::FillPortion(1))
				;
			return iced::widget::Column::new()
				.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
				.push(back_button)
				.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
				.push(page)
				.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
				.into()
		}
		let remaining_days = (seasons::SEASON_LENGTH as i64) + ((*seasons::CURRENT_SEASON_START) - chrono::Utc::now()).num_days();
		let remaining_days = format!("{} days remaining in Season {}", remaining_days, (*seasons::CURRENT_SEASON_NUMBER));
		let remaining_days = iced::widget::Text::new(remaining_days).size(48);
		let remaining_days = iced::widget::Container::new(remaining_days)
			.width(iced::Length::Fill)
			.center_x()
			.center_y()
			;

		let battle_pass_level_display = format!("Battle pass level {}", self.battle_pass_level);
		let battle_pass_level_display = iced::widget::Text::new(battle_pass_level_display)
			.horizontal_alignment(iced::alignment::Horizontal::Center)
			.vertical_alignment(iced::alignment::Vertical::Center)
			.size(32)
			;
		let battle_pass_level_display = iced::widget::Container::new(battle_pass_level_display)
			.width(iced::Length::FillPortion(2))
			.center_x()
			.center_y()
			;
		let battle_pass_slider = iced::widget::Slider::new(0..=self.battle_pass_target, self.battle_pass_level, Message::UpdateBattlePassLevel)
			.width(iced::Length::Fill)
			;
		let battle_pass_slider = iced::widget::Container::new(battle_pass_slider)
			.width(iced::Length::FillPortion(4))
			.padding(8)
			.center_x()
			.center_y()
			;
		let battle_pass_display_and_slider = iced::widget::Row::new()
			.push(battle_pass_level_display)
			.push(battle_pass_slider)
			.padding(16)
			;
		let levels_required_per_day = seasons::get_levels_required_per_day(self.battle_pass_level, self.battle_pass_target);
		let battle_pass_target_information = if levels_required_per_day == 0f64
		{
			String::from("You have reached your target! Congratulations!")
		}
		else
		{
			let target = match self.battle_pass_target
			{
				seasons::MYTHIC_BATTLE_PASS_END => "Mythic Skin".to_string(),
				seasons::PRESTIGE_BATTLE_PASS_END => "All Prestige Titles".to_string(),
				level => format!("level {}", level)
			};
			format!("{:.3} ({}) levels per day needed to unlock {}", levels_required_per_day, levels_required_per_day.ceil(), target)
		};
		let battle_pass_target_information = iced::widget::Text::new(battle_pass_target_information).size(32);
		let battle_pass_target_information = iced::widget::Container::new(battle_pass_target_information)
			.width(iced::Length::Fill)
			.center_x()
			.center_y()
			;

		let roll_mastery_display = iced::widget::Text::new("Roll Mastery Tracker")
			.horizontal_alignment(iced::alignment::Horizontal::Center)
			.vertical_alignment(iced::alignment::Vertical::Center)
			.size(48)
			.width(iced::Length::Fill)
			.height(iced::Length::FillPortion(1))
			;
		let tank_win_section = roll_win_counter(Roll::Tank, self.tank_wins);
		let damage_win_section = roll_win_counter(Roll::Damage, self.damage_wins);
		let support_win_section = roll_win_counter(Roll::Support, self.support_wins);

		let reset_win_section = iced::widget::Text::new("Reset wins")
			.horizontal_alignment(iced::alignment::Horizontal::Center)
			.size(32)
			;
		let reset_win_section = iced::widget::Button::new(reset_win_section)
			.on_press(Message::ResetWins)
			.style(iced::theme::Button::Destructive)
			.width(iced::Length::FillPortion(3))
			;
		let reset_win_section = iced::widget::Row::new()
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(2)))
			.push(reset_win_section)
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(2)))
			;

		let settings_button = iced::widget::Button::new(iced::widget::Text::new("Settings"))
			.on_press(Message::OpenSettings)
			.width(iced::Length::FillPortion(1))
			;
		let settings_button = iced::widget::Row::new()
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(7)))
			.push(settings_button)
			.push(iced::widget::Space::with_width(iced::Length::FillPortion(1)))
			;

		iced::widget::Column::new()
			.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
			.push(remaining_days)
			.push(battle_pass_display_and_slider)
			.push(battle_pass_target_information)
			.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
			.push(roll_mastery_display)
			.push(tank_win_section)
			.push(damage_win_section)
			.push(support_win_section)
			.push(reset_win_section)
			.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
			.push(settings_button)
			.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
			.into()
	}

	fn theme(&self) -> Self::Theme
	{
		self.theme.clone()
	}
}

#[derive(Debug, Clone)]
pub enum Roll
{
	Tank,
	Damage,
	Support,
}

fn roll_win_counter<'a>(roll: Roll, wins: u8) -> iced::Element<'a, Message>
{
	let size = 48;
	let roll_text = match roll
	{
		Roll::Tank => "Tank",
		Roll::Damage => "Damage",
		Roll::Support => "Support",
	};
	let roll_text = iced::widget::Text::new(roll_text)
		.width(iced::Length::Fill)
		.horizontal_alignment(iced::alignment::Horizontal::Center)
		.size(size);
	let roll_text = iced::widget::Container::new(roll_text)
		.center_x()
		.center_y()
		.width(iced::Length::FillPortion(3))
		;
	let plus =
	{
		let plus = iced::widget::Button::new(iced::widget::Text::new("+")
			.size(size))
			.width(iced::Length::Fill)
			;
		let plus = match roll
		{
			Roll::Tank if wins < 3 => plus.on_press(Message::UpdateTankWins(1)),
			Roll::Damage if wins < 3 => plus.on_press(Message::UpdateDamageWins(1)),
			Roll::Support if wins < 3 => plus.on_press(Message::UpdateSupportWins(1)),
			_ => plus.style(iced::theme::Button::Secondary), // Don't allow more than 3 wins and make it look disabled
		};
		plus
	};
	let plus = iced::widget::Container::new(plus)
		.center_x()
		.center_y()
		.padding(8)
		.width(iced::Length::FillPortion(1))
		;
	let minus =
	{
		let minus = iced::widget::Button::new(iced::widget::Text::new("-")
			.size(size))
			.width(iced::Length::Fill)
			;
		let minus = match roll
		{
			Roll::Tank if wins > 0 => minus.on_press(Message::UpdateTankWins(-1)),
			Roll::Damage if wins > 0 => minus.on_press(Message::UpdateDamageWins(-1)),
			Roll::Support if wins > 0 => minus.on_press(Message::UpdateSupportWins(-1)),
			_ => minus.style(iced::theme::Button::Secondary), // Don't allow more than 3 wins and make it look disabled
		};
		minus
	};
	let minus = iced::widget::Container::new(minus)
		.center_x()
		.center_y()
		.padding(8)
		.width(iced::Length::FillPortion(1))
		;
	let padding = iced::widget::Space::new(iced::Length::FillPortion(1), iced::Length::Units(1));
	let wins_text = ["|||", "| |", " | ", ""][3 - wins as usize];
	let wins_text = iced::widget::Text::new(wins_text)
		.size(size)
		.width(iced::Length::Fill)
		;
	let wins_text = iced::widget::Container::new(wins_text)
		.center_x()
		.center_y()
		.width(iced::Length::FillPortion(2))
		// .padding(32)
		;


	iced::widget::Row::new()
		.push(roll_text)
		.push(plus)
		.push(minus)
		.push(padding)
		.push(wins_text)
		.into()
}
