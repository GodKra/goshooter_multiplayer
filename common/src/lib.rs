use std::{io::Read, marker::Unpin};
use byteorder::ReadBytesExt;
use tokio::io::{AsyncBufRead, AsyncReadExt};
use bytes::{BufMut, BytesMut};

pub const BULLET_ID_LEN: usize = 8;
pub const PLAYER_ID_MAX: usize = 8;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const GAME_INFO:      u8 = 0x01;
const PLAYER_JOIN:    u8 = 0x02;
const PLAYER_DESTROY: u8 = 0x03;
const PLAYER_EVENT:   u8 = 0x04;
const PLAYER_POS:     u8 = 0x05;
const BULLET_CREATE:  u8 = 0x06;
const BULLET_DESTROY: u8 = 0x07;
const ENEMY_CREATE:   u8 = 0x08;
const ENEMY_DESTROY:  u8 = 0x09;
const GAME_WON:       u8 = 0x0A;
const GAME_LOST:      u8 = 0x0B;

#[derive(Clone, Debug)]
pub enum Packet {
    GameInfo { width: u32, height: u32, pids: Vec<String> }, // sent when starting

    PlayerJoin(String),
    PlayerDestroy(String),

    PlayerEvent {pid: String, event: PlayerEvent },
    PlayerPos { pid: String, x: u32, y: u32 },

    BulletCreate { id: String, x: u32, y: u32 },
    BulletDestroy(String),

    EnemyCreate { id: String, x: u32, y: u32 },
    EnemyDestroy(String),

    GameWon,
    GameLost,
}

