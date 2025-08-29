// the admin panel. this is a lil ratatui app that, when the server is built with the admin_panel feature, provides
// statistics, logging, world control, etc.

// it has a definite performance cost. don't enable admin_panel for production servers.
use crate::components::*;
use crate::resources::*;
use bevy::ecs::entity::Entities;
use bevy::ecs::system::{SystemId, SystemState};
use bevy::prelude::*;
use common::comms::Stage;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::time::Duration;

struct AdminWidget {
    config_filename: String,
    connected_clients: usize,
    players: usize,
    max_players: usize,
    tick: u16,
    time_in_stage: u16,
    stage_name: String,
    all_entities: u32,
    all_bullets: usize,
    all_sensors: usize,
    all_pieces: usize,
}

impl AdminWidget {
    fn build(
        config: Res<Config>,
        state: Res<GameState>,
        name: Res<ConfigFileName>,
        clients: Query<&Client>,
        playing: Query<&ClientPlaying>,
        entity_data: &Entities,
        bullets: Query<&Bullet>,
        sensors: Query<&FieldSensor>,
        pieces: Query<&GamePiece>,
    ) -> Self {
        let name: &ConfigFileName = &name;
        Self {
            config_filename: match name {
                ConfigFileName(Some(s)) => s.clone(),
                ConfigFileName(None) => "No Config File Loaded".to_string(),
            },
            connected_clients: clients.iter().len(),
            players: playing.iter().len(),
            max_players: config.counts.max_players as usize,
            stage_name: match state.get_state_enum() {
                Stage::MoveShips => "MOVE SHIPS",
                Stage::Playing => "PLAYING",
                Stage::Waiting => "WAITING",
            }
            .to_string(),
            time_in_stage: state.time_in_stage,
            tick: state.tick,
            all_entities: entity_data.len(),
            all_bullets: bullets.iter().len(),
            all_sensors: sensors.iter().len(),
            all_pieces: pieces.iter().len(),
        }
    }
}

#[derive(Resource)]
struct AdminPanel {
    terminal: DefaultTerminal,
    exit: bool,
}

impl AdminPanel {
    fn new() -> Self {
        Self {
            terminal: ratatui::init(),
            exit: false,
        }
    }

    fn tick(&mut self, widget: AdminWidget) -> bool {
        self.terminal
            .draw(|frame| Self::draw(frame, widget))
            .unwrap();
        loop {
            if !event::poll(Duration::from_millis(3)).unwrap() {
                break;
            }
            match event::read().unwrap() {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            self.exit = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.exit
    }

    fn draw(frame: &mut Frame, widget: AdminWidget) {
        frame.render_widget(&widget, frame.area());
    }
}

impl Widget for &AdminWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [title, stats, _] = Layout::vertical([Length(1), Max(9), Min(0)]).areas(area);
        let [quick_stats, _] = Layout::horizontal([Max(30), Fill(1)]).areas(stats);
        Line::from(vec![
            "Exosphere Admin Panel  ".bold(),
            (&self.config_filename).into(),
            "  [press Q to exit]".into(),
        ])
        .centered()
        .render(title, buf);
        let stats_bar = Block::default().borders(Borders::ALL).title("Quick Stats");
        Paragraph::new(vec![
            Line::from(vec![
                "Connected Clients: ".into(),
                self.connected_clients.to_string().bold(),
            ]),
            Line::from(vec![
                "Players: ".into(),
                self.players.to_string().bold(),
                "/".into(),
                self.max_players.to_string().into(),
            ]),
            Line::from(vec![
                (&self.stage_name).into(),
                " ".into(),
                self.tick.to_string().bold(),
                "/".into(),
                self.time_in_stage.to_string().into(),
            ]),
            Line::from(vec![
                "Entity count: ".into(),
                self.all_entities.to_string().bold(),
            ]),
            Line::from(vec![
                "Sensor count: ".into(),
                self.all_sensors.to_string().bold(),
            ]),
            Line::from(vec![
                "Bullet count: ".into(),
                self.all_bullets.to_string().bold(),
            ]),
            Line::from(vec![
                "Piece count: ".into(),
                self.all_pieces.to_string().bold(),
            ]),
        ])
        .block(stats_bar)
        .render(quick_stats, buf);
    }
}

impl Drop for AdminPanel {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

#[derive(Resource)]
struct AdminWidgetBuilderSystem(SystemId<(), AdminWidget>);

pub fn update_admin_panel(world: &mut World) {
    if let None = world.get_resource_mut::<AdminPanel>() {
        world.insert_resource(AdminPanel::new());
    }
    if let None = world.get_resource::<AdminWidgetBuilderSystem>() {
        let builder_system = world.register_system(AdminWidget::build);
        world.insert_resource(AdminWidgetBuilderSystem(builder_system));
    }
    let AdminWidgetBuilderSystem(builder) =
        world.get_resource::<AdminWidgetBuilderSystem>().unwrap();
    let widget = world.run_system::<AdminWidget>(*builder).unwrap();
    let mut admin = world.get_resource_mut::<AdminPanel>().unwrap();
    if admin.tick(widget) {
        world.send_event(AppExit::Success);
    }
}
