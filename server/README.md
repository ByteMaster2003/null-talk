## Installation
### From binaries (recommended)
#### Linux (binary)
```
// Get the server binary from release
wget https://github.com/ByteMaster2003/null-talk/releases/download/v1.0.0/null-talk-linux-server-v1.0.0

// Move it to bin directory
mv null-talk-linux-server-v1.0.0 /usr/local/bin/null-talk-server

// Give permission to execute
chmod +x /usr/local/bin/null-talk-server
```

#### MacOs (binary)
```
// Get the server binary from release
wget https://github.com/ByteMaster2003/null-talk/releases/download/v1.0.0/null-talk-macos-server-v1.0.0

// Move it to bin directory
mv null-talk-macos-server-v1.0.0 /usr/local/bin/null-talk-server

// Give permission to execute
chmod +x /usr/local/bin/null-talk-server
```

## Run server
- for running server we would require one configuration file `/etc/null-talk/Config.toml`
- we can also configure the server for TLS
- if you don't need TLS then just comment out tls section
```
# /etc/null-talk/Config.toml
port = 8443

# optional
[tls]
cert_path = "/etc/letsencrypt/live/example.com/fullchain.pem"
key_path = "/etc/letsencrypt/live/example.com/privkey.pem"

```
- run the server 
```
$ sudo null-talk-server
```
