use gpplugin::ear::Ear;
use gpplugin::talker;
use gpplugin::talker::{Base, Handler, Talker};

pub struct Lv2Talker {
    base: talker::Base,
    ears: Vec<Ear>,
}
impl Lv2Talker {
    pub fn new(uri: String) -> Result<Self, failure::Error> {
        println!("Lv2Talker plugin uri : {}", uri);
        Ok(Self {
            base: gpplugin::talker::Base::new(),
            ears: Vec::new(),
        })
    }
}

impl Talker for Lv2Talker {
    fn base<'a>(&'a self) -> &'a Base {
        &self.base
    }
    fn depends_of(&self, id: u32) -> bool {
        true
    }
    fn get_ears<'a>(&'a self) -> &'a Vec<Ear> {
        &self.ears
    }
}

fn get_plugin_handlers() -> Vec<Handler> {
    let phs = Vec::new();

    //    println!("lilv_plugins_size: {}", lilv_sys::lilv_plugins_size(plugins));
    let world = World::new().unwrap();

    println!("Print plugins start");

    for plugin in world.plugins() {
        let ph = Handler::new(
            plugin.name(),
            plugin.uri(),
            Box::new(|| Box::new(Lv2Talker::new(plugin.uri()))),
        );

        phs.push(ph);
        println!("{} {}", plugin.name(), plugin.uri());
        /*
        for port in plugin.inputs() {
            println!("> {:?}", port);
        }
        for port in plugin.outputs() {
            println!("< {:?}", port);
        }
        */
    }
    println!("Print plugins end");
    phs
}
