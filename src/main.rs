extern crate redis;
use redis::Commands;

mod book;
use book::*;

use std::collections::HashMap;

fn redis_connect() -> redis::RedisResult<redis::Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let con = client.get_connection()?;

    return Ok(con);
}

/// Update order book information summary into redis
fn update_info(redis_con: &redis::Connection, order_book: &OrderBook) -> redis::RedisResult<()> {
    redis_con.set("bid", order_book.bid_max())?;
    redis_con.set("ask", order_book.ask_min())?;
    Ok(())
}

/// Convert a string "amount:price:user_id:order_id" to an Order struct
fn parse_order(s: &String) -> Option<Order> {
    let v : Vec<&str> = s.split(":").collect();
    if v.len() < 3 { return None; }

    let a = v[0].parse();
    let p = v[1].parse();
    let u = v[2].parse();
    let o = v[3].parse();

    if a.is_err() || p.is_err() || u.is_err() { return None; }

    Some(Order { amount: a.unwrap(), price: p.unwrap(), user_id: u.unwrap(), order_id: o.unwrap()})
}

/// Consume the available order from redis
fn consume_redis_value(redis_con: &redis::Connection, order_book: &mut OrderBook) -> Option<()> {
    let r : HashMap<String, String> = redis::cmd("blpop").arg("orders").arg(10).query(redis_con).ok()?;
    let order = parse_order(&r.iter().next()?.1);

    order.map(|o| order_book.insert_order(o))?;
    order_book.sync(redis_con);
    Some(())
}

fn main() {
    let redis_con = redis_connect()
        .expect("Can't connect to redis");
    let mut order_book = OrderBook::new();

    println!("Starting cda engine...");
    loop {
        // Wait on the redis QUEUE and consume
        consume_redis_value(&redis_con, &mut order_book);
        println!("DEBUG: {:?}", order_book);
        // Update the order book summary
        let _ = update_info(&redis_con, &order_book);
    }
}
