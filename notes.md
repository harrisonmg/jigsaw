# TODO

- winit (or fork) might disable context menu by default now

- cursors
- server admin features

- throttle move piece messages?
    - load test first

- server holds multiple connections
    - for each connection
        - send puzzle
        - mpsc event sink
        - spmc event source

- server doesn't echo any client events
    - add client id to event in server to avoid this
    - connection movements are clientless so they do get sent

- if client receives any event with held piece, piece is released
    - don't need to send an event to the server for that release

- game holds list of "held" pieces for animation and to deny user from picking up already held piece
    - held piece linked to client id
    - cursors linked to client id

- events
    - player connected
    - piece pick up
    - piece put down
    - move piece
    - move cursor
    - piece connection
        - server checks connection on piece put down
        - sends resulting piece movements, then sends connection event with piece index
        - client checks connection but does not forward movements - there should be no change anyway
