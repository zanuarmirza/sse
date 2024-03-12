# Build
## client
`cargo build --bin sync-client`
## client nats
`cargo build --bin sync-client-nats`

## server
`cargo build --bin sync-server`
## server-nats
`cargo build --bin sync-server-nats`

# Messaging Service
## RabbitMQ docker config
It needs advertised_host configuration in rabbitmq.conf . 
more info https://www.rabbitmq.com/blog/2021/07/23/connecting-to-streams#advertised-host-and-port
## Nats
nats jetstream
