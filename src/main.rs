use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericPlatform, GenericMetadata};
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, Scope, ModificationBehavior};
use libcnb::{buildpack_main, Buildpack};
use std::path::Path;
use serde::Deserialize;

pub(crate) struct DotenvBuildpack;

impl Buildpack for DotenvBuildpack {
    type Platform = GenericPlatform;
    type Metadata = DotenvBuildpackMetadata;
    type Error = DotenvBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join(".env").exists() {
            DetectResultBuilder::pass()
                .build_plan(
                    BuildPlanBuilder::new()
                        .provides("dotenv")
                        .requires("dotenv")
                        .build()
                )
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        println!("---> DotEnv Buildpack");

        context.handle_layer(layer_name!("dotenv"), DotenvLayer)?;

        BuildResultBuilder::new()
            .build()
    }
}

pub(crate) struct DotenvLayer;

impl Layer for DotenvLayer {
    type Buildpack = DotenvBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: false,
            cache: false,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, DotenvBuildpackError> {
        println!("---> Parse .env file and set it to image");

        let metadata = &context.buildpack_descriptor.metadata;

        let suffix = if let Some(s) = option_env!("BP_DOTENV_SUFFIX") { s.to_string() } else { metadata.dotenv_suffix.to_string()  };

        let filename = format!(".env.{}", suffix).trim_end_matches('.').to_string();
        let mut le = LayerEnv::new();

        dotenv::from_filename_iter(layer_path.join(filename)).unwrap()
            .for_each(|r| {
                match r {
                    Ok((name, value)) => Some(le.insert(Scope::All, ModificationBehavior::Default, name, value)),
                    Err(_) => None,
                };
            });


        LayerResultBuilder::new(GenericMetadata::default()).env(le).build()
    }
}



#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct DotenvBuildpackMetadata {
    pub dotenv_suffix: String,
}


#[derive(Debug)]
pub(crate) enum DotenvBuildpackError {}

buildpack_main!(DotenvBuildpack);
