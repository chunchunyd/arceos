use crate::as_dev_err;
use driver_block::BlockDriverOps;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use virtio_drivers::{device::blk::VirtIOBlk as InnerDev, transport::Transport, Hal};

pub struct VirtIoBlkDev<H: Hal, T: Transport> {
    inner: InnerDev<H, T>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoBlkDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoBlkDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoBlkDev<H, T> {
    pub fn try_new(transport: T) -> DevResult<Self> {
        Ok(Self {
            inner: InnerDev::new(transport).map_err(as_dev_err)?,
        })
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIoBlkDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-blk"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
}

impl<H: Hal, T: Transport> BlockDriverOps for VirtIoBlkDev<H, T> {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> DevResult {
        self.inner.read_block(block_id, buf).map_err(as_dev_err)
    }

    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> DevResult {
        self.inner.write_block(block_id, buf).map_err(as_dev_err)
    }

    fn flush(&mut self) -> DevResult {
        Ok(())
    }
}
