use serde::{Deserialize, Serialize};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub card_key: String,
    pub card_number: String,
    pub pin: String,
    pub balance: f64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    ValidateCardKey {
        card_key: String,
    },
    Withdraw {
        card_number: String,
        pin: String,
        amount: f64,
    },
    CheckBalance {
        card_number: String,
        pin: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    ValidateCardKeySuccess { card_number: String },
    ValidateCardKeyErrorInvalid,

    WithdrawSuccess { new_balance: f64 },
    WithdrawErrorInsufficientFunds,

    CheckBalanceSuccess { amount: f64 },

    ErrorServerInternal,
    ErrorInvalidPin,
    ErrorCardNotFound,
}

pub fn send_command(stream: &mut UnixStream, command: &Command) -> io::Result<()> {
    let serialized = serde_json::to_string(command)?;
    let mut writer = BufWriter::new(stream);

    // Send length as u32 first (4 bytes), then the JSON data
    let len = serialized.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(serialized.as_bytes())?;
    writer.flush()?;

    Ok(())
}

pub fn receive_response(stream: &mut UnixStream) -> io::Result<Response> {
    let mut reader = BufReader::new(stream);

    // Read length first (4 bytes)
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Read JSON data
    let mut buffer = vec![0u8; len];
    reader.read_exact(&mut buffer)?;

    let response: Response = serde_json::from_slice(&buffer)?;
    Ok(response)
}

pub fn send_response(stream: &mut UnixStream, response: &Response) -> io::Result<()> {
    let serialized = serde_json::to_string(response)?;
    let mut writer = BufWriter::new(stream);

    // Send length as u32 first (4 bytes), then the JSON data
    let len = serialized.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(serialized.as_bytes())?;
    writer.flush()?;

    Ok(())
}

pub fn receive_command(stream: &mut UnixStream) -> io::Result<Command> {
    let mut reader = BufReader::new(stream);

    // Read length first (4 bytes)
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Read JSON data
    let mut buffer = vec![0u8; len];
    reader.read_exact(&mut buffer)?;

    let command: Command = serde_json::from_slice(&buffer)?;
    Ok(command)
}

pub const SOCKET_PATH: &str = "/tmp/atm_bank_socket";
