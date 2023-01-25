use crate::{sensor::Sensor, I2cBus, I2cRef, Retry, SensorMetrics};
use esp_idf_hal::delay::Ets;

pub type Bme680 = bosch_bme680::Bme680<I2cRef<'static>, Ets>;

impl Sensor for Bme680 {
    const NAME: &'static str = "BME680";

    fn bringup(busman: &'static I2cBus) -> anyhow::Result<Self> {
        let config = bosch_bme680::Configuration::default();
        log::info!("connecting to BME680 with config {config:#?}");
        let i2c = busman.acquire_i2c();
        bosch_bme680::Bme680::new(
            i2c,
            // the default I2C address of the Adafruit BME680 breakout board
            // is the "secondary" address, 0x77.
            bosch_bme680::DeviceAddress::Secondary,
            Ets,
            &config,
            // TODO(eliza): can we get the ambient temperature from a SCD30 measurement?
            20,
        )
        .map_err(|error| anyhow::anyhow!("failed to connect to BME680: {error:?}"))
    }

    fn poll(&mut self, metrics: &SensorMetrics) -> anyhow::Result<()> {
        let bosch_bme680::MeasurmentData {
            temperature,
            pressure,
            humidity,
            gas_resistance,
        } = self
            .measure()
            .map_err(|error| anyhow::anyhow!("error reading from BME680: {error:?}"))?;
        log::info!("[BME680]: Pressure: {pressure:>3.3} hPa, Temp: {temperature:>3.3} \u{00B0}C, Humidity: {humidity:>3.3}%");
        metrics.pressure.sensors().set_value(pressure);
        metrics.temp.sensors().bme680.set_value(temperature);
        metrics.humidity.sensors().bme680.set_value(humidity);
        if let Some(gas) = gas_resistance {
            log::info!("[BME680]: Gas resistance: {gas:>3.3} \u{2126}");
            metrics.gas_resistance.sensors().set_value(gas);
        }

        Ok(())
    }

    fn incr_error(metrics: &SensorMetrics) {
        metrics.sensor_errors.sensors().bme680.incr();
    }
}
