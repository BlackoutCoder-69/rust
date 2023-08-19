use std::collections::HashMap;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

struct Stock {
    base_price: f64,
    stock_security: i32,
    profit: f64,
    current_bid: f64,
    bidder: String,
    bid_time: Instant,
    bid_log: Vec<String>,
}

impl Stock {
    fn new(base_price: f64, stock_security: i32, profit: f64) -> Self {
        Self {
            base_price,
            stock_security,
            profit,
            current_bid: base_price,
            bidder: String::new(),
            bid_time: Instant::now(),
            bid_log: Vec::new(),
        }
    }
}

fn countdown(mut t: u64) {
    while t > 0 {
        print!("\r{:02}:{:02}", t / 60, t % 60);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
        t -= 1;
    }
    println!("\rSuccessfully placed a bid!!!");
}

fn handle_client(mut stream: TcpStream, stocks: &mut HashMap<String, Stock>) {
    let mut buffer = [0; 1024];
    let addr = stream.peer_addr().unwrap();

    println!("New connection from {:?}", addr);

    while let Ok(n) = stream.write(b"Enter ID: ") {
        let _ = stream.read(&mut buffer[..n]);

        let log_id = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

        if log_id.is_empty() {
            println!("No ID provided by {:?}", addr);
            break;
        }

        println!("Logged in: {}", log_id);

        while let Ok(n) = stream.write(b"Enter stock name: ") {
            let _ = stream.read(&mut buffer[..n]);
            let stock_symbol = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

            if stock_symbol == "quit" {
                break;
            }

            if let Some(stock) = stocks.get_mut(&stock_symbol) {
                let bid_amount_prompt = format!("Enter bid amount {}: ", stock_symbol);
                stream.write(bid_amount_prompt.as_bytes()).unwrap();

                let mut bid_amount_buf = [0; 1024];
                let _ = stream.read(&mut bid_amount_buf);
                let bid_amount = String::from_utf8_lossy(&bid_amount_buf).trim().parse::<f64>().unwrap();

                let security_code_prompt = format!("Enter security code {}: ", stock_symbol);
                stream.write(security_code_prompt.as_bytes()).unwrap();

                let mut security_code_buf = [0; 1024];
                let _ = stream.read(&mut security_code_buf);
                let entered_security_code = String::from_utf8_lossy(&security_code_buf).trim().to_string();

                if entered_security_code == stock.stock_security.to_string() {
                    if bid_amount > stock.current_bid {
                        stock.current_bid = bid_amount;
                        stock.bidder = log_id.clone();
                        stock.bid_time = Instant::now();
                        let response = format!("Bid placed successfully. Current highest bid: {}", stock.current_bid);
                        notify_clients(&mut stream, response);
                    } else {
                        let response = "Your offer is not greater than the current highest offer.".to_string();
                        stream.write(response.as_bytes()).unwrap();
                    }
                } else {
                    let response = "Invalid security code".to_string();
                    stream.write(response.as_bytes()).unwrap();
                }

                if stock.bid_time.elapsed().as_secs() < 300 {
                    let countdown_time = 300 - stock.bid_time.elapsed().as_secs();
                    let countdown_response = format!("Waiting for {} minutes....", countdown_time / 60);
                    stream.write(countdown_response.as_bytes()).unwrap();

                    let countdown_thread = thread::spawn(move || countdown(countdown_time));
                    countdown_thread.join().unwrap();

                    if stock.bid_time.elapsed().as_secs() < 300 {
                        let additional_time_response = "Additional time: 60 seconds".to_string();
                        stream.write(additional_time_response.as_bytes()).unwrap();
                    }
                }
            } else {
                let response = format!("{} not found", stock_symbol);
                stream.write(response.as_bytes()).unwrap();
            }
        }
    }

    println!("Connection from {:?} closed", addr);
}

fn notify_clients(stream: &mut TcpStream, message: String) {
    // Notify all connected clients
    // (You may need to maintain a list of connected clients here)
    // Iterate through the list of clients' streams and send the message to each
}

fn main() -> io::Result<()> {
    let mut stocks: HashMap<String, Stock> = HashMap::new();

    // Read the stock information from the CSV file using a CSV library
    // Populate the `stocks` HashMap

    let listener = TcpListener::bind("localhost:2022")?;
    println!("Server started. Listening on port 2022...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stocks_ref = &mut stocks;
                thread::spawn(move || handle_client(stream, stocks_ref));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
