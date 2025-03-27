# ATM System

A simple ATM system implementation in Rust that simulates basic banking operations through a terminal interface. The system consists of three main components: an ATM client, a bank server, and a shared communication library.

## Features

- Multi-language support (English and Bulgarian)
- Card validation and PIN verification
- Balance checking
- Money withdrawal with receipt option
- Secure communication between ATM and bank server
- Persistent account storage

## Prerequisites

- Rust (latest stable version)
- Unix-like operating system (for Unix socket support)

## Project Structure

The project consists of three main components:

1. `atm/` - The ATM client interface
2. `bank/` - The bank server
3. `common/` - Shared library for communication

## Clone the repository

```bash
git clone https://github.com/NovaBG03/atm-system
cd atm-system
```

## Running the Application

1. First, start the bank server in a separate terminal:

```bash
cargo run --bin bank
```

2. In another terminal, start the ATM client:

```bash
cargo run --bin atm
```

## Test Accounts

For testing purposes, the following accounts are available:

1. Card: 1234567890123456

   - PIN: 1234
   - Key: key123

2. Card: 9876543210987654
   - PIN: 4321
   - Key: key456

## Usage

1. When the ATM starts, you'll be prompted to select a language (English or Bulgarian)
2. Enter a card key (use one of the test keys provided above)
3. Choose from the following options:
   - Check Balance
   - Withdraw Money
   - Change Language
   - Exit

## Technical Details

- Communication between ATM and bank server is handled through Unix sockets
- Account data is stored in JSON format
- All transactions are validated by the bank server
- PIN verification is performed locally at the ATM

## Security Notes

- This is a simulation system and should not be used for real banking operations
- PIN codes are stored in plain text for demonstration purposes
- The system uses Unix sockets for local communication only
