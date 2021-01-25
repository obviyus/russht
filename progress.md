# 2021-01-24
- Start small, worry about Docker, deployment and setup later
- Things to consider: (priority-wise)
    - reading SSH logs 
        - `auth.log`, `/var/log/secure` -> not distro agnostic
        - understand how the sshd logs are structured: https://en.wikibooks.org/wiki/OpenSSH/Logging_and_Troubleshooting
        - read every time file is modified 
        - only care about the diff from last read
    - parsing SSH logs
        - read logs line by line
        - isolate info (time, user, location etc.)
        - https://andre.arko.net/2018/10/25/parsing-logs-230x-faster-with-rust/
    - collecting logs 
        - collect relevant data in a struct

Since I want to write this in Rust, these (naive) approaches might not be the best move. Worth asking around in **##rust**.

# 2021-01-25
- IRC folks mentioned giving `journalctl` a look: https://www.freedesktop.org/software/systemd/man/journalctl.html
-  Piping the output of `journalctl -f` and processing `BufReader` as it updates seems reasonable enough 
-  `regex` to capture relevant info and dump into a struct
  
Rough idea of the approach I have in mind:
- serialize captured info into a JSON body
- HTTP PUT to `AlphaServer` 
- `AlphaServer` could be written in Go, accepts and parses JSON inputs 
- Advantages:
  - each `AlphaClient` is just another HTTP PUT request to the Go server 
  - allows extreme felxibility: parsing in Rust -> collection with Go -> data viz in Python all through simple HTTP requests 
  - no special implementations required 

# 2021-01-26
- Rather than messing around with `regex` on raw `journalctl` outputs and THEN converting to a JSON, `journalctl` provied a `json` flag which would simplify matters. The Journal JSON Format spec: https://www.freedesktop.org/wiki/Software/systemd/json/