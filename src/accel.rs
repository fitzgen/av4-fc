//! The accel-related traits and actor.

use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

/// Raw, unprocessed accel data.
pub struct RawAccelData(u64);

/// Munged, processed accel data.
pub struct ProcessedAccelData(u64);

/// Anything that can provide raw accel data.
///
/// In tests, we can mock this trait to return whatever sequence of raw accel
/// data we want. For the real deal, this would perform IO directly.
pub trait AccelSource {
    fn read_accel(&self) -> RawAccelData;
}

/// Anything that can make use of processed accel data.
///
/// In tests, we would mock this to assert our expectations for processed data
/// based on whatever test data our mocked source was feeding in. For the real
/// deal, this would forward data as input to other actors.
pub trait AccelSink {
    fn send_accel(&self, data: ProcessedAccelData);
}

// For exposition.
impl AccelSource for mpsc::Receiver<RawAccelData> {
    fn read_accel(&self) -> RawAccelData {
        self.recv().unwrap()
    }
}

// For exposition, although we would probably want to really use something like
// this for a channel sender to sensor fusion.
impl<T> AccelSink for mpsc::Sender<T>
    where T: From<ProcessedAccelData>
{
    fn send_accel(&self, data: ProcessedAccelData) {
        self.send(data.into()).unwrap()
    }
}

/// A AccelActor is just a handle to the thread running the accel processing loop.
pub struct AccelActor<Source, Sink> {
    source: PhantomData<Source>,
    sink: PhantomData<Sink>,
}

impl<Source, Sink> AccelActor<Source, Sink>
    where Source: 'static + AccelSource + Send,
          Sink: 'static + AccelSink + Send
{
    /// Spawn the accel processing loop in its own thread, and get back the
    /// AccelActor handle to it.
    pub fn spawn(source: Source, sink: Sink) -> AccelActor<Source, Sink> {
        thread::spawn(move || AccelActor::run(source, sink));
        AccelActor {
            source: PhantomData,
            sink: PhantomData,
        }
    }

    // TODO: Maybe add a method to shut down this actor? Could use atomics or a
    // channel or something else.

    fn run(source: Source, sink: Sink) {
        loop {
            // TODO: check if we've been requested to terminate or something.
            AccelActor::process(&source, &sink);
            // TODO: delay between processing samples?
        }
    }

    fn process(source: &Source, sink: &Sink) {
        // Do whatever munging, massaging, and processing to go from raw to
        // processed accel data... This is the main function to unit test, most
        // everything else is boilerplate that we'd like to abstract out between
        // all actors once we know a little more about precisely what we are
        // doing.
        let raw = source.read_accel();
        let processed = ProcessedAccelData(raw.0);
        sink.send_accel(processed);
    }
}
