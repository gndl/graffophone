
#[derive(Debug, Fail)]
#[fail(display = "Unable to instantiate LV2 plugin")]
pub struct PluginInstantiationError;

#[derive(Debug, Fail)]
#[fail(display = "Missing connection: Port #{} is not connected and is not optional", index)]
pub struct MissingConnectionError {
    index: u32
}

#[derive(Debug, Fail)]
#[fail(display = "Missing feature")]
pub struct MissingFeatureError {}