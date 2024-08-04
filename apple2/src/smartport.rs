use std::{fs, io};
use std::fs::File;
use std::io::Read;
use crate::ui::iced::shared::Shared;

/// Implementation of the SmartPort interface. Make sure that $C0F8 returns the correct bytes
/// from the currently loaded hard drive / block number.
pub struct SmartPort {
    /// Content of the .hdv file
    file_content: Vec<u8>,

    /// The current block we are returning bytes from whenever $C0F8 is invoked
    block_content: [u8; 512],
    block_content_index: usize,

    /// Used to figure out if we need to load a new block
    last_block_read: Option<u16>,
}

impl Default for SmartPort {
    fn default() -> Self {
        Self {
            last_block_read: None,
            block_content: [0; 512],
            block_content_index: 0,
            file_content: vec![],
        }
    }
}

impl SmartPort {
    /// Invoked whenever $C0F8 is read
    pub fn next_byte(&mut self, block_number: u16) -> io::Result<u8> {
        // Load the block if we haven't read any or if the current block is different from
        // the one requested
        if self.last_block_read.map_or(true, |bn| bn != block_number) {
            self.read_block(block_number)?;
        }

        if self.block_content_index >= 512 {
            // Not sure what the behavior is supposed to be if the caller reads $C0F8
            // more than 512 times without loading another block. Assume we just loop
            // around the same buffer
            self.block_content_index = 0;
        }

        let result = self.block_content[self.block_content_index];
        self.block_content_index += 1;

        Ok(result)
    }

    /// Read the block_number in our holding 512 byte buffer.
    /// That buffer will then be returned one byte at a time each time $C0F8 is read
    fn read_block(&mut self, block_number: u16) -> io::Result<()> {
        if let Some(disk_info) = &Shared::get_hard_drive(0) {
            if self.file_content.is_empty() {
                let mut file = File::open(&disk_info.path)?;
                let metadata = fs::metadata(&disk_info.path)?;
                self.file_content = vec![0; metadata.len() as usize];
                file.read_exact(&mut self.file_content)?;
            }

            let offset = block_number as usize * 512;
            self.block_content.copy_from_slice(&self.file_content[offset..(512 + offset)]);
            self.last_block_read = Some(block_number);
            self.block_content_index = 0;

            Shared::set_block_number(0, block_number);
        }

        Ok(())
    }
}