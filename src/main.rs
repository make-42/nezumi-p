mod api;
mod config;
mod consts;

use api::CollectedData;
use chrono::{DateTime, Local, Utc};
use config::Config;
use std::{cmp, io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use humanize_duration::{prelude::DurationExt, Truncate};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge, List, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::init();
    let mut terminal = ratatui::init();
    let mut binding = App::default();
    binding.collected_data = api::get_departures(cfg.clone())
        .await
        .expect("Couldn't get data from");
    binding.config = cfg;
    let app_result = binding.run(&mut terminal);
    ratatui::restore();
    Ok(app_result?)
}

#[derive(Debug, Default)]
pub struct App {
    config: Config,
    collected_data: CollectedData,
    stations_state: ListState,
    selected_column: bool,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.stations_state.select_first();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(1000))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        use Constraint::{Fill, Length, Min};
        let default_margin = Margin::new(2, 2);
        let thin_margin = Margin::new(1, 1);
        let timetable_margin_end = Margin::new(1, 0);

        let vertical = Layout::vertical([Length(1), Min(0), Length(5)]);
        let [title_area, main_area, status_area] = vertical.areas(frame.area());
        let horizontal = Layout::horizontal([Fill(1); 2]);
        let [stations_area, timetable_area] = horizontal.areas(main_area);

        let mut station_names = vec![];
        for station in &self.config.stations {
            station_names.push(station.name.clone());
        }
        let station_list = List::new(station_names)
            //.block(Block::bordered().title("List"))
            .style(Style::new().white())
            .highlight_style(Style::default().bold().italic().fg(Color::Indexed(1)))
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);

        let (_, terminal_height) = crossterm::terminal::size().unwrap();

        let mut constraints = vec![];
        for (i, _) in self.collected_data.departure_data_list
            [self.stations_state.selected().unwrap()]
        .siri
        .service_delivery
        .stop_monitoring_delivery[0]
            .monitored_stop_visit
            .iter()
            .enumerate()
        {
            if i == 0 {
                constraints.push(Constraint::Length(7));
            } else if i
                > (cmp::max(i32::from(terminal_height / 6) - 3, 0))
                    .try_into()
                    .unwrap()
            {
            } else {
                constraints.push(Constraint::Length(6));
            }
        }

        let departure_vertical = Layout::vertical(constraints);
        let departure_vertical_areas = departure_vertical.split(timetable_area);
        for (i, departure_vertical_area) in departure_vertical_areas.iter().enumerate() {
            let calc_inner_rect = if i == 0 {
                Rect::new(
                    departure_vertical_area.x + 1,
                    departure_vertical_area.y + 1,
                    cmp::max(i32::from(departure_vertical_area.width) - 2, 0)
                        .try_into()
                        .unwrap(),
                    cmp::max(i32::from(departure_vertical_area.height) - 1, 0)
                        .try_into()
                        .unwrap(),
                )
            } else {
                departure_vertical_area.inner(timetable_margin_end)
            };
            let departure_time_datetime = Local::now()
                .checked_add_signed(
                    DateTime::parse_from_rfc3339(
                        self.collected_data.departure_data_list
                            [self.stations_state.selected().unwrap()]
                        .siri
                        .service_delivery
                        .stop_monitoring_delivery[0]
                            .monitored_stop_visit[i]
                            .monitored_vehicle_journey
                            .monitored_call
                            .expected_departure_time
                            .clone()
                            .as_str(),
                    )
                    .unwrap()
                    .signed_duration_since(Utc::now()),
                )
                .unwrap();

            let time_till_departure = departure_time_datetime.signed_duration_since(Local::now());

            let departure_data = Block::bordered().title(
                Span::from(
                    self.collected_data.departure_data_list
                        [self.stations_state.selected().unwrap()]
                    .siri
                    .service_delivery
                    .stop_monitoring_delivery[0]
                        .monitored_stop_visit[0]
                        .monitored_vehicle_journey
                        .direction_name[0]
                        .value
                        .clone(),
                )
                .style(Style::default().bold().italic().fg(Color::Indexed(3))),
            );
            let departure_time = Line::styled(
                format!("{}", departure_time_datetime.format("%H:%M:%S")),
                Style::default().bold().italic(),
            );
            let departure_status = self.collected_data.departure_data_list
                [self.stations_state.selected().unwrap()]
            .siri
            .service_delivery
            .stop_monitoring_delivery[0]
                .monitored_stop_visit[0]
                .monitored_vehicle_journey
                .monitored_call
                .departure_status
                .as_str();
            let departure_time_relative = Line::styled(
                format!(
                    "{} {}",
                    time_till_departure.human(Truncate::Second).to_string(),
                    match departure_status {
                        "onTime" => "ON TIME".into(),
                        "delayed" => "DELAYED".into(),
                        _ => departure_status,
                    },
                ),
                Style::default().bold().italic(),
            )
            .alignment(Alignment::Right);

            let horizontal = Layout::horizontal([Length(14), Fill(1), Length(20)]);
            let vertical = Layout::vertical([Fill(1),Length(1)]);
            let [top_area,bottom_area] = vertical.areas(calc_inner_rect);
            let [left_area, middle_area, right_area] = horizontal.areas(top_area);

            let progress_bar = Gauge::default()
                .block(
                    Block::bordered().title(
                        Span::from("Distance")
                            .style(Style::default().bold().italic().fg(Color::Indexed(2))),
                    ),
                )
                .gauge_style(Style::new().white().on_black().italic())
                .percent(
                    cmp::max(cmp::min(time_till_departure.num_seconds() / 36, 100), 0)
                        .try_into()
                        .unwrap(),
                ); // Starts at 60min

                frame.render_widget(
                        Line::from(if self.collected_data.departure_data_list
                            [self.stations_state.selected().unwrap()]
                        .siri
                        .service_delivery
                        .stop_monitoring_delivery[0]
                            .monitored_stop_visit[0]
                            .monitored_vehicle_journey
                            .vehicle_feature_ref
                            .len()
                            != 0
                        {
                            "/˳˳_˳˳][˳˳_˳˳][˳˳_˳˳][˳˳_˳˳][˳˳_˳˳][˳˳_˳˳][˳˳_˳˳\\".to_string()
                        } else {
                            "/˳˳_˳˳][˳˳_˳˳][˳˳_˳˳][˳˳_˳˳\\".to_string()
                        })
                            .style(Style::default().bold().fg(Color::Indexed(1))).alignment(Alignment::Center),
                            Rect::new(
                                bottom_area.x+1,
                                bottom_area.y-1,
                                cmp::max(i32::from(bottom_area.width)-2, 0)
                                    .try_into()
                                    .unwrap(),
                                cmp::max(i32::from(bottom_area.height), 0)
                                    .try_into()
                                    .unwrap(),
                            ),
                );

            frame.render_widget(progress_bar, middle_area.inner(thin_margin));
            frame.render_widget(departure_time, left_area.inner(default_margin));
            frame.render_widget(departure_time_relative, right_area.inner(default_margin));
            frame.render_widget(departure_data, calc_inner_rect);
        }

        if self.collected_data.general_message_data_list[self.stations_state.selected().unwrap()]
            .siri
            .service_delivery
            .general_message_delivery[0]
            .info_message
            .len()
            != 0
        {
            let paragraph = Paragraph::new(
                self.collected_data.general_message_data_list
                    [self.stations_state.selected().unwrap()]
                .siri
                .service_delivery
                .general_message_delivery[0]
                    .info_message[0]
                    .info_channel_content
                    .message[0]
                    .message_text
                    .value
                    .clone(),
            )
            .style(Style::default().bold().italic())
            .block(Block::bordered().title(format!(
                    "Status: {}",
                    self.collected_data.general_message_data_list
                        [self.stations_state.selected().unwrap()]
                    .siri
                    .service_delivery
                    .general_message_delivery[0]
                        .info_message[0]
                        .info_channel_ref
                        .value
                )))
            .style(Style::default().bold().italic().fg(Color::Indexed(1)))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, status_area);
        } else {
            frame.render_widget(
                Block::bordered().title(
                    Span::from("Status")
                        .style(Style::default().bold().italic().fg(Color::Indexed(2))),
                ),
                status_area,
            );
        }

        frame.render_stateful_widget(
            station_list,
            stations_area.inner(default_margin),
            &mut self.stations_state,
        );
        frame.render_widget(
            Block::bordered().title(
                Span::from(format!("nezumi-p {}", consts::VERSION))
                    .style(Style::default().bold().italic().fg(Color::Indexed(1))),
            ),
            title_area,
        );
        frame.render_widget(
            Block::bordered().title(
                Span::from("Stops").style(Style::default().bold().italic().fg(Color::Indexed(2))),
            ),
            stations_area,
        );
        frame.render_widget(
            Block::bordered().title(
                Span::from("Timetable")
                    .style(Style::default().bold().italic().fg(Color::Indexed(2))),
            ),
            timetable_area,
        );
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.switch_column(),
            KeyCode::Right => self.switch_column(),
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn switch_column(&mut self) {
        self.selected_column = !self.selected_column;
    }
    fn scroll_up(&mut self) {
        self.stations_state.scroll_up_by(1);
    }
    fn scroll_down(&mut self) {
        if self.stations_state.selected().unwrap() < self.config.stations.len() - 1 {
            self.stations_state.scroll_down_by(1);
        }
    }
}
