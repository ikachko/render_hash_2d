use ocl::enums::{DeviceInfo, DeviceInfoResult};

macro_rules! get_info {
    ($dev:ident, $name:ident) => {{
        match $dev.info(DeviceInfo::$name) {
            Ok(DeviceInfoResult::$name(value)) => value,
            _ => panic!("Failed to retrieve device {}", stringify!($name)),
        }
    }};
}

macro_rules! get_memory {
    ($dev:ident, $name:ident) => {{
        let memory = get_info!($dev, $name);
        memory_to_string(memory as usize);
    }};
}

fn memory_to_string(memory: usize) -> String {
    use number_prefix::{NumberPrefix, Standalone, Prefixed};
    match NumberPrefix::binary(memory as f64) {
        Standalone(bytes) => format!("{} bytes", bytes),
        Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix)
    }
}

fn print_device_info(dev: &ocl::Device) {
    // Some general information.
    println!("  * {}", dev.name().expect("Failed to retrieve device name"));

    let version = dev.version().expect("Failed to retrieve device version");
    println!("  - Version: {}", version);

    // Information related to work-groups and work items.
    println!("  * Work-group information");

    let max_compute_units = get_info!(dev, MaxComputeUnits);
    println!("   - Maximum compute units: {}", max_compute_units);

    let max_wg_size = dev.max_wg_size().expect(
        "Failed to retrieve max work-group size",
    );
    println!("   - Maximum work-group total size: {}", max_wg_size);

    let max_wi_sizes = get_info!(dev, MaxWorkItemSizes);
    println!("   - Maximum work item dimensions: {:?}", max_wi_sizes);

    // Information related to memory and memory allocation.
    println!("  * Memory information");

    let local_mem_size = get_memory!(dev, LocalMemSize);
    println!("   - Local memory size: {:#?}", local_mem_size);

    let global_mem_size = get_memory!(dev, GlobalMemSize);
    println!("   - Global memory size: {:#?}", global_mem_size);

    let max_mem_alloc_size = get_memory!(dev, MaxMemAllocSize);
    println!(
        "   - Maximum memory allocation size: {:#?}",
        max_mem_alloc_size
    );
}

fn print_platform_info(pl: &ocl::Platform) {
    println!(" * {}", pl.name().expect("Failed to retrieve platform name."));
    println!(" - Vendor: {}", pl.vendor().expect("Failed to retrieve platform vendor."));
    println!(" - Version: {}", pl.version().expect("Failed to retrieve platform version."));

    let devices = ocl::Device::list_all(pl)
        .expect("Failed to list platform devices.");
    
    println!(" - Device count: {}", devices.len());
    devices.iter().for_each(print_device_info);
}

pub fn print_all_info() {
    let platforms = ocl::Platform::list();

    println!("Number of OpenCL platforms: {}", platforms.len());

    platforms.iter().for_each(print_platform_info);
}