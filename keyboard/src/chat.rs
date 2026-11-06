use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;

const PORT: &str = "57007";

pub enum Direction {
    Inbound,
    Outbound,
}

pub struct Message {
    value: String,
    direction: Direction,
}

impl Message {
    pub fn new(message: &str) -> Self {
        Message {
            value: message.to_string(),
            direction: Direction::Outbound,
        }
    }

    pub fn get(&self) -> &str {
        return &self.value;
    }

    pub fn get_direction(&self) -> &Direction {
        return &self.direction;
    }
}

pub struct Peer {
    stream: Option<std::net::TcpStream>,
    peer: Option<SocketAddr>,

    sender: Sender<Message>,
    receiver: Receiver<Message>,

    waiting_message: Option<Message>,

    active: Arc<AtomicBool>,
}

impl Peer {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<Message>();
        let active = Arc::new(AtomicBool::new(false));

        Peer {
            stream: None,
            peer: None,
            sender,
            receiver,
            waiting_message: None,
            active,
        }
    }

    pub fn host(&mut self) -> std::io::Result<()> {
        assert!(!self.is_online());
        let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))?;
        listener.set_nonblocking(true)?;
        self.active.store(true, Ordering::SeqCst);

        let mut timeout = 0;
        while self.active.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, addr)) => {
                    stream.set_nonblocking(true)?;
                    self.stream = Some(stream);
                    self.peer = Some(addr);
                    // found our client, dont keep looking
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // no connection is ready right now, try again
                    timeout += 1;
                    if timeout == 10 {
                        self.active.store(false, Ordering::SeqCst);
                        break;
                    }
                    thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(_) => {
                    self.active.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }

        // client connected, create reading thread
        if self.active.load(Ordering::SeqCst) {
            self.create_reader()?;
        }
        Ok(())
    }

    pub fn connect(&mut self, addr: &str) -> std::io::Result<()> {
        assert!(!self.is_online());

        let addr_port = format!("{addr}:{PORT}");
        match TcpStream::connect(&addr_port) {
            Ok(stream) => {
                stream.set_nonblocking(true)?;
                self.stream = Some(stream);
                self.peer = Some(addr_port.parse().unwrap());
                self.active.store(true, Ordering::SeqCst);
            }
            Err(_) => {
                // couldnt connect
            }
        }

        // connected to server, create reading thread
        if self.active.load(Ordering::SeqCst) {
            return self.create_reader();
        }
        Ok(())
    }

    fn create_reader(&mut self) -> std::io::Result<()> {
        assert!(self.is_online());
        let mut stream_clone = self.stream.as_mut().unwrap().try_clone()?;
        let sender_clone = self.sender.clone();
        let active_clone = self.active.clone();
        thread::spawn(move || -> std::io::Result<()> {
            // have our connection, now read and write
            while active_clone.load(Ordering::SeqCst) {
                let mut buffer = [0; 512];
                match stream_clone.read(&mut buffer) {
                    Ok(0) => {
                        // 0 bytes read means the client gracefully disconnected
                        active_clone.store(false, Ordering::SeqCst);
                        break;
                    }
                    Ok(bytes_read) => {
                        let msg = String::from_utf8_lossy(&buffer[..bytes_read]);
                        let _ = sender_clone.send(Message {
                            value: msg.to_string(),
                            direction: Direction::Inbound,
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Client is still connected, but hasn't sent any new data yet
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => {
                        // A real connection error occurred (e.g., connection reset)
                        active_clone.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }

            Ok(())
        });
        Ok(())
    }

    pub fn is_online(&self) -> bool {
        if self.active.load(Ordering::SeqCst) {
            assert!(self.stream.is_some() && self.peer.is_some());
            return true;
        }
        return false;
    }

    pub fn send_message(&mut self, message: &Message) -> std::io::Result<()> {
        assert!(self.is_online());
        let mut writer_stream = self.stream.as_mut().unwrap().try_clone()?;
        let message_value = message.value.clone();
        thread::spawn(move || {
            let _ = writer_stream.write_all(message_value.as_bytes());
        });
        Ok(())
    }

    pub fn peek_message(&mut self) -> bool {
        assert!(self.waiting_message.is_none());
        match self.receiver.try_recv() {
            Ok(message) => {
                self.waiting_message = Some(message);
                return true;
            }
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => return false,
        }
    }

    pub fn pop_message(&mut self) -> Message {
        assert!(self.waiting_message.is_some());
        return self.waiting_message.take().unwrap();
    }

    pub fn get_peer(&self) -> String {
        if let Some(peer) = self.peer {
            return peer.to_string();
        } else {
            return "".to_string();
        }
    }

    pub fn reset_connection(&mut self) {
        self.active.store(false, Ordering::SeqCst);
        self.stream = None;
        self.peer = None;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        assert_eq, assert_ne,
        sync::{Condvar, Mutex},
        time::Duration,
    };

    use local_ip_address::linux::local_ip;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[derive(PartialEq)]
    enum Turn {
        Peer1,
        Peer2,
    }

    fn wait_for_message(peer: &mut Peer) -> Message {
        let mut timeout = 0;
        while !peer.peek_message() {
            thread::sleep(Duration::from_millis(100));
            timeout += 1;
            if timeout == 10 {
                break;
            }
        }
        assert_ne!(timeout, 10);
        return peer.pop_message();
    }

    #[test]
    fn test_chat() {
        let local_ip: std::net::IpAddr = local_ip().unwrap();
        let turn = Arc::new((Mutex::new(Turn::Peer2), Condvar::new()));

        let mut peer_1 = Peer::new();
        let turn_peer_1 = Arc::clone(&turn);
        let thread_1 = thread::spawn(move || {
            let _ = peer_1.host();
            assert!(peer_1.is_online());

            // wait for client to connect and send message
            {
                let (lock, cvar) = &*turn_peer_1;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer1 {
                    turn = cvar.wait(turn).unwrap();
                }

                let out_message = Message::new("Hello World!");
                let _ = peer_1.send_message(&out_message);

                *turn = Turn::Peer2;
                cvar.notify_one();
            }

            // wait for client to return message then reset connection
            {
                let (lock, cvar) = &*turn_peer_1;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer1 {
                    turn = cvar.wait(turn).unwrap();
                }

                let in_message = wait_for_message(&mut peer_1);
                assert_eq!(in_message.value, "Ack, Hello!");

                peer_1.reset_connection();
                assert!(!peer_1.is_online());

                *turn = Turn::Peer2;
                cvar.notify_one();
            }

            // wait for host to be available, then connect and send message
            {
                let (lock, cvar) = &*turn_peer_1;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer1 {
                    turn = cvar.wait(turn).unwrap();
                }

                // short sleep to make sure host is ready
                thread::sleep(std::time::Duration::from_millis(50));

                let _ = peer_1.connect(&local_ip.to_string());
                assert!(peer_1.is_online());

                let out_message = Message::new("Hello, Again...");
                let _ = peer_1.send_message(&out_message);

                *turn = Turn::Peer2;
                cvar.notify_one();
            }

            // receive return message
            {
                let (lock, cvar) = &*turn_peer_1;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer1 {
                    turn = cvar.wait(turn).unwrap();
                }

                let in_message = wait_for_message(&mut peer_1);
                assert_eq!(in_message.value, "yes...");
            }
        });

        let mut peer_2 = Peer::new();
        let turn_peer_2 = Arc::clone(&turn);
        let thread_2 = thread::spawn(move || {
            // wait for host then connect
            {
                let (lock, cvar) = &*turn_peer_2;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer2 {
                    turn = cvar.wait(turn).unwrap();
                }

                // short sleep to make sure host is ready
                thread::sleep(std::time::Duration::from_millis(50));

                let _ = peer_2.connect(&local_ip.to_string());
                assert!(peer_2.is_online());

                *turn = Turn::Peer1;
                cvar.notify_one();
            }

            // wait for message then respond
            {
                let (lock, cvar) = &*turn_peer_2;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer2 {
                    turn = cvar.wait(turn).unwrap();
                }

                let in_message = wait_for_message(&mut peer_2);
                assert_eq!(in_message.value, "Hello World!");

                let out_message = Message::new("Ack, Hello!");
                let _ = peer_2.send_message(&out_message);

                *turn = Turn::Peer1;
                cvar.notify_one();
            }

            // wait for message to be received, reset connection
            {
                let (lock, cvar) = &*turn_peer_2;
                let mut turn = lock.lock().unwrap();
                while *turn != Turn::Peer2 {
                    turn = cvar.wait(turn).unwrap();
                }
                peer_2.reset_connection();
                assert!(!peer_2.is_online());

                // must notify before hosting because host() blocks
                *turn = Turn::Peer1;
                cvar.notify_one();
            }

            // ..  and host
            let _ = peer_2.host();
            assert!(peer_2.is_online());

            // wait for client to connect and send message
            {
                let (lock, cvar) = &*turn_peer_2;
                let mut turn = lock.lock().unwrap();

                while *turn != Turn::Peer2 {
                    turn = cvar.wait(turn).unwrap();
                }

                let in_message = wait_for_message(&mut peer_2);
                assert_eq!(in_message.value, "Hello, Again...");

                let out_message = Message::new("yes...");
                let _ = peer_2.send_message(&out_message);
                *turn = Turn::Peer1;
                cvar.notify_one();
            }
        });

        thread_1.join().expect("peer 1 failed");
        thread_2.join().expect("peer 2 failed");
    }
}
