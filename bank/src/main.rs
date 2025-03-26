use common::{Account, Command, Response, SOCKET_PATH, receive_command, send_response};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

const ACCOUNTS_FILE: &str = "accounts.json";

fn load_accounts() -> HashMap<String, Account> {
    if !Path::new(ACCOUNTS_FILE).exists() {
        // Create some sample accounts if the file doesn't exist
        let mut accounts = HashMap::new();

        accounts.insert(
            "1234567890123456".to_string(),
            Account {
                card_key: "key123".to_string(),
                card_number: "1234567890123456".to_string(),
                pin: "1234".to_string(),
                balance: 1000.0,
                name: "John Doe".to_string(),
            },
        );

        accounts.insert(
            "9876543210987654".to_string(),
            Account {
                card_key: "key456".to_string(),
                card_number: "9876543210987654".to_string(),
                pin: "4321".to_string(),
                balance: 500.0,
                name: "Jane Smith".to_string(),
            },
        );

        let file = File::create(ACCOUNTS_FILE).unwrap();
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &accounts).unwrap();

        return accounts;
    }

    let file = File::open(ACCOUNTS_FILE).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| HashMap::new())
}

fn save_accounts(accounts: &HashMap<String, Account>) -> io::Result<()> {
    let file = File::create(ACCOUNTS_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, accounts)?;
    Ok(())
}

fn handle_client(
    mut stream: UnixStream,
    accounts: &mut HashMap<String, Account>,
) -> io::Result<()> {
    loop {
        match receive_command(&mut stream) {
            Ok(command) => {
                println!("Received command: {:?}", command);

                let response = match command {
                    Command::ValidateCardKey { card_key } => {
                        let mut found_card_number = None;
                        for (card_number, account) in accounts.iter() {
                            if account.card_key == card_key {
                                found_card_number = Some(card_number.clone());
                                break;
                            }
                        }

                        match found_card_number {
                            Some(card_number) => Response::ValidateCardKeySuccess { card_number },
                            None => Response::ValidateCardKeyErrorInvalid,
                        }
                    }
                    Command::Withdraw {
                        card_number,
                        pin,
                        amount,
                    } => {
                        if let Some(account) = accounts.get_mut(&card_number) {
                            if account.pin != pin {
                                Response::ErrorInvalidPin
                            } else if account.balance >= amount {
                                account.balance -= amount;
                                let result = Response::WithdrawSuccess {
                                    new_balance: account.balance,
                                };
                                save_accounts(accounts)?;
                                result
                            } else {
                                Response::WithdrawErrorInsufficientFunds
                            }
                        } else {
                            Response::ErrorCardNotFound
                        }
                    }
                    Command::CheckBalance { card_number, pin } => {
                        if let Some(account) = accounts.get(&card_number) {
                            if account.pin != pin {
                                Response::ErrorInvalidPin
                            } else {
                                Response::CheckBalanceSuccess {
                                    amount: account.balance,
                                }
                            }
                        } else {
                            Response::ErrorCardNotFound
                        }
                    }
                };

                send_response(&mut stream, &response)?;
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Client disconnected");
                    break;
                } else {
                    println!("Error receiving command: {:?}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    println!("Bank server starting...");

    // Load accounts
    let mut accounts = load_accounts();
    println!("Loaded {} accounts", accounts.len());

    // Remove the socket file if it already exists
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH)?;
    }

    // Create the Unix socket listener
    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("Listening on {}", SOCKET_PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected");
                // Clone the accounts for thread safety if multithreading is added later
                let mut accounts_clone = accounts.clone();

                // For now, handle clients synchronously
                if let Err(e) = handle_client(stream, &mut accounts_clone) {
                    println!("Error handling client: {:?}", e);
                }

                // Update the main accounts map
                accounts = accounts_clone;
            }
            Err(e) => {
                println!("Error accepting connection: {:?}", e);
            }
        }
    }

    Ok(())
}
