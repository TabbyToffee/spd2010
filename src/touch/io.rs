use super::{Error, SPD2010_ADDR, SPD2010Touch};
use crate::touch::InterruptInput;
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, Operation},
};

impl<'a, I2C: I2c, Ti: InterruptInput> SPD2010Touch<'a, I2C, Ti> {
    pub fn read_register(&mut self, reg: u16, data: &mut [u8]) -> Result<(), Error<I2C>> {
        let reg_bytes = [reg as u8, (reg >> 8) as u8];
        self.i2c
            .write(SPD2010_ADDR, &reg_bytes)
            .map_err(Error::I2C)?;
        self.i2c.read(SPD2010_ADDR, data).map_err(Error::I2C)?;
        Ok(())
    }

    pub fn write_command(&mut self, reg: u16, data: &[u8]) -> Result<(), Error<I2C>> {
        let reg_bytes = [reg as u8, (reg >> 8) as u8];

        self.i2c
            .transaction(
                SPD2010_ADDR,
                &mut [Operation::Write(&reg_bytes), Operation::Write(data)],
            )
            .map_err(Error::I2C)?;

        Ok(())
    }

    pub async fn clear_interrupt<DELAY: DelayNs>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), Error<I2C>> {
        let ack: [u8; 2] = [0x01, 0x00]; // step 1: ACK (acknowledge interrupt)
        let rearm: [u8; 2] = [0x00, 0x00]; // step 2: re-arm (setup interrupt again)

        self.write_command(0x0002, &ack)?; // ack
        delay.delay_us(200);
        self.write_command(0x0002, &rearm)?; // re-arm
        delay.delay_ms(10);

        if !self.get_interrupt_state() {
            Err(Error::ClearInterruptFailed)
        } else {
            Ok(())
        }
    }
}
