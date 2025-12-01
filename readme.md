# Ale's Interlude


## server 
- Accepts unidirectional streams 
- streams max size is 1k 
- one task per incoming connection
- on each task, pop uni streams and read them with read_chunks asap 

## Client 
- Open multiple connections 
- one task per connection 
- one each connection, open_uni(), write 1k in a loop asap
