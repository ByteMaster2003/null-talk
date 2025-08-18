## Installation
### From binaries (recommended)
#### Linux (binary)
```
// Get the client binary from release
wget https://github.com/ByteMaster2003/null-talk/releases/download/v1.0.0/null-talk-linux-client-v1.0.0

// Move it to bin directory
mv null-talk-linux-client-v1.0.0 /usr/local/bin/null-talk

// Give permission to execute
chmod +x /usr/local/bin/null-talk

// Run the command
null-talk

```

#### MacOs (binary)
```
// Get the client binary from release
wget https://github.com/ByteMaster2003/null-talk/releases/download/v1.0.0/null-talk-macos-client-v1.0.0

// Move it to bin directory
mv null-talk-macos-client-v1.0.0 /usr/local/bin/null-talk

// Give permission to execute
chmod +x /usr/local/bin/null-talk

// Run the command
null-talk

```

## Configure server connection
- create a configuration file `config.toml`
```
hostname = "localhost" 		// domain or IpAddress of server
port = "8080"
name = "ByteMaster"

public_key = "~/.ssh/id_rsa.pub"
private_key = "~/.ssh/id_rsa"

```
- Now run `null-talk config.toml`
- If this file is not provided then `null-talk` will ask for it
![Null Talk Demo](assets/null-talk-config.png)

- When connection to the server is successful then this screen will appear
![Null Talk Connection Success](assets/null-talk-conn-success.png)

- If connection is failed then you will see this screen
![Null Talk Connection Error](assets/null-talk-conn-err.png)

- Any Error of Information will be shown at the bottom of main panel

## Layout
- This application has two panels `side_panel & main_panel`
- By default `main_panel` is active
- To switch panels we can use `⬅️ or h` and `➡️ or l`
- Panel switching will only work in `NORMAL` mode

## Modes
- This application has total 3 modes
- `NORMAL, INSERT and COMMAND`
- press `i` for insert mode
- press `/` for cmd mode
- press `esc` key for normal mode

## Available Commands in cmd Mode
- `cmd: q` or `ctrl + c` exists the application 
- `cmd: my-id` will show your user_id. 
- user_id is a unique hash of `public_key` provided by user
![Null Talk Show user_id](assets/show-user-id.png)
- `cmd: mkgp path/to/make-group.toml` creates new group

```
# required
name = "Cypher"

# optional, if not provided server will create new id
group_id = "53df4ec65397d404aa54ef7afda4005356a17388e49fb9e1859417af1ab45905"

# user_id of members, only these two member will be able to join the group
members = [
	"8e9c9db840b1b66e77ba817b5bf341fbadd71c9388e4768410d590983742881a",
	"bc7e780e2b26dec01fa791a617ef51bb93755f404a8ae150279aeb9d55d5ccab",
]

```
![Null Talk Show user_id](assets/make-group.png)
- `cmd: new path/to/session.toml` this will help us to `join group` or initiate `direct messages`
```
# [dm]			direct_message
# [group]		group_chat
# (required)
connection_type = "group"

# it can be username or group name
# (required)
name = "Cypher"

# group_id or user_id
id = "53df4ec65397d404aa54ef7afda4005356a17388e49fb9e1859417af1ab45905"

# encryption algorithm
# Supported algorithms: AES256, ChaCha20 --> default(AES256)
algo = "AES256"

```
![Null Talk Show user_id](assets/direct-message.png)
- If user is not online then we might get this error `Member is not online`


## Sessions
- to switch sessions press `esc` key for normal mode
- press `⬅️ or h` to select side panel
- and now use `⬇️ or j` and `⬆️ or k` to select some session
- now press enter to activate that session


# ⚠️ Current Limitations

### This is an early release, so a few features are not fully implemented yet:

- Encrypted private keys are not yet supported. Use unencrypted private keys when running the client.
- Messages are not persistent — once the application is terminated, all messages are lost.
- TLS integration and other security enhancements are planned for future versions.