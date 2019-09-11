# test
test actix project

## run the server with:

```
$ cargo run --bin server
```

## and the client: 
```
$ cargo run --bin client
```

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


