use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, UdpSocket};

use crate::game::{PlayerId, TankInput, Vec2};

pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_PACKET_SIZE: usize = 512;
const MAGIC: [u8; 4] = *b"TR01";

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Hello {
        player_name: String,
    },
    JoinRequest {
        player_name: String,
    },
    JoinAccepted {
        player_id: PlayerId,
    },
    Ready {
        player_id: PlayerId,
        ready: bool,
    },
    Input {
        player_id: PlayerId,
        input: TankInput,
    },
    Snapshot {
        tick: u32,
        tanks: Vec<TankSnapshot>,
        projectiles: Vec<ProjectileSnapshot>,
    },
    Disconnect {
        player_id: PlayerId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankSnapshot {
    pub player_id: PlayerId,
    pub position: Vec2,
    pub rotation_radians: f32,
    pub alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectileSnapshot {
    pub owner: PlayerId,
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Packet {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(&MAGIC);
        bytes.push(PROTOCOL_VERSION);
        match self {
            Self::Hello { player_name } => {
                bytes.push(1);
                put_string(&mut bytes, player_name)?;
            }
            Self::JoinRequest { player_name } => {
                bytes.push(2);
                put_string(&mut bytes, player_name)?;
            }
            Self::JoinAccepted { player_id } => {
                bytes.push(3);
                bytes.push(player_id.0);
            }
            Self::Ready { player_id, ready } => {
                bytes.push(4);
                bytes.push(player_id.0);
                bytes.push(u8::from(*ready));
            }
            Self::Input { player_id, input } => {
                bytes.push(5);
                bytes.push(player_id.0);
                let flags = u8::from(input.forward)
                    | (u8::from(input.backward) << 1)
                    | (u8::from(input.left) << 2)
                    | (u8::from(input.right) << 3)
                    | (u8::from(input.fire) << 4);
                bytes.push(flags);
            }
            Self::Snapshot {
                tick,
                tanks,
                projectiles,
            } => {
                bytes.push(6);
                bytes.extend_from_slice(&tick.to_le_bytes());
                put_count(&mut bytes, tanks.len())?;
                for tank in tanks {
                    bytes.push(tank.player_id.0);
                    put_f32(&mut bytes, tank.position.x);
                    put_f32(&mut bytes, tank.position.y);
                    put_f32(&mut bytes, tank.rotation_radians);
                    bytes.push(u8::from(tank.alive));
                }
                put_count(&mut bytes, projectiles.len())?;
                for projectile in projectiles {
                    bytes.push(projectile.owner.0);
                    put_f32(&mut bytes, projectile.position.x);
                    put_f32(&mut bytes, projectile.position.y);
                    put_f32(&mut bytes, projectile.velocity.x);
                    put_f32(&mut bytes, projectile.velocity.y);
                }
            }
            Self::Disconnect { player_id } => {
                bytes.push(7);
                bytes.push(player_id.0);
            }
        }
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge);
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < 6 || bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::InvalidLength);
        }
        if bytes[0..4] != MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }
        if bytes[4] != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(bytes[4]));
        }
        let mut reader = Reader::new(&bytes[5..]);
        let packet_type = reader.u8()?;
        let packet = match packet_type {
            1 => Self::Hello {
                player_name: reader.string()?,
            },
            2 => Self::JoinRequest {
                player_name: reader.string()?,
            },
            3 => Self::JoinAccepted {
                player_id: PlayerId(reader.u8()?),
            },
            4 => Self::Ready {
                player_id: PlayerId(reader.u8()?),
                ready: reader.u8()? != 0,
            },
            5 => {
                let player_id = PlayerId(reader.u8()?);
                let flags = reader.u8()?;
                Self::Input {
                    player_id,
                    input: TankInput {
                        forward: flags & 1 != 0,
                        backward: flags & 2 != 0,
                        left: flags & 4 != 0,
                        right: flags & 8 != 0,
                        fire: flags & 16 != 0,
                    },
                }
            }
            6 => {
                let tick = reader.u32()?;
                let tank_count = reader.count()?;
                let mut tanks = Vec::with_capacity(tank_count);
                for _ in 0..tank_count {
                    tanks.push(TankSnapshot {
                        player_id: PlayerId(reader.u8()?),
                        position: Vec2::new(reader.f32()?, reader.f32()?),
                        rotation_radians: reader.f32()?,
                        alive: reader.u8()? != 0,
                    });
                }
                let projectile_count = reader.count()?;
                let mut projectiles = Vec::with_capacity(projectile_count);
                for _ in 0..projectile_count {
                    projectiles.push(ProjectileSnapshot {
                        owner: PlayerId(reader.u8()?),
                        position: Vec2::new(reader.f32()?, reader.f32()?),
                        velocity: Vec2::new(reader.f32()?, reader.f32()?),
                    });
                }
                Self::Snapshot {
                    tick,
                    tanks,
                    projectiles,
                }
            }
            7 => Self::Disconnect {
                player_id: PlayerId(reader.u8()?),
            },
            _ => return Err(ProtocolError::UnknownPacket(packet_type)),
        };
        if reader.remaining() != 0 {
            return Err(ProtocolError::TrailingBytes);
        }
        Ok(packet)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    InvalidLength,
    InvalidMagic,
    UnsupportedVersion(u8),
    UnknownPacket(u8),
    InvalidUtf8,
    PacketTooLarge,
    Malformed,
    TrailingBytes,
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.cursor
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ProtocolError> {
        let end = self
            .cursor
            .checked_add(count)
            .ok_or(ProtocolError::Malformed)?;
        if end > self.bytes.len() {
            return Err(ProtocolError::Malformed);
        }
        let slice = &self.bytes[self.cursor..end];
        self.cursor = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, ProtocolError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    fn f32(&mut self) -> Result<f32, ProtocolError> {
        Ok(f32::from_le_bytes(self.u32()?.to_le_bytes()))
    }

    fn count(&mut self) -> Result<usize, ProtocolError> {
        let count = self.u8()? as usize;
        if count > 32 {
            return Err(ProtocolError::Malformed);
        }
        Ok(count)
    }

    fn string(&mut self) -> Result<String, ProtocolError> {
        let length = self.u8()? as usize;
        if length > 32 {
            return Err(ProtocolError::Malformed);
        }
        std::str::from_utf8(self.take(length)?)
            .map(str::to_owned)
            .map_err(|_| ProtocolError::InvalidUtf8)
    }
}

