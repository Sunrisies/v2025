use wgpu::{Backends, Instance};

#[derive(Debug)]
pub enum GpuType {
    Nvidia(String),
    Amd(String),
    Intel(String),
}

/// Detect GPU(if available) on the system.
pub fn detect_gpu() -> Vec<Option<GpuType>> {
    let instance = Instance::default();
    let adapters = instance.enumerate_adapters(Backends::all());
    let mut gpus = Vec::new();
    for adapter in adapters {
        let info = adapter.get_info();
        // println!("Adapter name: {:?}, vendor: {:#x}", info.name, info.vendor);
        let gpu = match info.vendor {
            0x10DE => Some(GpuType::Nvidia(info.name)),
            0x1002 | 0x1022 => Some(GpuType::Amd(info.name)),
            0x8086 => Some(GpuType::Intel(info.name)),
            _ => None,
        };
        if gpu.is_some() {
            if !gpus.iter().any(|g| match g {
                Some(GpuType::Nvidia(_)) if info.vendor == 0x10DE => true,
                Some(GpuType::Amd(_)) if info.vendor == 0x1002 || info.vendor == 0x1022 => true,
                Some(GpuType::Intel(_)) if info.vendor == 0x8086 => true,
                _ => false,
            }) {
                // println!("GPU detected: {:?}, vendor: {:#x}", &info.name, info.vendor);
                // return gpu;
                gpus.push(gpu);
            }
        }
    }
    gpus
}
