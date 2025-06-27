use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Stylize, Color},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget},
    DefaultTerminal, Frame,

};
use ratatui::style::Style;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
struct Dot {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone)]
struct Food {
    x: u16,
    y: u16,
}

#[derive(Debug)]
pub struct App {
    counter: u8,
    exit: bool,
    dot: Dot,
    last_update: Instant,
    move_right: bool,
    move_left: bool,
    move_up: bool,
    move_down: bool,
    tail: VecDeque<Dot>,
    tail_length: u16,
    food: Food,
    show_game_over_popup: bool,
    show_win_popup: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            counter: 0,
            exit: false,
            dot: Dot { x: 20, y: 20 },
            food: Food { x: 50, y: 20 },
            last_update: Instant::now(),
            move_right: false,
            move_left: false,
            move_up: true,
            move_down: false,
            tail: VecDeque::new(),
            tail_length: 3,
            show_game_over_popup: false,
            show_win_popup: false,
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

const GAME_WIDTH: u16 = 60;
const GAME_HEIGHT: u16 = 25;
const GRID_SIZE: u16 = GAME_WIDTH * GAME_HEIGHT;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.spawn_food_randomly();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if !self.show_game_over_popup && !self.show_win_popup{
                self.update()?;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        
        if self.show_game_over_popup {
            self.render_game_over_popup(frame);
        }else if self.show_win_popup {
            self.render_win_popup(frame);
        }
    }

    fn render_game_over_popup(&self, frame: &mut Frame) {
        // Calculate popup size and position (centered)
        let popup_area = centered_rect(40, 20, frame.area());
        
        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);
        
        let popup_text = vec![
            Line::from(""),
            Line::from("Game over!".bold().yellow()),
            Line::from(""),
            Line::from(vec![
                "You scored: ".bold(),
                self.counter.to_string().blue().bold(),
            ])
        ];
        
        let popup_block = Block::bordered()
            .title(" Popup ".bold())
            .border_set(border::ROUNDED)
            .style(Style::default().bg(Color::DarkGray));
        
        let popup_paragraph = Paragraph::new(Text::from(popup_text))
            .block(popup_block)
            .alignment(Alignment::Center);
        
        frame.render_widget(popup_paragraph, popup_area);
    }

    fn render_win_popup(&self, frame: &mut Frame) {
        // Calculate popup size and position (centered)
        let popup_area = centered_rect(40, 20, frame.area());

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        let popup_text = vec![
            Line::from(""),
            Line::from("You won!".bold().yellow()),
            Line::from(""),
            Line::from(vec![
                "You scored: ".bold(),
                self.counter.to_string().blue().bold(),
            ])
        ];

        let popup_block = Block::bordered()
            .title(" Popup ".bold())
            .border_set(border::ROUNDED)
            .style(Style::default().bg(Color::DarkGray));

        let popup_paragraph = Paragraph::new(Text::from(popup_text))
            .block(popup_block)
            .alignment(Alignment::Center);

        frame.render_widget(popup_paragraph, popup_area);
    }


    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.show_game_over_popup {
            self.exit();
        }
        
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            _ => {}
        }
    }

    fn update(&mut self) -> io::Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(150) {
            self.handle_death();
            self.handle_tail();
            self.move_dot();
            self.last_update = now;
        }
        Ok(())
    }

    fn move_dot(&mut self) {
        
        let game_width: u16 = GAME_WIDTH;
        let game_height: u16 = GAME_HEIGHT;
        let max_x = game_width.saturating_sub(3);
        let max_y = game_height.saturating_sub(3); 
        
        if self.move_up && self.dot.y > 0 {
            self.handle_food();
            self.dot.y -= 1;

        }
        if self.move_right && self.dot.x < max_x {
            self.handle_food();
            self.dot.x += 1;
            if self.dot.x < max_x {
                self.handle_food();
                self.dot.x += 1;
            }
        }
        if self.move_left && self.dot.x > 0 {
            self.handle_food();
            self.dot.x -= 1;
            if self.dot.x > 0 {
                self.handle_food();
                self.dot.x -= 1;
            }
        }
        if self.move_down && self.dot.y < max_y {
            self.handle_food();
            self.dot.y += 1;

        }
    }
    
    fn handle_tail(&mut self) {
        self.tail.push_front(self.dot.clone());

        if self.tail_length < self.tail.len() as u16 {
            self.tail.pop_back();
        }
        
    }

    fn handle_food(&mut self){
        if self.dot.x == self.food.x && self.dot.y == self.food.y {
            self.tail_length = self.tail_length + 1;

            self.spawn_food_randomly();
            self.counter = self.counter + 1;
        }
    }