fn put_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProtocolError> {
    let value = value.as_bytes();
    if value.len() > 32 {
        return Err(ProtocolError::Malformed);
    }
    bytes.push(value.len() as u8);
    bytes.extend_from_slice(value);
    Ok(())
}

fn put_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), ProtocolError> {
    if count > 32 {
        return Err(ProtocolError::Malformed);
    }
    bytes.push(count as u8);
    Ok(())
}

fn put_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[derive(Debug)]
pub struct LanHost {
    socket: UdpSocket,
    peers: HashMap<PlayerId, SocketAddr>,
}

impl LanHost {
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            peers: HashMap::new(),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn register_peer(&mut self, player_id: PlayerId, addr: SocketAddr) {
        self.peers.insert(player_id, addr);
    }

    pub fn broadcast(&self, packet: &Packet) -> io::Result<()> {
        let encoded = packet
            .encode()
            .map_err(|_| io::Error::other("packet encoding failed"))?;
        for addr in self.peers.values() {
            self.socket.send_to(&encoded, addr)?;
        }
        Ok(())
    }

    pub fn receive(&self) -> io::Result<Option<(Packet, SocketAddr)>> {
        let mut buffer = [0u8; MAX_PACKET_SIZE];
        match self.socket.recv_from(&mut buffer) {
            Ok((length, addr)) => match Packet::decode(&buffer[..length]) {
                Ok(packet) => Ok(Some((packet, addr))),
                Err(_) => Ok(None),
            },
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_packet_round_trips() {
        let packet = Packet::Input {
            player_id: PlayerId(2),
            input: TankInput {
                forward: true,
                backward: false,
                left: true,
                right: false,
                fire: true,
            },
        };
        let encoded = packet.encode().unwrap();
        assert_eq!(Packet::decode(&encoded).unwrap(), packet);
    }

    #[test]
    fn snapshot_round_trips() {
        let packet = Packet::Snapshot {
            tick: 42,
            tanks: vec![TankSnapshot {
                player_id: PlayerId(1),
                position: Vec2::new(12.0, 34.0),
                rotation_radians: 1.5,
                alive: true,
            }],
            projectiles: vec![ProjectileSnapshot {
                owner: PlayerId(0),
                position: Vec2::new(20.0, 30.0),
                velocity: Vec2::new(180.0, 0.0),
            }],
        };
        let encoded = packet.encode().unwrap();
        assert_eq!(Packet::decode(&encoded).unwrap(), packet);
    }

    #[test]
    fn malformed_packets_are_rejected() {
        assert!(matches!(
            Packet::decode(b"bad"),
            Err(ProtocolError::InvalidLength)
        ));
        let mut encoded = Packet::JoinRequest {
            player_name: "Player".into(),
        }
        .encode()
        .unwrap();
        encoded[4] = 99;
        assert!(matches!(
            Packet::decode(&encoded),
            Err(ProtocolError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn host_can_bind_and_register_peer() {
        let host = LanHost::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = host.local_addr().unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }
}
