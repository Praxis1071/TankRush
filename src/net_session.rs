use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::game::{GameConfig, GameMap, GameMode, GameSimulation, PlayerId, TankInput};
use crate::lobby::{LobbyState, MAX_LOBBY_PLAYERS};
use crate::network::{LanHost, Packet, ProjectileSnapshot, TankSnapshot};

pub const PEER_TIMEOUT: Duration = Duration::from_secs(5);
pub const SNAPSHOT_INTERVAL_TICKS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerEvent {
    PlayerJoined(PlayerId),
    PlayerLeft(PlayerId),
    MatchStarted,
    MatchUpdated,
}

#[derive(Debug)]
pub struct AuthoritativeServer {
    pub transport: LanHost,
    pub lobby: LobbyState,
    pub simulation: GameSimulation,
    inputs: HashMap<PlayerId, TankInput>,
    last_seen: HashMap<PlayerId, Instant>,
    pub tick: u32,
    pub match_started: bool,
}

impl AuthoritativeServer {
    pub fn bind(
        addr: SocketAddr,
        mode: GameMode,
        team_count: usize,
        map: GameMap,
        config: GameConfig,
    ) -> io::Result<Self> {
        Ok(Self {
            transport: LanHost::bind(addr)?,
            lobby: LobbyState::new(mode, team_count)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid lobby rules"))?,
            simulation: GameSimulation::new(map, config),
            inputs: HashMap::new(),
            last_seen: HashMap::new(),
            tick: 0,
            match_started: false,
        })
    }

    pub fn poll(&mut self, now: Instant) -> io::Result<Vec<ServerEvent>> {
        let mut events = Vec::new();
        while let Some((packet, addr)) = self.transport.receive()? {
            events.extend(self.handle_packet(packet, addr, now)?);
        }
        events.extend(self.remove_timed_out(now)?);
        Ok(events)
    }

    pub fn step(&mut self, now: Instant) -> io::Result<Vec<ServerEvent>> {
        let mut events = self.poll(now)?;
        if !self.match_started && self.lobby.all_ready() {
            self.start_match()?;
            events.push(ServerEvent::MatchStarted);
        }
        if !self.match_started {
            return Ok(events);
        }

        let inputs: Vec<_> = self
            .inputs
            .iter()
            .map(|(&id, &input)| (id, input))
            .collect();
        let simulation_events = self.simulation.step(&inputs);
        self.tick = self.tick.wrapping_add(1);
        if self.tick.is_multiple_of(SNAPSHOT_INTERVAL_TICKS) {
            self.broadcast_snapshot()?;
            if !simulation_events.is_empty() {
                events.push(ServerEvent::MatchUpdated);
            }
        }
        Ok(events)
    }

    fn handle_packet(
        &mut self,
        packet: Packet,
        addr: SocketAddr,
        now: Instant,
    ) -> io::Result<Vec<ServerEvent>> {
        let mut events = Vec::new();
        match packet {
            Packet::JoinRequest { player_name } => {
                if let Some(existing_id) = self.transport.player_for_addr(addr) {
                    self.last_seen.insert(existing_id, now);
                    self.transport.send_to(
                        &Packet::JoinAccepted {
                            player_id: existing_id,
                        },
                        addr,
                    )?;
                    return Ok(events);
                }
                if self.match_started || self.lobby.players.len() >= MAX_LOBBY_PLAYERS {
                    return Ok(events);
                }
                let player_id = self
                    .lobby
                    .join(player_name)
                    .ok_or_else(|| io::Error::other("lobby rejected player"))?;
                if self.simulation.state.add_player(
                    self.lobby
                        .players
                        .iter()
                        .find(|player| player.id == player_id)
                        .map(|player| player.name.clone())
                        .unwrap_or_else(|| "Player".to_owned()),
                    self.lobby
                        .players
                        .iter()
                        .find(|player| player.id == player_id)
                        .and_then(|player| player.team),
                ) != Some(player_id)
                {
                    self.lobby.leave(player_id);
                    return Ok(events);
                }
                self.transport.register_peer(player_id, addr);
                self.last_seen.insert(player_id, now);
                self.inputs.insert(player_id, TankInput::idle());
                self.transport
                    .send_to(&Packet::JoinAccepted { player_id }, addr)?;
                events.push(ServerEvent::PlayerJoined(player_id));
            }
            Packet::Ready { player_id, ready } => {
                if self.transport.player_for_addr(addr) == Some(player_id)
                    && self.lobby.set_ready(player_id, ready)
                {
                    self.last_seen.insert(player_id, now);
                }
            }
            Packet::Input { player_id, input } => {
                if self.match_started
                    && self.transport.player_for_addr(addr) == Some(player_id)
                    && self
                        .lobby
                        .players
                        .iter()
                        .any(|player| player.id == player_id)
                {
                    self.inputs.insert(player_id, input);
                    self.last_seen.insert(player_id, now);
                }
            }
            Packet::Disconnect { player_id } => {
                if self.transport.player_for_addr(addr) == Some(player_id)
                    && self.remove_player(player_id)?
                {
                    events.push(ServerEvent::PlayerLeft(player_id));
                }
            }
            Packet::Hello { .. } | Packet::JoinAccepted { .. } | Packet::Snapshot { .. } => {}
        }
        Ok(events)
    }

