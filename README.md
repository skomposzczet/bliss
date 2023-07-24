# bliss
Bliss is Lemmy Instance Settings Synchronizer

## Installation
1. Clone repo
    ```bash
    git clone https://github.com/skomposzczet/bliss.git
    ```
1. Install app
    ```bash
    cd bliss && cargo install --path .
    ```
## Usage
For more detailed help run:
```bash
bliss --help
```
### Subcommands
- Pull lemmy account settings to local profile
    ```bash
    bliss pull -u <username or email> -i <instance url> -p <local profile name>
    ```
- Push local profile settings to lemmy account
    ```bash
    bliss push -u <username or email> -i <instance url> -p <local profile name>
    ```
### Password
Bliss will search for password in environment variables `LEMMY_SRC_PW` and `LEMMY_DST_PW`. If unsuccessfully it will prompt user.
## Backlog
- [X] general sync
- [X] allow user to select settings to not sync (i.e. email)
- [X] fullsync - unblock/unfollow communities/users that are not blocked/followed in source account (local profile)
- [X] support 2fa login
- [ ] change local profile path
- [ ] sync avatar + banner
