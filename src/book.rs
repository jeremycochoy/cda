extern crate redis;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::cmp::min;

#[derive(Default, Clone, Debug)]
pub struct Order {
    pub amount:  i64,
    pub price:   u32,
    pub user_id: u64,
    pub order_id: u32,
}

#[derive(Default, Debug)]
/// The order book should maintain non empty VecDeque.
/// If an entry exist in the bid or ask BTreeMap, it is expected
/// that the VecDeque associated is non empty.
pub struct OrderBook {
    pub bid: BTreeMap<u32, VecDeque<Order>>,
    pub ask: BTreeMap<u32, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook::default()
    }

    /// Matching algorithm
    pub fn sync(&mut self, redis_con: &redis::Connection) -> Option<()> {
        while self.bid_max() >= self.ask_min() {
            println!("WTF: {} >= {}", self.bid_max(), self.ask_min());
            let mut bid = self.bid_pop()?;
            let mut ask = self.ask_pop()?;

            println!("Match BID:{:?} zith ASK:{:?}", bid, ask);
            // TODO: Taker VS Maker...

            let amount = min(ask.amount, -bid.amount);

            if ask.amount < -bid.amount {
                bid.amount += ask.amount;
                self.unpop_order(bid.clone());
            }
            else if ask.amount > -bid.amount {
                ask.amount += bid.amount;
                self.unpop_order(ask.clone());
            }
            else { // ask == bid
            }

            log_order(redis_con, "fills_bid", &Order { amount, ..bid });
            log_order(redis_con, "fills_ask", &Order { amount, ..ask });
        }
        Some(())
    }

    pub fn bid_max(&self) -> u32 {
        *self.bid.iter().next_back()
            .map(|(k, _)| k)
            .unwrap_or(&std::u32::MIN)
    }

    pub fn ask_min(&self) -> u32 {
        *self.ask.iter().next()
            .map(|(k, _)| k)
            .unwrap_or(&std::u32::MAX)
    }

    pub fn bid_pop(&mut self) -> Option<Order> {
        let key = self.bid_max();

        let deque = self.bid.get_mut(&key)?;
        let order = deque.pop_front()?;

        if deque.is_empty() { self.bid.remove(&key); }
        Some(order)
    }

    pub fn ask_pop(&mut self) -> Option<Order> {
        let key = self.ask_min();

        let deque = self.ask.get_mut(&key)?;
        let order = deque.pop_front()?;

        if deque.is_empty() { self.ask.remove(&key); }
        Some(order)
    }

    pub fn insert_order(&mut self, order: Order) {
        // Insert order
        let hash = match order.amount {
            // Buy
            v if v > 0 => &mut self.ask,
            // Sell
            v if v < 0 => &mut self.bid,
            // 0 Quantity order are just ignored
            _ => return (),
        };

        if !hash.contains_key(&order.price) {
            hash.insert(order.price, VecDeque::default());
        }
        hash.get_mut(&order.price).unwrap().push_back(order);
    }

    pub fn unpop_order(&mut self, order: Order) {
        // Insert order
        let hash = match order.amount {
            // Buy
            v if v > 0 => &mut self.ask,
            // Sell
            v if v < 0 => &mut self.bid,
            // 0 Quantity order are just ignored
            _ => return (),
        };

        if !hash.contains_key(&order.price) {
            hash.insert(order.price, VecDeque::default());
        }
        hash.get_mut(&order.price).unwrap().push_front(order);
    }
}

pub fn log_order(redis_con: &redis::Connection, queue: &str, o: &Order) {
    let _ : Option<()> = redis::cmd("rpush")
        .arg(queue)
        .arg(format!("{}:{}:{}:{}", o.amount, o.price, o.user_id, o.order_id))
        .query(redis_con).ok();
}

