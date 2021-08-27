use std::io::{self, Read};

pub const BULLET_ID_LEN: usize = 8;

#[derive(Clone, Debug)]
pub enum Packet {
    GameStart { width: u8, height: u8, pids: Vec<u8> }, // sent when starting game | symbol @

    PlayerCreate(u8), // new player | symbol +
    PlayerDestroy(u8), // player leaves | symbol -

    PlayerEvent { pid: u8, event: Event }, // { player id, event } | symbol $
    BulletPos { id: String, x: u8, y: u8 }, // { bullet id, x, y } | symbol *
    BulletDestroy(String), // symbol ^

    GameWon,
    GameLost,
}

impl Packet {
    pub fn parse(self) -> Vec<u8> {
        match self {
            Self::GameStart { width, height, mut pids } => {
                let mut raw = vec![b'@', width, height, pids.len() as u8];
                raw.append(&mut pids);
                raw
            },
            Self::PlayerCreate(pid) => {
                vec![b'+', pid]
            },
            Self::PlayerDestroy(pid) => {
                vec![b'-', pid]
            },
            Self::PlayerEvent { pid, event } => {
                vec![b'$', pid, event.parse()]
            },
            Self::BulletPos { id, x, y } => {
                let mut raw = vec![b'*']; 
                raw.append(&mut id.into_bytes());
                raw.push(x);
                raw.push(y);
                raw
            },
            Self::BulletDestroy(id) => {
                let mut raw = vec![b'^'];
                raw.append(&mut id.into_bytes());
                raw
            },
            Self::GameWon => {
                vec![b'w']
            },
            Self::GameLost => {
                vec![b'l']
            }
        }
    }

    pub fn read_from(stream: &mut impl Read) -> Result<Option<Self>, io::Error> {
        let first_byte = Self::read_byte(stream)?;
        match first_byte {
            b'@' => {
                let width = Self::read_byte(stream)?;
                let height = Self::read_byte(stream)?;
                let len = Self::read_byte(stream)?;
                let pids = Self::read_length(stream, len as usize)?;
                Ok(Some(Self::GameStart{width, height, pids}))
            },
            b'+' => {
                let pid = Self::read_byte(stream)?;
                Ok(Some(Self::PlayerCreate(pid)))
            },
            b'-' => {
                let pid = Self::read_byte(stream)?;
                Ok(Some(Self::PlayerDestroy(pid)))
            },
            b'$' => {
                let pid = Self::read_byte(stream)?;
                let event = Event::get(Self::read_byte(stream)?).unwrap();
                Ok(Some(Self::PlayerEvent{pid, event}))
            },
            b'*' => {
                let id = Self::read_length(stream, BULLET_ID_LEN)?;
                let x = Self::read_byte(stream)?;
                let y = Self::read_byte(stream)?;
                Ok(Some(Self::BulletPos{id: String::from_utf8_lossy(&id).to_string(), x, y}))
            },
            b'^' => {
                let id = Self::read_length(stream, BULLET_ID_LEN)?;
                Ok(Some(Self::BulletDestroy(String::from_utf8_lossy(&id).to_string())))
            },
            b'w' => {
                Ok(Some(Self::GameWon))
            },
            b'l' => {
                Ok(Some(Self::GameLost))
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid first byte")),
        }
    }

    fn read_byte(stream: &mut impl Read) -> Result<u8, io::Error> {
        let mut byte = [0u8];
        stream.read_exact(&mut byte)?;
        Ok(byte[0])
    }
    fn read_length(stream: &mut impl Read, len: usize) -> Result<Vec<u8>, io::Error> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(Self::read_byte(stream)?);
        }
        Ok(vec)
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    MoveRight,
    MoveLeft,
    Fire,
    Exit,
}

impl Event {
    pub fn parse(&self) -> u8 {
        match self {
            Self::MoveRight => 0,
            Self::MoveLeft  => 1,
            Self::Fire      => 2,
            Self::Exit      => 3,
        }
    }

    pub fn get(value: u8) -> Result<Self, &'static str> {
        match value {
            0 => Ok(Self::MoveRight),
            1 => Ok(Self::MoveLeft),
            2 => Ok(Self::Fire),
            3 => Ok(Self::Exit),
            _ => Err("invalid event"),
        }
    }
}