use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{prelude::*, BufReader, ErrorKind};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};

pub struct Pipe {
    conn: BufReader<LocalSocketStream>,
    buffer: [u8; 1024],
}

impl Pipe {
    pub fn new(conn: LocalSocketStream) -> Self {
        let conn = BufReader::new(conn);
        let buffer = [0; 1024];
        Pipe { conn, buffer }
    }

    fn do_write<T: Serialize>(&mut self, data: &T) -> Result<()> {
        let data = postcard::to_allocvec(data)?;

        let len = data.len();
        let (len, chunks_len) = (len, len / 1024 + if len % 1024 == 0 { 0 } else { 1 });
        self.conn
            .get_mut()
            .write_all(&postcard::to_allocvec(&(len, chunks_len))?)?;
        self.conn.get_mut().flush()?;

        for chunk in data.chunks(1024) {
            self.conn.get_mut().write_all(chunk)?;
            self.conn.get_mut().flush()?;
        }

        let ack = "ACK".as_bytes();
        self.conn.get_mut().write_all(&ack)?;
        self.conn.get_mut().flush()?;

        Ok(())
    }

    fn do_read<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T> {
        self.conn.read(&mut self.buffer)?;
        let (len, chunks_len): (usize, usize) = postcard::from_bytes(&self.buffer)?;
        self.buffer = [0; 1024];

        let mut data = Vec::with_capacity(len);

        for _ in 0..chunks_len {
            self.conn.read(&mut self.buffer)?;
            data.extend_from_slice(&self.buffer);

            self.buffer = [0; 1024];
        }

        self.conn.read(&mut self.buffer)?;
        if &self.buffer[0..3] != "ACK".as_bytes() {
            return Err(anyhow!("No ACK"));
        }
        self.buffer = [0; 1024];

        Ok(postcard::from_bytes(&data[0..len])?)
    }

    pub fn write<T: Serialize>(&mut self, data: &T) -> Result<()> {
        self.do_write(data).map_err(|err| {
            println!("Pipe failed to write: {:?}", err);
            err
        })
    }

    pub fn read<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T> {
        self.do_read().map_err(|err| {
            println!("Pipe failed to read: {:?}", err);
            err
        })
    }
}

pub fn create_client(name: String) -> Result<Pipe> {
    let name = {
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => format!("/tmp/{name}.sock"),
            OnlyNamespaced | Both => format!("@{name}.sock"),
        }
    };

    let conn = LocalSocketStream::connect(name.clone())?;

    log::info!("Connected to {}", name);

    Ok(Pipe::new(conn))
}

pub fn create_server(name: String) -> Result<Pipe> {
    let name = {
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => format!("/tmp/{name}.sock"),
            OnlyNamespaced | Both => format!("@{name}.sock"),
        }
    };

    let listener = match LocalSocketListener::bind(name.clone()) {
        Ok(ret) => ret,
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            return Err(anyhow!("Address already in use"));
        }
        Err(e) => return Err(e.into()),
    };

    log::info!("Server running at {}", name);

    if let Some(Ok(conn)) = listener.incoming().next() {
        log::info!("Incoming connection at {}", name);

        Ok(Pipe::new(conn))
    } else {
        Err(anyhow!("No incoming connection"))
    }
}