    fn remove_timed_out(&mut self, now: Instant) -> io::Result<Vec<ServerEvent>> {
        let expired: Vec<_> = self
            .last_seen
            .iter()
            .filter_map(|(&player_id, &last_seen)| {
                (now.duration_since(last_seen) >= PEER_TIMEOUT).then_some(player_id)
            })
            .collect();
        let mut events = Vec::new();
        for player_id in expired {
            if self.remove_player(player_id)? {
                events.push(ServerEvent::PlayerLeft(player_id));
            }
        }
        Ok(events)
    }

    fn remove_player(&mut self, player_id: PlayerId) -> io::Result<bool> {
        let existed = self.lobby.leave(player_id);
        if existed {
            self.transport.unregister_peer(player_id);
            self.last_seen.remove(&player_id);
            self.inputs.remove(&player_id);
            self.simulation.state.destroy_tank(player_id);
            self.transport
                .broadcast(&Packet::Disconnect { player_id })?;
        }
        Ok(existed)
    }

    fn start_match(&mut self) -> io::Result<()> {
        for (index, player) in self.lobby.players.iter().enumerate() {
            let spawn = self
                .simulation
                .map
                .spawn_points()
                .get(index)
                .copied()
                .ok_or_else(|| io::Error::other("map has insufficient spawn points"))?;
            self.simulation.state.add_tank(player.id, spawn);
            self.inputs.insert(player.id, TankInput::idle());
        }
        self.match_started = true;
        self.broadcast_snapshot()
    }

    fn broadcast_snapshot(&self) -> io::Result<()> {
        let tanks = self
            .simulation
            .state
            .tanks
            .iter()
            .map(|tank| TankSnapshot {
                player_id: tank.player_id,
                position: tank.position,
                rotation_radians: tank.rotation_radians,
                alive: tank.alive,
            })
            .collect();
        let projectiles = self
            .simulation
            .state
            .projectiles
            .iter()
            .map(|projectile| ProjectileSnapshot {
                owner: projectile.owner,
                position: projectile.position,
                velocity: projectile.velocity,
            })
            .collect();
        self.transport.broadcast(&Packet::Snapshot {
            tick: self.tick,
            tanks,
            projectiles,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MapSize;
    use crate::network::LanClient;

    #[test]
    fn join_ready_and_input_are_authorized_by_address() {
        let mut server = AuthoritativeServer::bind(
            "127.0.0.1:0".parse().unwrap(),
            GameMode::FreeForAll,
            0,
            GameMap::rectangular(MapSize::Small),
            GameConfig::default(),
        )
        .unwrap();
        let server_addr = server.transport.local_addr().unwrap();
        let client = LanClient::bind("127.0.0.1:0".parse().unwrap(), server_addr).unwrap();
        let client_addr = client.local_addr().unwrap();
        client
            .send(&Packet::JoinRequest {
                player_name: "Alice".into(),
            })
            .unwrap();
        server.poll(Instant::now()).unwrap();
        assert_eq!(server.lobby.players.len(), 1);
        assert_eq!(
            server.transport.player_for_addr(client_addr),
            Some(PlayerId(0))
        );

        client
            .send(&Packet::Ready {
                player_id: PlayerId(0),
                ready: true,
            })
            .unwrap();
        server.step(Instant::now()).unwrap();
        assert!(server.match_started);
    }

    #[test]
    fn timed_out_peer_is_removed() {
        let mut server = AuthoritativeServer::bind(
            "127.0.0.1:0".parse().unwrap(),
            GameMode::FreeForAll,
            0,
            GameMap::rectangular(MapSize::Small),
            GameConfig::default(),
        )
        .unwrap();
        let server_addr = server.transport.local_addr().unwrap();
        let client = LanClient::bind("127.0.0.1:0".parse().unwrap(), server_addr).unwrap();
        client
            .send(&Packet::JoinRequest {
                player_name: "Alice".into(),
            })
            .unwrap();
        let now = Instant::now();
        server.poll(now).unwrap();
        assert_eq!(server.lobby.players.len(), 1);
        server
            .poll(now + PEER_TIMEOUT + Duration::from_millis(1))
            .unwrap();
        assert!(server.lobby.players.is_empty());
    }
}
