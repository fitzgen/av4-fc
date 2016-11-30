//! The mag-related traits and actor.

use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;

/// Raw, unprocessed mag data.
pub struct RawMagData(u64);

/// Munged, processed mag data.
pub struct ProcessedMagData(u64);

/// Anything that can provide raw mag data.
///
/// In tests, we can mock this trait to return whatever sequence of raw mag
/// data we want. For the real deal, this would perform IO directly.
pub trait MagSource {
    fn read_mag(&self) -> RawMagData;
}

/// Anything that can make use of processed mag data.
///
/// In tests, we would mock this to assert our expectations for processed data
/// based on whatever test data our mocked source was feeding in. For the real
/// deal, this would forward data as input to other actors.
pub trait MagSink {
    fn send_mag(&self, data: ProcessedMagData);
}

// For exposition.
impl MagSource for mpsc::Receiver<RawMagData> {
    fn read_mag(&self) -> RawMagData {
        self.recv().unwrap()
    }
}

// For exposition, although we would probably want to really use something like
// this for a channel sender to sensor fusion.
impl<T> MagSink for mpsc::Sender<T>
    where T: From<ProcessedMagData>
{
    fn send_mag(&self, data: ProcessedMagData) {
        self.send(data.into()).unwrap()
    }
}

/// A MagActor is just a handle to the thread running the mag processing loop.
pub struct MagActor<Source, Sink> {
    source: PhantomData<Source>,
    sink: PhantomData<Sink>,
}

impl<Source, Sink> MagActor<Source, Sink>
    where Source: 'static + MagSource + Send,
          Sink: 'static + MagSink + Send,
{
    /// Spawn the mag processing loop in its own thread, and get back the
    /// MagActor handle to it.
    pub fn spawn(source: Source, sink: Sink) -> MagActor<Source, Sink> {
        thread::spawn(move || MagActor::run(source, sink));
        MagActor {
            source: PhantomData,
            sink: PhantomData,
        }
    }

    // TODO: Maybe add a method to shut down this actor? Could use atomics or a
    // channel or something else.

    fn run(source: Source, sink: Sink) {
        loop {
            // TODO: check if we've been requested to terminate or something.
            MagActor::process(&source, &sink);
            // TODO: delay between processing samples?
        }
    }

    fn process(source: &Source, sink: &Sink) {
        // Do whatever munging, massaging, and processing to go from raw to
        // processed mag data... This is the main function to unit test, most
        // everything else is boilerplate that we'd like to abstract out between
        // all actors once we know a little more about precisely what we are
        // doing.
        let raw = source.read_mag();
        let processed = ProcessedMagData(raw.0);
        sink.send_mag(processed);
    }
}
