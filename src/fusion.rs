//! The traits and actor related to sensor fusion.

use accel;
use gyro;
use mag;
use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

/// The input to sensor fusion is all of our various kinds of sensor data.
pub enum SensorInput {
    /// Input from the accel sensor.
    Accel(accel::ProcessedAccelData),

    /// Input from the gyro sensor.
    Gyro(gyro::ProcessedGyroData),

    /// Input from the mag sensor.
    Mag(mag::ProcessedMagData),
}

// A bunch of type conversion trait implementation glue so that this can be used
// with the `impl<T> {Gyro,Mag,Accel}Sink for mpsc::Sender<T>` implementations.

impl From<accel::ProcessedAccelData> for SensorInput {
    fn from(data: accel::ProcessedAccelData) -> Self {
        SensorInput::Accel(data)
    }
}

impl From<gyro::ProcessedGyroData> for SensorInput {
    fn from(data: gyro::ProcessedGyroData) -> Self {
        SensorInput::Gyro(data)
    }
}

impl From<mag::ProcessedMagData> for SensorInput {
    fn from(data: mag::ProcessedMagData) -> Self {
        SensorInput::Mag(data)
    }
}

/// Anything that can provide sensor input data.
///
/// In tests, we can mock this trait to return whatever sequence of sensor input
/// data we want. For the real deal, this would use mpsc channels to talk to the
/// actors performing IO and processing raw data from the real sensors.
pub trait SensorInputSource {
    fn read_sensor_input(&self) -> SensorInput;
}

// For exposition.
impl SensorInputSource for mpsc::Receiver<SensorInput> {
    fn read_sensor_input(&self) -> SensorInput {
        self.recv().unwrap()
    }
}

/// The output of sensor fusion.
#[derive(Clone, Default)]
pub struct FusedSensorOutput {
    // Whatever fused output looks like...
}

impl FusedSensorOutput {
    /// Fuse more sensor input data into this fused output.
    pub fn join(self, _more_input: SensorInput) -> FusedSensorOutput {
        // TODO: actually fuse data...
        self
    }
}

/// Anything that wants to use the fused sensor output.
///
/// In tests, we would mock this to assert our expectations for fused output
/// given the test input from different mocked sensors that collectively
/// implement a mocked SensorInputSource. For the real deal, this would forward
/// data to the flight controller, probably along an mpsc channel.
pub trait SensorOutputSink {
    /// Send the fused sensor output to the sink.
    fn send_sensor_output(&self, output: FusedSensorOutput);
}

// For exposition, although we would probably want to really use something like
// this for a channel sender to flight control.
impl<T> SensorOutputSink for mpsc::Sender<T>
    where T: From<FusedSensorOutput>
{
    fn send_sensor_output(&self, output: FusedSensorOutput) {
        self.send(output.into()).unwrap()
    }
}

/// A SensorFusionActor is just a handle to the thread running the sensor fusion
/// loop.
pub struct SensorFusionActor<Source, Sink> {
    source: PhantomData<Source>,
    sink: PhantomData<Sink>,
}

impl<Source, Sink> SensorFusionActor<Source, Sink>
    where Source: 'static + SensorInputSource + Send,
          Sink: 'static + SensorOutputSink + Send
{
    /// Spawn the sensor fusion processing loop in its own thread, and get back
    /// the SensorFusionActor handle to it.
    pub fn spawn(source: Source, sink: Sink) -> SensorFusionActor<Source, Sink> {
        thread::spawn(move || SensorFusionActor::run(source, sink));
        SensorFusionActor {
            source: PhantomData,
            sink: PhantomData,
        }
    }

    // TODO: Maybe add a method to shut down this actor? Could use atomics or a
    // channel or something else.

    fn run(source: Source, sink: Sink) {
        let mut data = FusedSensorOutput::default();
        loop {
            // TODO: check if we've been requested to terminate or something.
            data = SensorFusionActor::process(data, &source, &sink);
        }
    }

    fn process(data: FusedSensorOutput, source: &Source, sink: &Sink) -> FusedSensorOutput {
        // Do whatever sensor input fusion... This is the main function to unit
        // test, most everything else is boilerplate that we'd like to abstract
        // out between all actors once we know a little more about precisely
        // what we are doing.
        //
        // Note that this takes and returns the state it wants to persist,
        // unlike say the GyroActor which does not persist any state.
        let input = source.read_sensor_input();
        let fused = data.join(input);
        sink.send_sensor_output(fused.clone());
        fused
    }
}
