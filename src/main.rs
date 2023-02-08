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
			.push(settings_button)
			.push(iced::widget::Space::with_height(iced::Length::FillPortion(1)))
			.into()
	}

	fn theme(&self) -> Self::Theme
	{
		self.theme.clone()
	}
}

