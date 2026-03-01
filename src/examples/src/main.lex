// =====================================================
// Lexicon Database & Socket Connection Examples
// =====================================================
// This file demonstrates how to use database connections
// and socket communication in Lexicon
// =====================================================

import core.io.Console;
import core.json.Json;

// =====================================================
// DATABASE MODULE
// =====================================================
// The Db module provides connection to PostgreSQL, MySQL, and SQLite
// 
// Connection String Formats:
// - SQLite:  "sqlite", "database.db"
// - PostgreSQL: "postgresql", "host", port, "database", "user", "password"  
// - MySQL: "mysql", "host", port, "database", "user", "password"
// =====================================================

module Database {
    
    // Example 1: Connect to SQLite (local file)
    // fn connectSqlite(name: String, path: String) -> DbConnection
    // Usage:
    //   let db = Database::connect("mydb", "sqlite", "./data.db");
    
    // Example 2: Connect to PostgreSQL
    // fn connectPostgres(name: String, config: DbConfig) -> DbConnection
    // Usage:
    //   let config = {
    //       engine: "postgresql",
    //       host: "localhost",
    //       port: 5432,
    //       database: "myapp",
    //       username: "admin",
    //       password: "secret"
    //   };
    //   let db = Database::connect("prod_db", config);
    
    // Example 3: Connect to MySQL
    // fn connectMysql(name: String, config: DbConfig) -> DbConnection
    // Usage:
    //   let db = Database::connect("mysql_db", "localhost", 3306, "shop", "root", "pass");
    
    // Query Methods:
    //   db.query(sql: String) -> QueryResult
    //   db.execute(sql: String) -> u64 (affected rows)
    //   db.disconnect() -> void
    
    // QueryResult structure:
    //   result.columns: [String]    - Column names
    //   result.rows: [[Value]]      - Data rows
    //   result.affectedRows: u64    - Number of affected rows
}

fn demoDatabasePattern() -> void {
    Console.writeLine("\n=== Database Pattern Demo ===");
    
    // 1. Connect to database
    // let db = Db::connect("app", "sqlite", "app.db");
    
    // 2. Create tables
    // db.execute(`
    //     CREATE TABLE IF NOT EXISTS users (
    //         id INTEGER PRIMARY KEY AUTOINCREMENT,
    //         name TEXT NOT NULL,
    //         email TEXT UNIQUE,
    //         created_at TEXT DEFAULT CURRENT_TIMESTAMP
    //     )
    // `);
    
    // 3. Insert data
    // let affected = db.execute("INSERT INTO users (name, email) VALUES ('John', 'john@email.com')");
    // Console.writeLine("Inserted " + affected + " row(s)");
    
    // 4. Query data
    // let result = db.query("SELECT id, name, email FROM users WHERE name LIKE 'J%'");
    // 
    // Console.writeLine("Columns: " + result.columns.join(", "));
    // for row in result.rows {
    //     Console.writeLine("  " + row[0] + ": " + row[1] + " (" + row[2] + ")");
    // }
    
    // 5. Update data
    // let updated = db.execute("UPDATE users SET email = 'new@email.com' WHERE name = 'John'");
    // Console.writeLine("Updated " + updated + " row(s)");
    
    // 6. Delete data
    // let deleted = db.execute("DELETE FROM users WHERE name = 'John'");
    // Console.writeLine("Deleted " + deleted + " row(s)");
    
    // 7. Disconnect
    // db.disconnect();
    
    Console.writeLine("Database pattern demonstrated!");
}

// =====================================================
// SOCKET MODULE  
// =====================================================
// The Socket module provides TCP and UDP socket communication
//
// TCP Server:
//   let server = Socket::tcpServer(port: String) -> TcpServer
//   server.accept() -> TcpClient
//
// TCP Client:
//   let client = Socket::tcpConnect(host: String, port: String) -> TcpClient
//   client.send(data: String) -> void
//   client.receive(bufferSize: number) -> String
//   client.disconnect() -> void
//
// UDP Socket:
//   let udp = Socket::udpBind(port: String) -> UdpSocket
//   udp.sendTo(data: String, host: String, port: String) -> void
//   udp.receive(bufferSize: number) -> (String, String) // (message, sender address)
// =====================================================

module SocketDemo {
    
    // TCP Server Pattern
    fn demoTcpServer() -> void {
        Console.writeLine("\n=== TCP Server Pattern ===");
        
        // Start server on port 8080
        // let server = Socket::tcpServer("8080");
        // Console.writeLine("Server listening on 0.0.0.0:8080");
        
        // Accept connections in loop
        // loop {
        //     let client = server.accept();
        //     Console.writeLine("Client connected: " + client.remoteAddr());
        //     
        //     // Handle client in separate task
        //     spawn handleClient(client);
        // }
        
        // fn handleClient(client: TcpClient) -> void {
        //     let request = client.receive(4096);
        //     let response = processRequest(request);
        //     client.send(response);
        //     client.disconnect();
        // }
        
        Console.writeLine("TCP Server would listen on 0.0.0.0:8080");
    }
    
