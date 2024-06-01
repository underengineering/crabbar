use async_broadcast::{InactiveReceiver, Receiver, Sender};
use pulse::{
    callbacks::ListResult,
    context::{
        subscribe::{InterestMaskSet, Operation},
        FlagSet,
    },
    volume::{ChannelVolumes, Volume},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct SinkInfo {
    pub name: Option<String>,
    pub index: u32,
    pub volume: ChannelVolumes,
    pub base_volume: Volume,
    pub mute: bool,
}

#[derive(Clone, Debug)]
pub enum PulseaudioEvent {
    StateChange(pulse::context::State),
    SinkUpdate { op: Operation, sink_info: SinkInfo },
    DefaultSinkChanged(SinkInfo),
}

pub struct PulseaudioWrapper {
    mainloop: pulse_glib::Mainloop,

    tx: Sender<PulseaudioEvent>,
    rx: InactiveReceiver<PulseaudioEvent>,
}

impl PulseaudioWrapper {
    pub fn new() -> Self {
        let mainloop = pulse_glib::Mainloop::new(None).expect("Failed to create mainloop");

        let (tx, rx) = async_broadcast::broadcast(24);
        let rx = rx.deactivate();
        Self { mainloop, tx, rx }
    }

    pub fn receiver(&self) -> Receiver<PulseaudioEvent> {
        self.rx.activate_cloned()
    }

    fn connect(&self) {
        let context = pulse::context::Context::new(&self.mainloop, "crabbar")
            .expect("Failed to create context");
        let mut context = Context::new(context, self.tx.clone());
        context.connect();
    }

    pub async fn run(&mut self) {
        self.connect();

        let mut rx = self.rx.activate_cloned();
        while let Ok(event) = rx.recv().await {
            if let PulseaudioEvent::StateChange(
                pulse::context::State::Terminated | pulse::context::State::Failed,
            ) = event
            {
                self.connect();
            }
        }
    }
}

struct Context {
    context: Rc<RefCell<pulse::context::Context>>,
    tx: Sender<PulseaudioEvent>,
}

impl Context {
    fn new(context: pulse::context::Context, tx: Sender<PulseaudioEvent>) -> Self {
        let context = Rc::new(RefCell::new(context));
        Self { context, tx }
    }

    fn connect(&mut self) {
        let context_ref = self.context.clone();
        let tx = self.tx.clone();
        self.context
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                // NOTE: Content:connect may call state callback while being borrowed
                let state = context_ref
                    .try_borrow()
                    .map(|ctx| ctx.get_state())
                    .unwrap_or(pulse::context::State::Connecting);

                let event = PulseaudioEvent::StateChange(state);
                pollster::block_on(tx.broadcast_direct(event)).unwrap();

                if state == pulse::context::State::Ready {
                    Self::subscribe(&context_ref, &tx);
                    Self::request_default_sink(&context_ref, &tx);
                }
            })));

        self.context
            .borrow_mut()
            .connect(None, FlagSet::NOAUTOSPAWN, None)
            .unwrap();
    }

    fn subscribe(context: &Rc<RefCell<pulse::context::Context>>, tx: &Sender<PulseaudioEvent>) {
        let context_ref = context.clone();
        let tx = tx.clone();
        context.borrow_mut().subscribe(
            InterestMaskSet::SINK | InterestMaskSet::SERVER,
            move |_success| {
                Self::set_subscribe_callback(&context_ref, &tx);
            },
        );
    }

    fn set_subscribe_callback(
        context: &Rc<RefCell<pulse::context::Context>>,
        tx: &Sender<PulseaudioEvent>,
    ) {
        let context_ref = context.clone();
        let tx = tx.clone();
        context
            .borrow_mut()
            .set_subscribe_callback(Some(Box::new(move |facility, op, index| match facility {
                Some(pulse::context::subscribe::Facility::Server)
                    if op == Some(Operation::Changed) =>
                {
                    Self::request_default_sink(&context_ref, &tx);
                }
                Some(pulse::context::subscribe::Facility::Sink) => {
                    if let Some(op) = op {
                        Self::send_sink_update_event(index, op, &context_ref, &tx);
                    }
                }
                _ => {}
            })));
    }

    fn request_default_sink(
        context: &Rc<RefCell<pulse::context::Context>>,
        tx: &Sender<PulseaudioEvent>,
    ) {
        let context_ref = context.clone();
        let tx = tx.clone();
        context
            .borrow_mut()
            .introspect()
            .get_server_info(move |info| {
                if let Some(default_sink_name) = &info.default_sink_name {
                    Self::send_default_sink_changed_event(default_sink_name, &context_ref, &tx);
                }
            });
    }

    fn send_default_sink_changed_event(
        sink_name: &str,
        context: &Rc<RefCell<pulse::context::Context>>,
        tx: &Sender<PulseaudioEvent>,
    ) {
        let tx = tx.clone();
        context
            .borrow_mut()
            .introspect()
            .get_sink_info_by_name(sink_name, move |result| {
                if let ListResult::Item(item) = result {
                    let sink_info = SinkInfo {
                        name: item.name.as_ref().map(|name| name.to_string()),
                        index: item.index,
                        volume: item.volume,
                        base_volume: item.base_volume,
                        mute: item.mute,
                    };

                    pollster::block_on(
                        tx.broadcast_direct(PulseaudioEvent::DefaultSinkChanged(sink_info)),
                    )
                    .unwrap();
                }
            });
    }

    fn send_sink_update_event(
        sink_index: u32,
        op: Operation,
        context: &Rc<RefCell<pulse::context::Context>>,
        tx: &Sender<PulseaudioEvent>,
    ) {
        let tx = tx.clone();

        context.borrow_mut().introspect().get_sink_info_by_index(
            sink_index,
            move |result: ListResult<&pulse::context::introspect::SinkInfo<'_>>| {
                if let ListResult::Item(item) = result {
                    let sink_info = SinkInfo {
                        name: item.name.as_ref().map(|name| name.to_string()),
                        index: item.index,
                        volume: item.volume,
                        base_volume: item.base_volume,
                        mute: item.mute,
                    };

                    pollster::block_on(
                        tx.broadcast_direct(PulseaudioEvent::SinkUpdate { op, sink_info }),
                    )
                    .unwrap();
                }
            },
        );
    }
}
