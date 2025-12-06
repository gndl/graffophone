use crate::plugin_ui::Parameters;

pub fn _show(plugin: &livi::Plugin) {
    println!("plugin {} ({})", plugin.name(), plugin.uri());

    for classe in plugin.classes() {
        println!("\tclasse : {:?}", classe);
    }

   let lilv_plugin = plugin.raw();

    println!("\tbundle_uri : {:?}", lilv_plugin.bundle_uri());
    println!("\tlibrary_uri : {:?}", lilv_plugin.library_uri());

    for node in lilv_plugin.data_uris() {
       println!("\tdata_uri : {:?}", node.turtle_token());
    }

    for node in lilv_plugin.supported_features() {
       println!("\tsupported_feature : {:?}", node.turtle_token());
    }

    for node in lilv_plugin.required_features() {
       println!("\trequired_feature : {:?}", node.turtle_token());
    }

    if let Some(nodes) = lilv_plugin.extension_data() {
        for node in nodes {
            println!("\textension_data : {:?}", node.turtle_token());
        }
    }
    for port in plugin.ports() {
        println!("\tport : {:?}", port);
    }

    if let Some(uis) = lilv_plugin.uis() {
        for ui in uis {
            println!("\tUI plugin {:?} :", ui.uri().turtle_token());
            println!("\t\tbinary_uri : {:?}", ui.binary_uri().map_or("None".to_string(), |n| n.turtle_token()));
            println!("\t\tbundle_uri : {:?}", ui.bundle_uri().map_or("None".to_string(), |n| n.turtle_token()));
            // ui.is_supported(container_type, ui_type)

            println!("\t\tclasses :");
            for classe in ui.classes() {
                println!("\t\t\t{:?}", classe.turtle_token());
            }
        }
    }
}

pub fn select_plugin_ui_type(world: &livi::World, plugin: &livi::Plugin, ui_type: &str) -> Result<Parameters, failure::Error> {
    let lilv_plugin = plugin.raw();

    if let Some(uis) = lilv_plugin.uis() {
        for ui in uis {
            for ui_type_uri_node in ui.classes() {
                if let Some(ui_type_uri) = ui_type_uri_node.as_uri() {
                    if ui_type.is_empty() || ui_type_uri.contains(ui_type) {
                        let ui_bundle_uri = ui.bundle_uri().expect("Missing bundle uri.");
                        let ui_bundle_path = ui_bundle_uri.as_str().expect("Missing bundle uri str.").get(7..).unwrap();
                        let ui_binary_uri = ui.binary_uri().expect("Missing binary uri.");
                        let ui_binary_path = ui_binary_uri.as_str().expect("Missing binary uri str.").get(7..).unwrap();
                        
                        let is_x11= ui_type_uri.contains("/ui#X11UI");
                        let mut has_show_interface= true;
                        
                        for node in lilv_plugin.supported_features() {
                            if node.as_uri().is_some_and(|uri| uri.contains("/ui#showInterface")) {
                                has_show_interface = true;
                                break;
                            }
                        }
                        
                        if let Some(ui_plugin_uri) = ui.uri().as_uri() {
                            if let Some(ui_plugin) = world.plugin_by_uri(ui_plugin_uri) {
                                println!("UI plugin uri loaded!!!");
                                
                                for req_feat_node in ui_plugin.raw().required_features() {
                                    println!("{:?}", req_feat_node);
                                }
                            }
                        }

                        let pui = Parameters::new(
                            plugin.uri(),
                            ui.uri().as_uri().expect("Missing ui uri."),
                            ui_type_uri,
                            ui_bundle_path,
                            ui_binary_path,
                            is_x11,
                            has_show_interface,
                        );
                        return Ok(pui);
                    }
                }
            }
        }
    }
    Err(failure::err_msg(format!("Luil failed to select plugin ui.")))
}

pub fn select_plugin_ui(world: &livi::World, plugin: &livi::Plugin) -> Result<Parameters, failure::Error> {
    for ui_type in ["/ui#X11UI", ""] {
        let res = select_plugin_ui_type(world, plugin, ui_type);

        if res.is_ok() {
            return res;
        }
    }
    Err(failure::err_msg(format!("Luil failed to select plugin ui.")))
}

