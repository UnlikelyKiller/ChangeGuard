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

        thread::spawn(move || {
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

        // This is tricky for synchronous pipes as it might block.
        // For now, let's assume we read until the first empty line or EOF
        // But for IPC, we might need a timeout here too.

        let mut line = String::new();
        if reader.read_line(&mut line).into_diagnostic()? > 0 {
            if let Ok(record) = deserialize_record(&line) {
                records.push(record);
            }
        }

        Ok(records)
    }
}
