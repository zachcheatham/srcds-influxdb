use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use std::error::Error;
use std::io::{Cursor, Read, BufRead};

pub struct SourceQuery {
    host: String,
    port: u16,
    pub full_host: String,
    pub community: String
}

#[derive(Debug)]
pub struct A2SInfoResult {
    pub ping: u128,
    pub server_name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub game_id: u16,
    pub num_players: u8,
    pub num_bots: u8,
    pub max_players: u8
}

const A2SINFO_PACKET: &[u8] = b"\xFF\xFF\xFF\xFF\x54Source Engine Query\x00";
const PACKET_HEADER: i32 = -1;
const S2AINFO_HEADER: u8 = b'\x49';
const CHALLENGE_HEADER: u8 = b'\x41';

impl SourceQuery {

    pub fn new(host: String, port: u16, community: String) -> Self {
        let full_host = format!("{}:{}", host, port);
        SourceQuery { host, port, full_host, community }
    }

    pub fn query_a2s_info(&self) -> Result<A2SInfoResult, Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let mut buf = [0; 200];

        // let debug_str = A2SINFO_PACKET.iter().map(|b| format!("{:02X}", b)).collect::<String>();

        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        let ping_start = SystemTime::now();
        socket.send_to(A2SINFO_PACKET, &self.full_host)?;
        let (_bytes, src) = socket.recv_from(&mut buf)?;
        let ping = SystemTime::now().duration_since(ping_start).unwrap().as_millis();

        if src.to_string() != self.full_host {
            return Err("Invalid host responded.".into());
        }

        // println!("{:x?}", buf);
        
        if PACKET_HEADER != i32::from_le_bytes(buf[0..4].try_into().unwrap()) {
            return Err("Invalid packet header in response.".into());
        }

        if CHALLENGE_HEADER == buf[4] {
            let mut challenge_reply = A2SINFO_PACKET.to_vec();
            challenge_reply.extend_from_slice(&buf[5..9]);

            // println!("{:x?}", challenge_reply);

            socket.send_to(&challenge_reply, &self.full_host)?;
            socket.recv_from(&mut buf)?;

            // println!("{:x?}", buf);

            if PACKET_HEADER != i32::from_le_bytes(buf[0..4].try_into().unwrap()) {
                return Err("Invalid packet header in response after challenge.".into());
            }

            Ok(handle_a2s_response(&buf, ping))
        }
        else
        {
            Ok(handle_a2s_response(&buf, ping))
        }
    }
    
}

fn handle_a2s_response(buf: &[u8], ping: u128) -> A2SInfoResult {

    let mut cursor = Cursor::new(buf);
    cursor.set_position(4);

    if S2AINFO_HEADER != read_byte(&mut cursor) {
        println!("Invalid S2A_INFO packet header in response.");
    }

    let server_name = read_string(&mut cursor);
    let server_map = read_string(&mut cursor);
    let server_mod = read_string(&mut cursor);
    let server_game = read_string(&mut cursor);
    let game_id = read_short(&mut cursor);
    let mut num_players = read_byte(&mut cursor);
    let max_players = read_byte(&mut cursor);
    let bot_players = read_byte(&mut cursor);

    if bot_players <= num_players {
        num_players -= bot_players;
    }

    // println!("{} (map: {}) (mod: {}) (game: {}) (game_id: {}) (players: {}) (max_players: {}) (bots: {})",
    //     server_name, server_map, server_mod, server_game, game_id, num_players, max_players, bot_players);

    A2SInfoResult {
        ping,
        server_name,
        map: server_map,
        folder: server_mod,
        game: server_game,
        game_id,
        num_players,
        num_bots: bot_players,
        max_players
    }
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> String {
    let mut string_bytes = vec![];
    cursor.read_until(0, &mut string_bytes).unwrap();
    String::from_utf8_lossy(&string_bytes[..string_bytes.len() - 1]).to_string()
}

fn read_byte(cursor: &mut Cursor<&[u8]>) -> u8 {
    let mut byte = [0; 1];
    cursor.read_exact(&mut byte).unwrap();
    byte[0]
}

fn read_short(cursor: &mut Cursor<&[u8]>) -> u16 {
    let mut bytes = [0; 2];
    cursor.read_exact(&mut bytes).unwrap();
    u16::from_le_bytes(bytes)
}

fn read_long(cursor: &mut Cursor<&[u8]>) -> i32 {
    let mut bytes = [0; 4];
    cursor.read_exact(&mut bytes).unwrap();
    i32::from_le_bytes(bytes)
}

