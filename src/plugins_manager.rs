use crate::lilv_talker::Lv2Talker;

fn load_plugins() -> Vec<Handler> {
    let tkr: Box<dyn Talker> = Box::new(Lv2Talker::new().unwrap());
    println!("tkr id {}, name {}", tkr.get_id(), tkr.get_name());

    lv2_talker::get_plugin_handlers()
}
