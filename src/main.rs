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
struct App<R> {
    data: VecDeque<Vec<f64>>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    source: BufReader<R>
}

impl<R: Read> App<R> {
    fn new(source: BufReader<R>) -> App<R> {
        App {
            data: VecDeque::with_capacity(200),
            terminal: Terminal::new(CrosstermBackend::new(io::stdout())).unwrap(),
            source
        }
    }

    fn update(&mut self) -> std::io::Result<()> {
        let mut line = String::new();
        let len = self.source.read_line(&mut line)?;

        if len > 0 {
            let values: Vec<f64> = line.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap())
                .collect();

            if self.data.len() == 200 { 
                self.data.pop_front();
            }
            self.data.push_back(values);
            self.draw();
        }
        Ok(())
    }

    fn draw(&mut self) {
        let points: Vec<(f64, f64)> = self.data.iter()
            .enumerate()
            .map(|(i, data)| (i as f64, data[0] as f64))
            .collect();

        let datasets = vec![
            Dataset::default()
                .name("Some dataset")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(&points),
        ];

        self.terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(100),
                    ].as_ref()
                )
                .split(f.size()); 

            let chart = Chart::new(datasets)
                .block(Block::default().title("Chart"))
                .x_axis(Axis::default()
                    .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([-100.0, 100.0])
                    .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
                .y_axis(Axis::default()
                    .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([-100.0, 100.0])
                    .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()));            

            f.render_widget(chart, chunks[0]);
        });
    }
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000")?;
    let mut app: App<TcpStream> = App::new(BufReader::new(stream));

    loop {
        app.update();
    }

    Ok(())
}