    // TCP Client Pattern
    fn demoTcpClient() -> void {
        Console.writeLine("\n=== TCP Client Pattern ===");
        
        // Connect to server
        // let client = Socket::tcpConnect("localhost", "8080");
        
        // Send request
        // let request = Json::stringify({
        //     action: "getUser",
        //     id: 123
        // });
        // client.send(request);
        
        // Receive response
        // let response = client.receive(4096);
        // let data = Json::parse(response);
        // 
        // Console.writeLine("Received: " + data);
        // 
        // // Disconnect
        // client.disconnect();
        
        Console.writeLine("TCP Client would connect to localhost:8080");
    }
    
    // UDP Pattern
    fn demoUdp() -> void {
        Console.writeLine("\n=== UDP Pattern ===");
        
        // Bind UDP socket
        // let udp = Socket::udpBind("9000");
        // Console.writeLine("UDP socket bound to 0.0.0.0:9000");
        // 
        // // Receive messages
        // loop {
        //     let (message, address) = udp.receive(1024);
        //     Console.writeLine("From " + address + ": " + message);
        //     
        //     // Respond
        //     udp.sendTo("ACK: " + message, address.split(":")[0], address.split(":")[1]);
        // }
        
        // // Or send broadcast
        // udp.sendTo("Hello everyone!", "255.255.255.255", "9000");
        
        Console.writeLine("UDP would bind to 0.0.0.0:9000");
    }
    
    // WebSocket Pattern (using TCP)
    fn demoWebSocket() -> void {
        Console.writeLine("\n=== WebSocket Pattern ===");
        
        // WebSocket uses TCP connection with upgrade
        // 
        // Server:
        // let server = Socket::tcpServer("8080");
        // let client = server.accept();
        // 
        // // Handle WebSocket handshake
        // let key = extractSecWebSocketKey(client.receive());
        // let response = buildWebSocketAccept(key);
        // client.send(response);
        // 
        // // Now communicate with WebSocket frames
        // loop {
        //     let frame = client.receive(4096);
        //     let message = decodeWebSocketFrame(frame);
        //     let response = processMessage(message);
        //     client.send(encodeWebSocketFrame(response));
        // }
        
        Console.writeLine("WebSocket uses TCP with handshake");
    }
}

// =====================================================
// MAIN FUNCTION
// =====================================================

pub fn main() -> void {
    Console.writeLine("╔════════════════════════════════════════╗");
    Console.writeLine("║  Lexicon Database & Socket Examples   ║");
    Console.writeLine("╚════════════════════════════════════════╝");
    
    // Run all demos
    demoDatabasePattern();
    SocketDemo::demoTcpServer();
    SocketDemo::demoTcpClient();
    SocketDemo::demoUdp();
    SocketDemo::demoWebSocket();
    
    Console.writeLine("\n=== Complete ===");
    Console.writeLine("See code comments for actual API usage.");
}

// =====================================================
// ADDITIONAL PATTERNS
// =====================================================

// Connection Pool (for production):
// fn getDb() -> DbConnection {
//     static pool = ConnectionPool::new("postgresql", "localhost", 5432, "mydb", "user", "pass", 10);
//     return pool.getConnection();
// }
//
// // Use in request handler
// fn handleRequest(req) -> Response {
//     let db = getDb();
//     defer db.release();
//     
//     let users = db.query("SELECT * FROM users");
//     return Response::ok(users);
// }

// ORM-like pattern:
// fn User_findById(id: number) -> User? {
//     let db = getDb();
//     let result = db.query("SELECT * FROM users WHERE id = " + id);
//     if result.rows.length > 0 {
//         return User_fromRow(result.rows[0]);
//     }
//     return null;
// }
//
// fn User_save(user: User) -> void {
//     let db = getDb();
//     if user.id > 0 {
//         db.execute("UPDATE users SET name = ? WHERE id = ?", user.name, user.id);
//     } else {
//         db.execute("INSERT INTO users (name) VALUES (?)", user.name);
//     }
// }

// Message Queue (using UDP or TCP):
// fn sendMessage(queue: String, message: String) -> void {
//     let socket = Socket::udpBind("0");
//     socket.sendTo(message, "localhost", getQueuePort(queue));
// }
//
// fn receiveMessages(queue: String, handler: fn(String) -> void) -> void {
//     let socket = Socket::udpBind(getQueuePort(queue));
//     loop {
//         let (msg, _) = socket.receive(4096);
//         handler(msg);
//     }
// }

// Real-time updates (WebSocket):
// fn broadcastToClients(event: String, data: String) -> void {
//     static clients = [];
//     for client in clients {
//         client.send(Json::stringify({ event, data }));
//     }
// }
