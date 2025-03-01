# MordorWide UDP TURN Server

This application is used to relay UDP packets from NAT-restricted clients behind a firewall. This enables the hosting of servers even in cases of symmetric NAT-restricted connections, at the expense of a substantially increased ping.

Whether TURN needs to be applied depends on the connection. If the public UDP port matches the advertised UDP port of a THEATER connection, no TURN is enabled on the server.

Inactive relays are dropped after 30 minutes.

## How to Run
### Dev Mode
If the turn server should be launched in development mode without containerization, do the following steps.
1. Ensure that Rust and Cargo are installed and up to date.
2. Update the environmental variables in `env.standalone` and load it via `ENV_ARGS="$(grep -v '^#' env.standalone  | grep -v '^$' | tr '\n' ' '  )"`.
3. Run `eval "$ARGS cargo run"` to build and run the turn server with the environmental variables set.

### Container Mode
1. Make sure that Docker or Podman is installed.
2. Build the image using `docker compose -f docker-compose.standalone.yml build`.
3. Update the environmental variables in `env.standalone`.
4. Run the standalone container with `docker compose --env-file env.standalone -f docker-compose.standalone.yml up -d`
5. Check the logs via `docker logs -f mordorwide-udpturn`
6. Stop the standalone server again with `docker compose --env-file env.standalone -f docker-compose.standalone.yml down -v`

## Client Testing
The TURN connection can be tested in one direction as follows.
```
# Make sure that the TURN server is running...

# Run the receiving UDP endpoint at port 9999 in a second terminal window.
nc -ul 9999

# Configure the TURN server to relay port 8888 to 9999 (and vice versa)
curl -X POST -H "Content-Type: application/json" \
    --data '{"client_ip_0": "127.0.0.1", "client_port_0": 8888, "client_ip_1": "127.0.0.1", "client_port_1": 9999}' \
    http://localhost:8080/launch

# The result should look like this:
# $> {"success":true,"relay_port_0": <port0>,"relay_port_1":<port1> }

# Send the UDP message via netcat to the second terminal window (8888 should be the source port).
echo "Test Message" | nc -up 8888 127.0.0.1 <port0>

# The netcat receiver endpoint should show "Test Message" in the second terminal window.
```