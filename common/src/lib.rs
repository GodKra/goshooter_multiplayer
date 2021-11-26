use std::{io::Read, marker::Unpin};
use byteorder::ReadBytesExt;
use tokio::io::{AsyncBufRead, AsyncReadExt};
use bytes::{BufMut, BytesMut};

pub const BULLET_ID_LEN: usize = 8;
pub const PLAYER_ID_MAX: usize = 8;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Debug)]
pub enum Packet {
    GameInfo { width: u32, height: u32, pids: Vec<String> }, // sent when starting game | id 0x01

    PlayerJoin(String), // new player joins; name max 8 | id 0x02
    PlayerDestroy(String), // player leaves; name max 8 | id 0x03

    PlayerEvent {pid: String, event: PlayerEvent }, // id 0x04
    PlayerPos { pid: String, x: u32, y: u32 }, // player position sent every loop | id 0x05

    BulletCreate { id: String, x: u32, y: u32 }, // id 0x06
    BulletDestroy(String), // id 0x07

    EnemyCreate { id: String, x: u32, y: u32 }, // id 0x08
    EnemyDestroy(String), // id 0x09

    GameWon, // id 0x0A
    GameLost, // id 0x0B
}

impl Packet {
    pub fn parse(self) -> Vec<u8> {
        match self {
            Self::GameInfo { width, height, pids } => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x01);
                raw.put_u32(width);
                raw.put_u32(height);
                raw.put_u8(pids.len() as u8);
                raw.put(pids.join("").as_bytes());
                raw.to_vec()
            },
            Self::PlayerJoin(pid) => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x02);
                raw.put(pid.as_bytes());
                raw.to_vec()
            },
            Self::PlayerDestroy(pid) => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x03);
                raw.put(pid.as_bytes());
                raw.to_vec()
            },
            Self::PlayerEvent { pid, event } => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x04);
                raw.put(pid.as_bytes());
                raw.put_u8(event.parse());
                raw.to_vec()
            },
            Self::PlayerPos { pid, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x05);
                raw.put(pid.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            }
            Self::BulletCreate { id, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x06);
                raw.put(id.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            },
            Self::BulletDestroy(id) => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x07);
                raw.put(id.as_bytes());
                raw.to_vec()
            },
            Self::EnemyCreate { id, x, y } => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x08);
                raw.put(id.as_bytes());
                raw.put_u32(x);
                raw.put_u32(y);
                raw.to_vec()
            },
            Self::EnemyDestroy(id) => {
                let mut raw = BytesMut::new();
                raw.put_u8(0x09);
                raw.put(id.as_bytes());
                raw.to_vec()
            },
            Self::GameWon => {
                vec![0x0A]
            },
            Self::GameLost => {
                vec![0x0B]
            },
        }
    }

    pub fn read_from(stream: &mut impl Read) -> Result<Option<Self>> {
        let first_byte = Self::read_u8(stream)?;
        match first_byte {
            0x01 => {
                let width = Self::read_u32(stream)?;
                let height = Self::read_u32(stream)?;
                let len = Self::read_u8(stream)?;
                let pids = Self::read_len_str(stream, len as usize, PLAYER_ID_MAX)?;
                Ok(Some(Self::GameInfo{width, height, pids}))
            }
            0x02 => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                Ok(Some(Self::PlayerJoin(pid)))
            }
            0x03 => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                Ok(Some(Self::PlayerDestroy(pid)))
            }
            0x04 => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                let event = PlayerEvent::get(Self::read_u8(stream)?).unwrap();
                Ok(Some(Self::PlayerEvent{pid, event}))
            }
            0x05 => {
                let pid = Self::read_str(stream, PLAYER_ID_MAX)?;
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::PlayerPos{pid, x, y}))
            }
            0x06 => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?; 
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::BulletCreate{id, x, y}))
            }
            0x07 => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?;
                Ok(Some(Self::BulletDestroy(id)))
            }
            0x08 => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?; 
                let x = Self::read_u32(stream)?;
                let y = Self::read_u32(stream)?;
                Ok(Some(Self::EnemyCreate{id, x, y}))
            }
            0x09 => {
                let id = Self::read_str(stream, BULLET_ID_LEN)?;
                Ok(Some(Self::EnemyDestroy(id)))
            }
            0x0A => {
                Ok(Some(Self::GameWon))
            }
            0x0B => {
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
            0x01 => {
                let width = Self::async_read_u32(stream).await?;
                let height = Self::async_read_u32(stream).await?;
                let len = Self::async_read_u8(stream).await?;
                let pids = Self::async_read_len_str(stream, len as usize, PLAYER_ID_MAX).await?;
                Ok(Some(Self::GameInfo{width, height, pids}))
            }
            0x02 => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                Ok(Some(Self::PlayerJoin(pid)))
            }
            0x03 => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                Ok(Some(Self::PlayerDestroy(pid)))
            }
            0x04 => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                let event = PlayerEvent::get(Self::async_read_u8(stream).await?).unwrap();
                Ok(Some(Self::PlayerEvent{pid, event}))
            }
            0x05 => {
                let pid = Self::async_read_str(stream, PLAYER_ID_MAX).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::PlayerPos{pid, x, y}))
            }
            0x06 => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::BulletCreate{id, x, y}))
            }
            0x07 => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                Ok(Some(Self::BulletDestroy(id)))
            }
            0x08 => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                let x = Self::async_read_u32(stream).await?;
                let y = Self::async_read_u32(stream).await?;
                Ok(Some(Self::EnemyCreate{id, x, y}))
            }
            0x09 => {
                let id = Self::async_read_str(stream, BULLET_ID_LEN).await?;
                Ok(Some(Self::EnemyDestroy(id)))
            }
            0x0A => {
                Ok(Some(Self::GameWon))
            }
            0x0B => {
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