use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::canvas::Canvas,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up, Down, Left, Right,
}

#[derive(Copy, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

enum GameState {
    Waiting,
    Playing,
    GameOver,
}

struct App {
    snake: Vec<Point>,
    food: Point,
    direction: Direction,
    score: u32,
    state: GameState,
}

impl App {
    fn new() -> App {
        App {
            snake: vec![Point { x: 10.0, y: 10.0 }],
            food: Point { x: 5.0, y: 5.0 },
            direction: Direction::Right,
            score: 0,
            state: GameState::Waiting,
        }
    }

    fn update(&mut self) {
        if !matches!(self.state, GameState::Playing) { return; }

        let head = self.snake[0];
        let mut new_head = head;

        match self.direction {
            Direction::Up => new_head.y += 1.0,
            Direction::Down => new_head.y -= 1.0,
            Direction::Left => new_head.x -= 1.0,
            Direction::Right => new_head.x += 1.0,
        }

        // Çarpışma Kontrolü
        if new_head.x < 0.0 || new_head.x >= 20.0 || new_head.y < 0.0 || new_head.y >= 20.0 || self.snake.contains(&new_head) {
            self.state = GameState::GameOver;
            return;
        }

        self.snake.insert(0, new_head);

        // Yemek yeme
        if (new_head.x - self.food.x).abs() < 0.1 && (new_head.y - self.food.y).abs() < 0.1 {
            self.score += 10;
            self.spawn_food();
        } else {
            self.snake.pop();
        }
    }

    fn spawn_food(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.food = Point {
            x: rng.gen_range(0..20) as f64,
            y: rng.gen_range(0..20) as f64,
        };
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res { println!("Hata: {:?}", err); }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(200); // YILAN YAVAŞ GİTSİN (200ms)

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Herhangi bir tuşla oyunu başlat
                if matches!(app.state, GameState::Waiting) {
                    app.state = GameState::Playing;
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    // WASD Kontrolleri
                    KeyCode::Char('w') if app.direction != Direction::Down => app.direction = Direction::Up,
                    KeyCode::Char('s') if app.direction != Direction::Up => app.direction = Direction::Down,
                    KeyCode::Char('a') if app.direction != Direction::Right => app.direction = Direction::Left,
                    KeyCode::Char('d') if app.direction != Direction::Left => app.direction = Direction::Right,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }

        if matches!(app.state, GameState::GameOver) {
            // Oyun bitince 2 saniye bekle ve kapat (veya istersen 'q' bekle)
            std::thread::sleep(Duration::from_secs(2));
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.size();
    
    let title = match app.state {
        GameState::Waiting => " [ HERHANGİ BİR TUŞA BAS VE BAŞLA! ] ",
        GameState::Playing => " [ WASD İLE OYNA | 'Q' ÇIKIŞ ] ",
        GameState::GameOver => " [ OYUN BİTTİ! ] ",
    };

    let canvas = Canvas::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                format!("{} Skor: {} ", title, app.score),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            )))
        .paint(|ctx| {
            if matches!(app.state, GameState::Waiting) {
                ctx.print(5.0, 10.0, Span::styled("HAZIR MISIN?", Style::default().fg(Color::White)));
            } else {
                ctx.print(app.food.x, app.food.y, Span::raw("🍎"));
                for (i, cell) in app.snake.iter().enumerate() {
                    let symbol = if i == 0 { "🟢" } else { "🟩" };
                    ctx.print(cell.x, cell.y, Span::raw(symbol));
                }
            }
        })
        .x_bounds([0.0, 20.0])
        .y_bounds([0.0, 20.0]);

    f.render_widget(canvas, area);
}