/*
use lilv::port::Port;
use lilv::port::TypedPort;
use lilv::port::buffer::CellBuffer;
use lilv::port::buffer::VecBuffer;
use lilv::instance::{errors::MissingFeatureError, PluginInstance, ResolvedPlugin};
use lv2::core::{Feature, FeatureBuffer, FeatureSet};
use lv2::core::FeatureBuffer;
use std::cell::RefCell;
use std::rc::Rc;
*/


use lilv::instance::PluginInstance;
use lilv::plugin::Plugin;
use lilv::port::{UnknownInputPort, UnknownOutputPort};
use lilv::world::World;
use lv2::core::ports::Audio;
use lv2::core::ports::Control;

use gpplugin::audio_format;
use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Talker, TalkerBase, TalkerHandler, TalkerHandlerBase};

use lv2::core::FeatureBuffer;
/*
struct GpFeatureSet {
    hard_rt_capable: ::lv2::core::features::HardRTCapable,
}

impl GpFeatureSet {
    pub fn new() -> Self {
        Self {
            hard_rt_capable: ::lv2::core::features::HardRTCapable,
        }
    }
}

impl<'a> FeatureSet<'a> for GpFeatureSet {
    fn to_list(&self) -> FeatureBuffer {
        FeatureBuffer::from_vec(vec![Feature::descriptor(&self.hard_rt_capable)])
    }
}
const FEATURE_SET: GpFeatureSet = GpFeatureSet {
    hard_rt_capable: ::lv2::core::features::HardRTCapable,
};

struct Global {
    world: World,
    feature_set: GpFeatureSet,
}

thread_local!(static GLOBAL: RefCell<Global> = RefCell::new(Global {
            world: World::new().unwrap(),
            feature_set: GpFeatureSet {
                hard_rt_capable: ::lv2::core::features::HardRTCapable,
            },
        }));
*/
pub struct Lv2Talker<'a> {
    base: talker::TalkerBase,
    ears: Vec<Ear>,
    instance: PluginInstance<'a>,
}

impl<'a> Lv2Talker<'a> {
    pub fn new<'w>(
        world: &'w World,
        features: &'w FeatureBuffer<'a>,
        /*
         */
        uri: &String,
    ) -> Result<Lv2Talker<'a>, failure::Error> {
        println!("Lv2Talker plugin uri : {}", uri);
/*
        GLOBAL.with(|global_cell| {
            let global = global_cell.borrow_mut();

            //        let world = World::new().unwrap();
            let plugin = global.world.get_plugin_by_uri(uri.as_str()).unwrap();
*/
            //        show_plugin(&plugin);
            //                let features = global.feature_set.to_list();
          let plugin = world.get_plugin_by_uri(uri.as_str()).unwrap();

//            match plugin.resolve(&global.feature_set.to_list()) {
            match plugin.resolve(features) {
                Ok(p) => match p.instantiate(audio_format::sample_rate() as f64) {
                    Ok(instance) => Ok(/*Box::new(*/Self {
                        base: TalkerBase::new(),
                        ears: Vec::new(),
                        instance: instance,
                    }),//),
                    _ => Err(failure::err_msg("PluginInstantiationError")),
                },
                _ => Err(failure::err_msg("MissingFeatureError")),
            }
//        })
    }
}

impl<'a> Talker for Lv2Talker<'a> {
    fn base<'b>(&'b self) -> &'b TalkerBase {
        &self.base
    }
    fn depends_of(&self, _id: u32) -> bool {
        true
    }
    fn ears<'e>(&'e self) -> &'e Vec<Ear> {
        &self.ears
    }
}

fn show_plugin(plugin: &Plugin) {
    println!("> {:?}", plugin);
    for port in plugin.inputs() {
        println!("> {:?}", port);
    }
    for port in plugin.outputs() {
        println!("< {:?}", port);
    }
    for port in plugin
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Audio>)
    {
        println!("\t{:?}", port)
    }
    for port in plugin
        .outputs()
        .filter_map(UnknownOutputPort::into_typed::<Audio>)
    {
        println!("\t{:?}", port)
    }
    for port in plugin
        .inputs()
        .filter_map(UnknownInputPort::into_typed::<Control>)
    {
        println!("\t{:?}", port)
    }
}

pub struct Lv2TalkerHandler {
    base: TalkerHandlerBase,
    /*
    world: &'a World,
    features: &'a FeatureBuffer<'a>,
     */
    uri: String,
}

impl Lv2TalkerHandler {
    pub fn new(
        base: TalkerHandlerBase,
        /*
        world: &'a World,
        features: &'a FeatureBuffer,
        */
        uri: String,
    ) -> Self {
        Self {
            base,
            /*
                        world,
                        features,
            */
            uri,
        }
    }
}
/*
impl TalkerHandler for Lv2TalkerHandler {
    fn base<'b>(&'b self) -> &'b TalkerHandlerBase {
        &self.base
    }
    fn make(&self) -> Result<Box<dyn Talker>, failure::Error> {
        Lv2Talker::new(
            /*&global.world, &global.feature_set.to_list(),*/ &self.uri,
        )
    }
}

pub fn load_plugins(talker_handlers: &mut Vec<Box<dyn TalkerHandler>>) {
    GLOBAL.with(|global_cell| {
        let global = global_cell.borrow_mut();

        for plugin in global.world.plugins() {
            talker_handlers.push(Box::new(Lv2TalkerHandler::new(
                TalkerHandlerBase::new(plugin.name().to_str(), plugin.class().label().to_str()),
                String::from(plugin.uri().to_string()),
            )));
        }
    });
}
*/
