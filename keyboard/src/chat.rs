use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;

struct Message {
    value: String,
}

enum PeerType {
    None,
    Server,
    Client,
}

struct Peer {
    stream: Option<std::net::TcpStream>,
    peer: Option<SocketAddr>,

    sender: Sender<Message>,
    receiver: Receiver<Message>,

    waiting_message: Option<Message>,

    active: Arc<AtomicBool>,
}

impl Peer {
    fn new() -> Self {
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

    fn host(&mut self) -> std::io::Result<()> {
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
        if self.stream.is_some() {
            self.create_reader()?;
        }
        Ok(())
    }

    fn connect(&mut self, addr: &str) -> std::io::Result<()> {
        assert!(!self.is_online());
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

    fn send_message(&mut self, message: Message) -> std::io::Result<()> {
        assert!(self.is_online());
        let mut writer_stream = self.stream.as_mut().unwrap().try_clone()?;
        thread::spawn(move || {
            let _ = writer_stream.write_all(message.value.as_bytes());
        });
        Ok(())
    }

    fn is_online(&self) -> bool {
        return self.stream.is_some() && self.peer.is_some();
    }

    fn peek_message(&mut self) -> bool {
        assert!(!self.is_online());
        assert!(self.waiting_message.is_none());
        match self.receiver.try_recv() {
            Ok(message) => {
                self.waiting_message = Some(message);
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => false,
        }
    }

    fn pop_message(&mut self) -> Message {
        assert!(!self.is_online());
        assert!(self.waiting_message.is_some());
        return self.waiting_message.take().unwrap();
    }
}
