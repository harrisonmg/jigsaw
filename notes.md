# TODO
* side shader
* frame, piece lock, random starting pos
* outline at low z height
* check wasm performance
* transform and rebroadcast cursor move event?
    * server cursor position (cursor pos)
    * move piece (cursor delta)
    * proper zoom (cursor delta)
    * pan camera (cursor pos)

# architecture
- server
    - tokio, warp
    - HTTPS REST API
        - auth server admin
        - load new image, cut and start puzzle
        - load a game state
        - list and kick clients
    - serves wasm client
    - communicate with clients via websockets
        - new connection
            - client sends name and cursor color
            - server sends game state
        - accept actions
            - pickup
            - move
            - place
            - cursor location
        - deny actions due to conflict
            - reply with correct piece location and status
        - broadcast updates to other clients
        - automatically put down piece or group on disconnect
    - game logic that occurs server side
        - piece connections checked by server on piece place

- client
    - bevy, tungstenite, wasm
    - connect to server
    - download state
    - given a state, render the game
    - resiable reference image panel
    - shader for piece outline
    - display a little controls help box
    - accept player input
    - rollback-like state
        - render based on state + player input
        - apply updates from server regardless of player input

- game
    - serde, rmp_serde
    - message types
    - serializable data structures
        - game state
            - game constants
                - piece size
                - puzzle size
                - full image
            - pieces
                - piece type
                - pose
                - image indices
                - sprite
            - groups
            - players
                - uuid
                - name
                - cursor
                    - color
                    - position
    - connection and grouping logic

- cutter
    - take an image and a piece count
    - cut that baby up into pieces
    - return a bunch of images and their image indices
        - ideally in game state data structure form

- admin interface
    - native app that runs the puzzle cutter and creates a new game state
    - interfaces with server admin API
