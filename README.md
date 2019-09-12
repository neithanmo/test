# test
test actix project

## run the server with:

```
$ cargo run --bin server -- -p 8080
```
the above command will start a websocket server on localhost and port 8080
## and the client: 
```
$ cargo run --bin client -- -i 127.0.0.1 -p 8080
```
Creates a client which would try to connect to http://127.0.0.1:8080/ws/
if the client's address is whitelisted, it will be connected

As an authorize client,  You can add a client to the server's whitelist with the next command:
```
$ ADD x.x.x.x 
```
Also, authorized clients can remove others from the whitelist:
```
$ REMOVE x.x.x.x
```

## list of authorized clients
http://localhost:8080/


