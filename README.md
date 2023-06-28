# Chatty Backend

Heavily influenced by [Actix Chat Example](https://github.com/actix/examples/tree/master/websockets/chat)

## Current Deployment

insert deployment url here.

## About

This repo serves as the backend for the [chatty frontend](https://github.com/adradan/chatty-frontend).

Built using Rust and Actix, it servers as the central server that manages each WebSocket connection and sends along
messages across users. The server has no access to a user's messages or private keys.

## Goals

- Deployed with some Frontend
- End-to-End Encryption
- No rooms, only Direct Messages (DMs)

## License

This project is licensed under GNU GPL v3.

Refer to COPYING for details.
