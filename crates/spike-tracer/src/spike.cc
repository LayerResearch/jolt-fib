#include "spike-tracer/src/spike.h"
#include "riscv/cfg.h"
#include "riscv/sim.h"
#include "riscv/mmu.h"
#include "riscv/devices.h"
#include <vector>
#include <string>
#include <memory>
#include <fstream>

SpikeTracer::SpikeTracer(const rust::Str isa) : max_instructions(1000000), isa(isa.data(), isa.size()) {
}

int SpikeTracer::run(
    const rust::Str elf,
    const rust::Slice<const uint8_t> input,
    const rust::Slice<uint8_t> output,
    const rust::Str log_path
) {
    // Create configuration
    cfg_t cfg;
    cfg.isa = isa.c_str();
    cfg.priv = "MSU";  // Machine, Supervisor, User modes
    cfg.misaligned = false;  // Disable misaligned access for deterministic behavior
    cfg.endianness = endianness_little;
    cfg.pmpregions = 16;
    cfg.pmpgranularity = 4;
    cfg.real_time_clint = false;  // Disable real-time CLINT for deterministic behavior
    cfg.trigger_count = 4;
    cfg.cache_blocksz = 64;
    
    // Set up memory layout (2GB of memory)
    std::vector<mem_cfg_t> mem_layout;
    mem_layout.push_back(mem_cfg_t(0x80000000, 0x80000000)); // 2GB of memory starting at 0x80000000
    cfg.mem_layout = mem_layout;

    // Create memory regions
    std::vector<std::pair<reg_t, abstract_mem_t*>> mems;
    for (const auto& cfg : mem_layout) {
        mems.push_back(std::make_pair(cfg.get_base(), new mem_t(cfg.get_size())));
    }

    // Create simulator
    std::vector<std::string> htif_args;
    htif_args.push_back(std::string(elf.data(), elf.size()));
    
    debug_module_config_t dm_config;
    const char* log_path_ptr = log_path.size() > 0 ? log_path.data() : nullptr;
    sim_t sim(&cfg, false, mems, {}, htif_args, dm_config, log_path_ptr, true, nullptr, false, nullptr, max_instructions);

    // Configure logging for zkvm trace generation
    sim.configure_log(true, true);  // Enable both instruction trace and commit log
    sim.set_histogram(false);  // Disable histogram for cleaner logs

    // Run the simulator
    auto return_code = sim.run();

    // Clean up memory
    for (auto& mem : mems) {
        delete mem.second;
    }
    return return_code;
}

std::unique_ptr<SpikeTracer> new_spike_tracer(const rust::Str isa) {
    return std::make_unique<SpikeTracer>(isa);
}