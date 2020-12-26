use anyhow::{anyhow, Result};
use std::io::prelude::*;
use std::net::TcpStream;
use std::collections::VecDeque;
use std::io::BufReader;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    symbols,
    text::Span,
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct App<R> {
    data: Vec<Vec<(f64, f64)>>,
    index: u32,
    num_readings: u32,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    source: BufReader<R>
}

impl<R: Read> App<R> {
    fn new(source: BufReader<R>) -> App<R> {
        let num_readings = 200;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        App {
            index: 0,
            data: Vec::new(),
            terminal: Terminal::new(CrosstermBackend::new(stdout)).unwrap(),
            num_readings,
            source
        }
    }

    fn init(&mut self) -> Result<()> {
        for _ in self.read_stream()? {
            self.data.push(Vec::new());
        }
        Ok(())
    }

    fn read_stream(&mut self) -> Result<Vec<f64>> {
        let mut line = String::new();
        let len = self.source.read_line(&mut line)?;

        if len > 0 {
            let values: Vec<f64> = line.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap())
                .collect();
            return Ok(values)
        }
        Err(anyhow!("Can't read!"))
    }

    fn update(&mut self) -> Result<()> {
        let mut line = String::new();
        let len = self.source.read_line(&mut line)?;

        match self.read_stream() {
            Ok(values) => {
                for (i, val) in values.iter().enumerate() {
                    if self.data[i].len() == self.num_readings as usize { 
                        self.data[i].remove(0);
                    }
                    self.data[i].push((self.index as f64, *val));
                    self.index += 1;
                }
                self.draw();
            },
            Err(_) => {}
        };
        Ok(()) 
    }

    fn draw(&mut self) {
        let datasets: Vec<Dataset> = self.data.iter().map(|dataset| {
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(&dataset)
        }).collect();

        let bounds = if self.index < self.num_readings {
            [0.0, 200.0]
        } else {
            [(self.index - self.num_readings) as f64, self.index as f64]
        };

        self.terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints(
                    [
                        Constraint::Percentage(100),
                    ] .as_ref()
                )
                .split(f.size()); 

            let chart = Chart::new(datasets)
                .block(Block::default().title("Chart"))
                .x_axis(Axis::default()
                    .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds(bounds)
                    .labels(["0.0", "100.0", "200.0"].iter().cloned().map(Span::from).collect()))
                .y_axis(Axis::default()
                    .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([-100.0, 100.0])
                    .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()));            

            f.render_widget(chart, chunks[0]);
        });
    }
}

fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000")?;
    let mut app: App<TcpStream> = App::new(BufReader::new(stream));

    app.init();
    loop {
        match app.update() {
            Err(val) => println!("OMG"),
            _ => {}
        }
    }

    Ok(())
}
