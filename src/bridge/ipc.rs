use crate::bridge::model::{BridgeRecord, deserialize_record, serialize_record};
use miette::{IntoDiagnostic, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct IpcClient {
    pipe: File,
}

impl IpcClient {
    const PIPE_PATH: &'static str = r"\\.\pipe\aibrains-sync";

    pub fn connect_with_timeout(timeout: Duration) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        // Spawning a thread is still needed for synchronous open on Windows pipes
        // but we'll try to be more careful.
        thread::spawn(move || {
            // Using FILE_FLAG_FIRST_PIPE_INSTANCE would be for servers,
            // but for clients we just want to open.
            let res = OpenOptions::new()
                .read(true)
                .write(true)
                .open(Self::PIPE_PATH);
            let _ = tx.send(res);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(pipe)) => Ok(Self { pipe }),
            Ok(Err(e)) => Err(e).into_diagnostic(),
            Err(_) => Err(miette::miette!("Connection to AI-Brains IPC timed out.")),
        }
    }

    pub fn send_record(&mut self, record: &BridgeRecord) -> Result<()> {
        let line = serialize_record(record).into_diagnostic()?;
        self.pipe.write_all(line.as_bytes()).into_diagnostic()?;
        self.pipe.write_all(b"\n").into_diagnostic()?;
        self.pipe.flush().into_diagnostic()?;
        Ok(())
    }

    pub fn receive_records(&mut self) -> Result<Vec<BridgeRecord>> {
        let mut reader = BufReader::new(&self.pipe);
        let mut records = Vec::new();

        // Use a small timeout for the read as well
        // Since we are using synchronous pipes, we'll just read one line if available.
        // On Windows, named pipes can be put in non-blocking mode after opening.

        let mut line = String::new();
        if reader.read_line(&mut line).into_diagnostic()? > 0
            && let Ok(record) = deserialize_record(&line)
        {
            records.push(record);
        }

        Ok(records)
    }
}
