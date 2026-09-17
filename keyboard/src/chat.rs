use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;

#[derive(Clone)]
pub struct Message {
    value: String,
}

impl Message {
    pub fn new(message: &str) -> Self {
        Message {
            value: message.to_string(),
        }
    }

    pub fn get(&self) -> &String {
        return &self.value;
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
        let listener = TcpListener::bind("127.0.0.1:57007")?;
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
                        break;
                    }
                    thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(_) => {
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
        self.active.store(true, Ordering::SeqCst);
        self.stream = Some(TcpStream::connect(addr)?);
        self.peer = Some(addr.parse().unwrap());
        self.stream.as_mut().unwrap().set_nonblocking(true)?;

        // connected to server, create reading thread
        return self.create_reader();
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
        let message_clone = message.clone();
        thread::spawn(move || {
            let _ = writer_stream.write_all(message_clone.value.as_bytes());
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
