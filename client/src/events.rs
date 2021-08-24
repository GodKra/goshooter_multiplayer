use std::thread;

use crossbeam::channel::{self, SendError};
use crossterm::event::{self, KeyCode};

use common::Event;
pub struct EventHandler {
    sender: channel::Sender<Event>,
}

impl EventHandler {
    pub fn handle_events(sender: channel::Sender<Event>) {
        let handler = EventHandler { sender };
        thread::spawn(move || -> Result<(), SendError<Event>> {
            loop {
                if let Ok(event::Event::Key(key)) =  event::read() {
                    match key.code {
                        KeyCode::Right | KeyCode::Char('d') => 
                            handler.sender.send(Event::MoveRight)?,
                        KeyCode::Left | KeyCode::Char('a') => 
                            handler.sender.send(Event::MoveLeft)?,
                        KeyCode::Char(' ') | KeyCode::Char('w') | KeyCode::Up => 
                            handler.sender.send(Event::Fire)?,
                        KeyCode::Esc => {
                            handler.sender.send(Event::Exit)?;
                            return Ok(());
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}