fn spawn_food_randomly(&mut self) {
    if self.tail_length == (GRID_SIZE - 1) {
        self.show_win_popup = true;
    }
    
    let mut rng = rand::thread_rng();
    let game_width: u16 = GAME_WIDTH;
    let game_height: u16 = GAME_HEIGHT;
    let max_x = game_width.saturating_sub(3);
    let max_y = game_height.saturating_sub(3);

    loop {
        let mut x = rng.gen_range(0..=max_x);
        let y = rng.gen_range(0..=max_y);

        // Ensure x is even (since horizontal movement is by 2)
        if x % 2 != 0 {
            x = if x == max_x { x - 1 } else { x + 1 };
        }

        // Check if the generated position conflicts with the head
        if x == self.dot.x && y == self.dot.y {
            continue;
        }

        // Check if the generated position conflicts with any tail segment
        let conflicts_with_tail = self.tail.iter().any(|tail_dot| {
            tail_dot.x == x && tail_dot.y == y
        });

        if conflicts_with_tail {
            continue;
        }

        // If we reach here, the position is valid
        self.food = Food { x, y };
        break;
    }
}

    fn handle_death(&mut self) {
        if self.tail.contains(&self.dot) {
           self.show_game_over_popup = true;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_up(&mut self) {
        if !self.move_down {
            self.move_right = false;
            self.move_left = false;
            self.move_up = true;
            self.move_down = false;
        }
    }
    
    fn move_down(&mut self) {
        if !self.move_up {
            self.move_right = false;
            self.move_left = false;
            self.move_up = false;
            self.move_down = true;
        }
    }

    fn move_right(&mut self) {
        if !self.move_left {
            self.move_right = true;
            self.move_left = false;
            self.move_up = false;
            self.move_down = false;
        }
    }

    fn move_left(&mut self) {
        if !self.move_right {
            self.move_left = true;
            self.move_right = false;
            self.move_up = false;
            self.move_down = false;
        }
    }
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

    let game_width = GAME_WIDTH.min(area.width);
    let game_height = GAME_HEIGHT.min(area.height);
    
    let x = area.x + (area.width.saturating_sub(game_width)) / 2;
    let y = area.y + (area.height.saturating_sub(game_height)) / 2;
    
    let game_area = Rect {
        x,
        y,
        width: game_width,
        height: game_height,
    };

    let title = Line::from(vec![
        " Snake - Score: ".bold(),
        self.counter.to_string().yellow().bold(),
        " ".into(),
    ]);
    
    let instructions = Line::from(vec![
        " Move ".into(),
        " <Left> ".blue().bold(),
        " <Right> ".blue().bold(),
        " <Up> ".blue().bold(),
        " <Down> ".blue().bold(),
        " - ".bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let mut content = vec![];

    for y in 0..game_area.height {
        let mut line_chars: Vec<char> = " ".repeat(game_area.width.saturating_sub(2) as usize).chars().collect();

        for tail_dot in &self.tail {
            if y == tail_dot.y {
                line_chars[tail_dot.x as usize] = '○';
            }
        }
        if y == self.dot.y {
            if (self.dot.x as usize) < line_chars.len() {
                line_chars[self.dot.x as usize] = '●';
            }
        }

        if y == self.food.y {
            line_chars[self.food.x as usize] = '■';
        }

        content.push(Line::from(String::from_iter(line_chars).red().bold()));
    }

    let display_text = Text::from(content);

    Paragraph::new(display_text)
        .block(block)
        .render(game_area, buf);
}
}