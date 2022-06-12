# Termtalk Summary
Termtalk functions as a chat server with a rudimentary CLI for having conversations online that are not persisted to a database. The supported functionality of Termtalk is registering a user, logging in and then chatting with multiple users in a room, one user through a direct chat, or whispering to one user regardless of which room or direct chat they’re currently in.

Actix Web framework is used due to its support of web sockets which allow Termtalk API to maintain the connections of users logged in and sending messages. Elasticsearch is used for storing users information and encrypted passwords. The reason I opted for Elasticsearch is I hoped to support advanced search functionality at a later point when messages would be persisted - Elasticsearch has very powerful full-text search features. Redis is used as a queuing mechanism for chat messages. The reason I opted to use Redis as a queuing mechanism for chat messages is so that at a later point if Termtalk API scales to multiple instances, Redis could broadcast incoming chat messages to multiple Termtalk API instances and the relevant instance storing the receiving users session could pick up and deliver the chat message to that user connection. This is documented in the system architecture diagram below. I had a strong desire to understand exactly how JWT Tokens work and decided to implement my own JWT Token struct as opposed to downloading a crate.

Future improvements for Termtalk include containerizing the application, developing the application and all of its components using Kubernetes minikube and Skaffold architecture, and persisting messages to Elasticsearch via a cron job as shown in the below diagram that would throttle the frequency of writes to prevent overloading Elasticsearch. Persisting messages would segue into developing search functionality for users to search messages from the past. For loading the most recent previous messages for a given room/direct chat Redis would again be used for caching to limit the number of calls to Elasticsearch as much as possible.

# Starting the Application
Termtalk API must be running in order for Termtalk to work. Must have Elasticsearch and Redis running on the same machine that Termtalk API is meant to run on. After installing Elasticsearch cd into the `elastic-manager` directory and run `cargo run` to create the necessary elastic search indexes with their respective mappings. Start Elasticsearch and Redis and configure the following env vars by creating a `.env` file inside of the `termtalk-api` dir:
```
SECRET_KEY=some_secret_for_jwt_tokens
RUST_LOG=debug
RUST_LOG.actix_web=debug
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
TERMTALK_API_HOST=127.0.0.1
TERMTALK_API_PORT=8080
```

As a note, Elasticsearch is expected to be running on `http://localhost:9200`.

Once Elasticsearch and Redis are running, and you've created your `.env` file inside of `termtalk-api` dir you can run Termtalk API by running `cargo run` inside of the `termtalk-api` dir.

From a separate terminal window you can cd into `termtalk-cli` and execute `cargo run` to register, login, and start chatting inside of Termtalk.

# Termtalk System Design Diagram
![alt text](https://github.com/mektievp/termtalk/blob/master/docs/termtalk-system-design.png?raw=true)
