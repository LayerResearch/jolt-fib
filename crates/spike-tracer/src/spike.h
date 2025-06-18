#pragma once

#include <cstdint>
#include <memory>
#include "rust/cxx.h"

class SpikeTracer {
private:
    std::string isa;
public:
    SpikeTracer(const rust::Str isa);
    
    // Using rust::String in C++
    int run(const rust::Str elf, const rust::Slice<const uint8_t> input, const rust::Slice<uint8_t> output);

private:
    uint64_t max_instructions;
};

std::unique_ptr<SpikeTracer> new_spike_tracer(const rust::Str isa);