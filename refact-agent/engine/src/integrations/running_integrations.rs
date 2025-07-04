use std::sync::Arc;
use indexmap::IndexMap;
use tokio::sync::RwLock as ARwLock;

use crate::custom_error::YamlError;
use crate::global_context::GlobalContext;
use crate::integrations::integr_abstract::IntegrationTrait;

/// Loads and set up integrations from config files.
///
/// If `include_paths_matching` is `None`, all integrations are loaded,
/// otherwise only those matching `include_paths_matching` glob patterns.
pub async fn load_integrations(
    gcx: Arc<ARwLock<GlobalContext>>,
    include_paths_matching: &[String],
) -> (IndexMap<String, Box<dyn IntegrationTrait + Send + Sync>>, Vec<YamlError>) {
    let active_project_path = crate::files_correction::get_active_project_path(gcx.clone()).await;
    let (config_dirs, global_config_dir) = crate::integrations::setting_up_integrations::get_config_dirs(gcx.clone(), &active_project_path).await;
    let (integrations_yaml_path, is_inside_container, allow_experimental) = {
        let gcx_locked = gcx.read().await;
        (gcx_locked.cmdline.integrations_yaml.clone(), gcx_locked.cmdline.inside_container, gcx_locked.cmdline.experimental)
    };

    let mut error_log: Vec<YamlError> = Vec::new();
    let lst: Vec<&str> = crate::integrations::integrations_list(allow_experimental);
    let vars_for_replacements = crate::integrations::setting_up_integrations::get_vars_for_replacements(gcx.clone(), &mut error_log).await;
    let records = crate::integrations::setting_up_integrations::read_integrations_d(
        &config_dirs,
        &global_config_dir,
        &integrations_yaml_path,
        &vars_for_replacements,
        &lst,
        &mut error_log,
        include_paths_matching,
        false,
    );

    let mut integrations_map = IndexMap::new();
    for rec in records {
        if !is_inside_container && !rec.on_your_laptop { continue; }
        if is_inside_container && !rec.when_isolated { continue; }
        if !rec.integr_config_exists { continue; }
        let mut integr = match crate::integrations::integration_from_name(&rec.integr_name) {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("don't have integration {}: {}", rec.integr_name, e);
                continue;
            }
        };
        let should_be_fine = integr.integr_settings_apply(gcx.clone(), rec.integr_config_path.clone(), &rec.config_unparsed).await;
        if let Err(err) = should_be_fine {
            let error_line = err.line();
            error_log.push(YamlError {
                path: rec.integr_config_path.clone(),
                error_line,
                error_msg: format!("failed to apply settings: {}", err),
            });
        }
        integrations_map.insert(rec.integr_name.clone(), integr);
    }

    for e in error_log.iter() {
        tracing::error!("{e}");
    }

    (integrations_map, error_log)
}
