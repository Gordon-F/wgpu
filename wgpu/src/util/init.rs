use wgt::{BackendBit, PowerPreference, RequestAdapterOptions};

use crate::{Adapter, Instance};

/// Get a set of backend bits from the environment variable WGPU_BACKEND.
pub fn backend_bits_from_env() -> Option<BackendBit> {
    Some(
        match std::env::var("WGPU_BACKEND")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("vulkan") => BackendBit::VULKAN,
            Ok("dx12") => BackendBit::DX12,
            Ok("dx11") => BackendBit::DX11,
            Ok("metal") => BackendBit::METAL,
            Ok("gl") => BackendBit::GL,
            Ok("webgpu") => BackendBit::BROWSER_WEBGPU,
            _ => None?,
        },
    )
}

/// Get a power preference from the environment variable WGPU_POWER_PREF
pub fn power_preference_from_env() -> Option<PowerPreference> {
    Some(
        match std::env::var("WGPU_POWER_PREF")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("low") => PowerPreference::LowPower,
            Ok("high") => PowerPreference::HighPerformance,
            _ => None?,
        },
    )
}

/// Initialize the adapter obeying the WGPU_ADAPTER_NAME environment variable.
pub fn initialize_adapter_from_env(
    instance: &Instance,
    backend_bits: BackendBit,
) -> Option<Adapter> {
    let adapters = instance.enumerate_adapters(backend_bits);

    let desired_adapter_name = std::env::var("WGPU_ADAPTER_NAME")
        .as_deref()
        .map(str::to_lowercase)
        .ok()?;

    let mut chosen_adapter = None;
    for adapter in adapters {
        let info = adapter.get_info();

        if info.name.to_lowercase().contains(&desired_adapter_name) {
            chosen_adapter = Some(adapter);
            break;
        }
    }

    chosen_adapter
}

/// Initialize the adapter obeying the WGPU_ADAPTER_NAME environment variable and if it doesn't exist fall back on a default adapter.
pub async fn initialize_adapter_from_env_or_default(
    instance: &Instance,
    backend_bits: wgt::BackendBit,
) -> Option<Adapter> {
    match initialize_adapter_from_env(&instance, backend_bits) {
        Some(a) => Some(a),
        None => {
            instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: power_preference_from_env()
                        .unwrap_or_else(PowerPreference::default),
                    compatible_surface: None,
                })
                .await
        }
    }
}
