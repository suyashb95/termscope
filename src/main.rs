use std::io::prelude::*;
use std::net::TcpStream;
use std::collections::VecDeque;
use std::io::BufReader;
struct App<R> {
    data: VecDeque<Vec<f64>>,
    source: BufReader<R>,
}

impl<R: Read> App<R> {
    fn new(source: BufReader<R>) -> App<R> {
        App {
            data: VecDeque::with_capacity(200),
            source
        }
    }

    fn update(&mut self) -> std::io::Result<()> {
        let mut line = String::new();

        println!("waiting to read line from stream");
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
            println!("data from stream: {:?}", values);
            self.data.push_back(values);
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:5000")?;
    let mut app = App::new(BufReader::new(stream));

    loop {
        app.update();
    }

    Ok(())
}