impl Packet {
    pub fn parse(self) -> Vec<u8> {
        match self {
            Self::GameInfo { width, height, pids } => {
                let mut raw = BytesMut::new();
                raw.put_u8(GAME_INFO);
                raw.put_u32(width);
                raw.put_u32(height);
                raw.put_u8(pids.len() as u8);
                raw.put(pids.join("").as_bytes());
                raw.to_vec()
            },
            Self::PlayerJoin(pid) => {
                let mut raw = BytesMut::new();
                raw.put_u8(PLAYER_JOIN);
                raw.put(pid.as_bytes());
                raw.to_vec()
            },
            Self::PlayerDestroy(pid) => {
                let mut raw = BytesMut::new();
                raw.put_u8(PLAYER_DESTROY);
                raw.put(pid.as_bytes());
                raw.to_vec()
            },
            Self::PlayerEvent { pid, event } => {
                let mut raw = BytesMut::new();
                raw.put_u8(PLAYER_EVENT);
                raw.put(pid.as_bytes());
                raw.put_u8(event.parse());
                raw.to_vec()
            },
            Self::PlayerPos { pid, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(PLAYER_POS);
                raw.put(pid.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            }
            Self::BulletCreate { id, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(BULLET_CREATE);
                raw.put(id.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            },
            Self::BulletDestroy(id) => {
                let mut raw = BytesMut::new();
                raw.put_u8(BULLET_DESTROY);
                raw.put(id.as_bytes());
                raw.to_vec()
            },
            Self::EnemyCreate { id, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(ENEMY_CREATE);
                raw.put(id.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            },
            Self::EnemyDestroy(id) => {
                let mut raw = BytesMut::new();
                raw.put_u8(ENEMY_DESTROY);
                raw.put(id.as_bytes());
                raw.to_vec()
            },
            Self::GameWon => {
                vec![GAME_WON]
            },
            Self::GameLost => {
                vec![GAME_LOST]
            },
        }
    }

    pub fn read_from(stream: &mut impl Read) -> Result<Option<Self>> {
        let first_byte = Self::read_u8(stream)?;
        match first_byte {
            GAME_INFO => {
                let width = Self::read_u32(stream)?;
                let height = Self::read_u32(stream)?;
                let len = Self::read_u8(stream)?;
                let pids = Self::read_len_str(stream, len as usize, PLAYER_ID_MAX)?;
                Ok(Some(Self::GameInfo{width, height, pids}))
            }
            PLAYER_JOIN => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                Ok(Some(Self::PlayerJoin(pid)))
            }
            PLAYER_DESTROY => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                Ok(Some(Self::PlayerDestroy(pid)))
            }
            PLAYER_EVENT => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                let event = PlayerEvent::get(Self::read_u8(stream)?).unwrap();
                Ok(Some(Self::PlayerEvent{pid, event}))
            }
            PLAYER_POS => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::PlayerPos{pid, x, y}))
            }
            BULLET_CREATE => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?; 
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::BulletCreate{id, x, y}))
            }
            BULLET_DESTROY => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?;
                Ok(Some(Self::BulletDestroy(id)))
            }
            ENEMY_CREATE => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?; 
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::EnemyCreate{id, x, y}))
            }
            ENEMY_DESTROY => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?;
                Ok(Some(Self::EnemyDestroy(id)))
            }
            GAME_WON => {
                Ok(Some(Self::GameWon))
            }
            GAME_LOST => {
                Ok(Some(Self::GameLost))
            }
            _ => Err(String::from("invalid first byte").into()),
        }
    }

    // for consistency
    fn read_u8(stream: &mut impl Read) -> Result<u8> {
        Ok(stream.read_u8()?)
    }

    fn read_u32(stream: &mut impl Read) -> Result<u32> {
        Ok(stream.read_u32::<byteorder::BigEndian>()?)
    }

    // read string of length
    fn read_str(stream: &mut impl Read, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        stream.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    // read consecutive strings of size s into vector of len
    fn read_len_str(stream: &mut impl Read, len: usize, s: usize) -> Result<Vec<String>> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(Self::read_str(stream, s)?);
        }
        Ok(vec)
    }

    pub async fn async_read_from<T: AsyncBufRead + Unpin>(stream: &mut T) -> Result<Option<Self>> {
        let first_byte = Self::async_read_u8(stream).await?;
        match first_byte {
            GAME_INFO => {
                let width = Self::async_read_u32(stream).await?;
                let height = Self::async_read_u32(stream).await?;
                let len = Self::async_read_u8(stream).await?;
                let pids = Self::async_read_len_str(stream, len as usize, PLAYER_ID_MAX).await?;
                Ok(Some(Self::GameInfo{width, height, pids}))
            }
            PLAYER_JOIN => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                Ok(Some(Self::PlayerJoin(pid)))
            }
            PLAYER_DESTROY => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                Ok(Some(Self::PlayerDestroy(pid)))
            }
            PLAYER_EVENT => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                let event = PlayerEvent::get(Self::async_read_u8(stream).await?).unwrap();
                Ok(Some(Self::PlayerEvent{pid, event}))
            }
            PLAYER_POS => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::PlayerPos{pid, x, y}))
            }
            BULLET_CREATE => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::BulletCreate{id, x, y}))
            }
            BULLET_DESTROY => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                Ok(Some(Self::BulletDestroy(id)))
            }
            ENEMY_CREATE => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::EnemyCreate{id, x, y}))
            }
            ENEMY_DESTROY => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                Ok(Some(Self::EnemyDestroy(id)))
            }
            GAME_WON => {
                Ok(Some(Self::GameWon))
            }
            GAME_LOST => {
                Ok(Some(Self::GameLost))
            }
            _ => Err(String::from("invalid first byte").into()),
        }
    }

    // for consistency
    async fn async_read_u8<T: AsyncBufRead + Unpin>(stream: &mut T) -> Result<u8> {
        Ok(stream.read_u8().await?)
    }

    async fn async_read_u32<T: AsyncBufRead + Unpin>(stream: &mut T) -> Result<u32> {
        Ok(stream.read_u32().await?)
    }

    // read string of length
    async fn async_read_str<T: AsyncBufRead + Unpin>(stream: &mut T, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        stream.read_exact(&mut buf).await?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    // read consecutive strings of size s into vector of len
    async fn async_read_len_str<T: AsyncBufRead + Unpin>(stream: &mut T, len: usize, s: usize) -> Result<Vec<String>> {
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(Self::async_read_str(stream, s).await?);
        }
        Ok(vec)
    }
}

#[derive(Clone, Debug)]
pub enum PlayerEvent {
    Fire,
    Exit,
}

impl PlayerEvent {
    pub fn parse(&self) -> u8 {
        match self {
            Self::Fire      => 0,
            Self::Exit      => 1,
        }
    }

    pub fn get(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Fire),
            1 => Ok(Self::Exit),
            _ => Err(String::from("invalid event").into()),
        }
    }
}