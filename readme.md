# Build

# RabbitMq stream 
## client
`cargo build --bin sync-client`
## server
`cargo build --bin sync-server`


# Nats jetstream
## client
`cargo build --bin sync-client-nats`
## server
`cargo build --bin sync-server-nats`
## SSE page
- `GET /`
- `GET /stream-2`

# Messaging Service
## RabbitMQ 
It needs advertised_host configuration in rabbitmq.conf when using docker
more info https://www.rabbitmq.com/blog/2021/07/23/connecting-to-streams#advertised-host-and-port
## Nats
nats jetstream

## How to run sse nats jetstream
- docker compose up
- run server bin then client bin, `cargo run --bin sync-server-nats` `cargo run --bin sync-client-nats`
- open page at `/` and `/stream-2`
- to publish a message, send POST to `/message`, payload `{id:number}`
