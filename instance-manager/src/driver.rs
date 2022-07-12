use std::{fs::File, io::BufWriter};

use csv::Writer;
use serde::Serialize;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

#[derive(Debug)]
pub struct CsvExportDriver<T>
where
    T: Serialize + Send + Sync + 'static,
{
    out: Writer<BufWriter<File>>,
    reciver: Receiver<T>,
}

impl<T> CsvExportDriver<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub fn new(out: BufWriter<File>, reciver: Receiver<T>) -> Self {
        Self {
            out: Writer::from_writer(out),
            reciver,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<anyhow::Result<()>> {
        tokio::task::spawn_blocking(move || loop {
            if let Some(data) = self.reciver.blocking_recv() {
                self.store(data)?;
            } else {
                break Ok(());
            }
        })
    }

    fn store(&mut self, data: T) -> anyhow::Result<()> {
        self.out.serialize(data)?;
        Ok(())
    }
}

impl<T: Serialize + Send + Sync> Drop for CsvExportDriver<T> {
    fn drop(&mut self) {
        self.out.flush().unwrap();
        self.reciver.close();
    }
}
