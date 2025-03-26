use common::{Command, Response, SOCKET_PATH, receive_response, send_command};
use std::io::{self, Write};
use std::os::unix::net::UnixStream;

enum Language {
    English,
    Bulgarian,
}

struct ATM {
    stream: UnixStream,
    language: Language,
    card_number: Option<String>,
    pin: Option<String>,
}

impl ATM {
    fn new() -> io::Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH)?;
        Ok(ATM {
            stream,
            language: Language::English,
            card_number: None,
            pin: None,
        })
    }

    fn select_language(&mut self) {
        println!("Select language / Изберете език:");
        println!("1. English");
        println!("2. Български");

        let choice = self.read_choice();
        match choice {
            1 => self.language = Language::English,
            2 => self.language = Language::Bulgarian,
            _ => {
                println!("Invalid choice, defaulting to English");
                self.language = Language::English;
            }
        }
    }

    fn display_message(&self, eng: &str, bg: &str) {
        match self.language {
            Language::English => println!("{}", eng),
            Language::Bulgarian => println!("{}", bg),
        }
    }

    fn read_choice(&self) -> u32 {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        input.trim().parse().unwrap_or(0)
    }

    fn read_input(&self, prompt_eng: &str, prompt_bg: &str) -> String {
        self.display_message(prompt_eng, prompt_bg);
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        input.trim().to_string()
    }

    fn get_pin(&mut self) -> String {
        if let Some(pin) = &self.pin {
            return pin.clone();
        }

        // PIN is not set, request from user
        let pin = self.read_input("Enter your PIN:", "Въведете вашия ПИН:");
        self.pin = Some(pin.clone());
        pin
    }

    fn insert_card(&mut self) -> bool {
        // In a real system, this would read from a card reader
        // For simulation, we'll use predefined card keys
        self.display_message(
            "Available card keys for testing:",
            "Налични ключове на карти за тестване:",
        );
        println!("1. key123 (Card: 1234567890123456, PIN: 1234)");
        println!("2. key456 (Card: 9876543210987654, PIN: 4321)");

        let card_key = self.read_input("Enter your card key:", "Въведете ключа на вашата карта:");

        let command = Command::ValidateCardKey { card_key };

        if let Err(e) = send_command(&mut self.stream, &command) {
            self.display_message(
                &format!("Error sending card key validation: {}", e),
                &format!(
                    "Грешка при изпращане на валидация на ключа на картата: {}",
                    e
                ),
            );
            return false;
        }

        match receive_response(&mut self.stream) {
            Ok(Response::ValidateCardKeySuccess { card_number }) => {
                self.display_message(
                    "Card key validated successfully",
                    "Ключът на картата е успешно валидиран",
                );
                self.card_number = Some(card_number);
                true
            }
            Ok(Response::ValidateCardKeyErrorInvalid) => {
                self.display_message("Invalid card key", "Невалиден ключ на картата");
                false
            }
            Ok(Response::ErrorCardNotFound) => {
                self.display_message("Card not found", "Картата не е намерена");
                false
            }
            Ok(Response::ErrorInvalidPin) => {
                self.display_message("Invalid PIN", "Невалиден ПИН");
                self.pin = None;
                false
            }
            Ok(Response::ErrorServerInternal) => {
                self.display_message("Server error", "Сървърна грешка");
                false
            }
            Ok(_) => {
                self.display_message(
                    "Unexpected response from server",
                    "Неочакван отговор от сървъра",
                );
                false
            }
            Err(e) => {
                self.display_message(
                    &format!("Error receiving response: {}", e),
                    &format!("Грешка при получаване на отговор: {}", e),
                );
                false
            }
        }
    }

    fn check_balance(&mut self) {
        let card_number = self.card_number.clone().unwrap();
        let pin = self.get_pin();

        let command = Command::CheckBalance { card_number, pin };

        if let Err(e) = send_command(&mut self.stream, &command) {
            self.display_message(
                &format!("Error sending balance check: {}", e),
                &format!("Грешка при изпращане на проверка на баланса: {}", e),
            );
            return;
        }

        match receive_response(&mut self.stream) {
            Ok(Response::CheckBalanceSuccess { amount }) => {
                self.display_message(
                    &format!("Your current balance is: ${:.2}", amount),
                    &format!("Текущият ви баланс е: ${:.2}", amount),
                );
            }
            Ok(Response::ErrorCardNotFound) => {
                self.display_message("Card not found", "Картата не е намерена");
            }
            Ok(Response::ErrorInvalidPin) => {
                self.display_message("Invalid PIN", "Невалиден ПИН");
                self.pin = None;
            }
            Ok(Response::ErrorServerInternal) => {
                self.display_message("Server error", "Сървърна грешка");
            }
            Ok(_) => {
                self.display_message(
                    "Unexpected response from server",
                    "Неочакван отговор от сървъра",
                );
            }
            Err(e) => {
                self.display_message(
                    &format!("Error receiving response: {}", e),
                    &format!("Грешка при получаване на отговор: {}", e),
                );
            }
        }
    }

    fn withdraw(&mut self) {
        let amount_str = self.read_input("Enter amount to withdraw:", "Въведете сума за теглене:");

        let amount = match amount_str.parse::<f64>() {
            Ok(amount) if amount > 0.0 => amount,
            _ => {
                self.display_message("Invalid amount", "Невалидна сума");
                return;
            }
        };

        let want_receipt = self.read_input(
            "Do you want a receipt? (y/n):",
            "Искате ли касова бележка? (y/n):",
        );

        let want_receipt = want_receipt.to_lowercase() == "y";

        let card_number = self.card_number.clone().unwrap();
        let pin = self.get_pin();

        let command = Command::Withdraw {
            card_number,
            pin,
            amount,
        };

        if let Err(e) = send_command(&mut self.stream, &command) {
            self.display_message(
                &format!("Error sending withdrawal request: {}", e),
                &format!("Грешка при изпращане на заявка за теглене: {}", e),
            );
            return;
        }

        match receive_response(&mut self.stream) {
            Ok(Response::WithdrawSuccess { new_balance }) => {
                self.display_message(
                    &format!("Successfully withdrew ${:.2}", amount),
                    &format!("Успешно изтеглихте ${:.2}", amount),
                );

                self.display_message(
                    &format!("Your new balance is: ${:.2}", new_balance),
                    &format!("Новият ви баланс е: ${:.2}", new_balance),
                );

                if want_receipt {
                    self.display_message("Printing receipt...", "Отпечатване на касова бележка...");

                    self.display_message(
                        &format!("=== RECEIPT ===\nWithdraw Amount: ${:.2}\nNew Balance: ${:.2}\n==============", amount, new_balance),
                        &format!("=== КАСОВА БЕЛЕЖКА ===\nИзтеглена Сума: ${:.2}\nНов Баланс: ${:.2}\n====================", amount, new_balance)
                    );
                }
            }
            Ok(Response::WithdrawErrorInsufficientFunds) => {
                self.display_message("Insufficient funds", "Недостатъчна наличност");
            }
            Ok(Response::ErrorCardNotFound) => {
                self.display_message("Card not found", "Картата не е намерена");
            }
            Ok(Response::ErrorInvalidPin) => {
                self.display_message("Invalid PIN", "Невалиден ПИН");
                self.pin = None;
            }
            Ok(Response::ErrorServerInternal) => {
                self.display_message("Server error", "Сървърна грешка");
            }
            Ok(_) => {
                self.display_message(
                    "Unexpected response from server",
                    "Неочакван отговор от сървъра",
                );
            }
            Err(e) => {
                self.display_message(
                    &format!("Error receiving response: {}", e),
                    &format!("Грешка при получаване на отговор: {}", e),
                );
            }
        }
    }

    fn run(&mut self) {
        println!("=============================");
        println!("Welcome to the ATM System");
        println!("=============================");

        self.select_language();

        if !self.insert_card() {
            self.display_message(
                "Card validation failed. Exiting...",
                "Валидирането на картата е неуспешно. Изход...",
            );
            return;
        }

        loop {
            println!();
            if let Some(card_number) = &self.card_number {
                let last_four = &card_number[card_number.len() - 4..];
                let x_count = card_number.len() - 4;
                let masked_card = format!("{}{}", "x".repeat(x_count), last_four);
                self.display_message(
                    &format!("You are using card {}", masked_card),
                    &format!("Използвате карта {}", masked_card),
                );
            }
            self.display_message("Select an option:", "Изберете опция:");
            self.display_message(
                "1. Check Balance\n2. Withdraw Money\n3. Change language\n4. Exit",
                "1. Проверка на баланс\n2. Теглене на пари\n3. Промени езика (Change language)\n4. Изход",
            );

            let choice = self.read_choice();
            match choice {
                1 => self.check_balance(),
                2 => self.withdraw(),
                3 => self.select_language(),
                4 => {
                    self.display_message(
                        "Thank you for using our ATM. Goodbye!",
                        "Благодарим ви, че използвахте нашия банкомат. Довиждане!",
                    );
                    break;
                }
                _ => {
                    self.display_message(
                        "Invalid option. Please try again.",
                        "Невалидна опция. Моля, опитайте отново.",
                    );
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    println!("Starting ATM client...");

    let mut atm = ATM::new()?;
    atm.run();

    Ok(())
}
