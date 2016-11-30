//! The gyro-related traits and actor.

use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

/// Raw, unprocessed gyro data.
pub struct RawGyroData(u64);

/// Munged, processed gyro data.
pub struct ProcessedGyroData(u64);

/// Anything that can provide raw gyro data.
///
/// In tests, we can mock this trait to return whatever sequence of raw gyro
/// data we want. For the real deal, this would perform IO directly.
pub trait GyroSource {
    fn read_gyro(&self) -> RawGyroData;
}

/// Anything that can make use of processed gyro data.
///
/// In tests, we would mock this to assert our expectations for processed data
/// based on whatever test data our mocked source was feeding in. For the real
/// deal, this would forward data as input to other actors.
pub trait GyroSink {
    fn send_gyro(&self, data: ProcessedGyroData);
}

// For exposition.
impl GyroSource for mpsc::Receiver<RawGyroData> {
    fn read_gyro(&self) -> RawGyroData {
        self.recv().unwrap()
    }
}

// For exposition, although we would probably want to really use something like
// this for a channel sender to sensor fusion.
impl<T> GyroSink for mpsc::Sender<T>
    where T: From<ProcessedGyroData>
{
    fn send_gyro(&self, data: ProcessedGyroData) {
        self.send(data.into()).unwrap()
    }
}

/// A GyroActor is just a handle to the thread running the gyro processing loop.
pub struct GyroActor<Source, Sink> {
    source: PhantomData<Source>,
    sink: PhantomData<Sink>,
}

impl<Source, Sink> GyroActor<Source, Sink>
    where Source: 'static + GyroSource + Send,
          Sink: 'static + GyroSink + Send
{
    /// Spawn the gyro processing loop in its own thread, and get back the
    /// GyroActor handle to it.
    pub fn spawn(source: Source, sink: Sink) -> GyroActor<Source, Sink> {
        thread::spawn(move || GyroActor::run(source, sink));
        GyroActor {
            source: PhantomData,
            sink: PhantomData,
        }
    }

    // TODO: Maybe add a method to shut down this actor? Could use atomics or a
    // channel or something else.

    fn run(source: Source, sink: Sink) {
        loop {
            // TODO: check if we've been requested to terminate or something.
            GyroActor::process(&source, &sink);
            // TODO: delay between processing samples?
        }
    }

    fn process(source: &Source, sink: &Sink) {
        // Do whatever munging, massaging, and processing to go from raw to
        // processed gyro data... This is the main function to unit test, most
        // everything else is boilerplate that we'd like to abstract out between
        // all actors once we know a little more about precisely what we are
        // doing.
        let raw = source.read_gyro();
        let processed = ProcessedGyroData(raw.0);
        sink.send_gyro(processed);
    }
}
