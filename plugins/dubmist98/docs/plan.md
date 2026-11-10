# Dubmist 98: implementation plan

Current status: milestones 1?2 and a bounded milestone 3a rhythmic slice mode
are implemented. See rhythm-milestone.md and next-step-note.md for exact scope.
The original plan is retained below; crossover/bass preservation remains open.

1. Inspect and pin original Clouds, Monsoon, Kammerl, Parasites and a desktop port.
2. Compile original Clouds/stmlib (including portable ShyFFT) as a separate C++ core.
3. Preserve 32 kHz/32-frame processing and original memory sizes. Add bounded,
   causal windowed-sinc input/output conversion rather than changing constants.
4. Build a stereo JUCE/GPLv3 VST3 with all four original modes, four independent
   blend controls, quality, freeze, trigger, clear, compensated bypass and gains.
5. Add complete versioned parameter-only state, presets, native workstation UI.
6. Test original-reference equality, stereo, resampling, block partitioning,
   freeze/live capture, feedback extremes, allocations, host state and actual VST3.
7. Ship this coherent milestone 1-2 preview with real captures and explicit limits.

Later milestones: Kammerl-derived clocked slices with sample-aligned scheduling
and seek/stop/tempo tests; bass preservation with measured complementary crossover
and path latency alignment. Neither will be represented by nonfunctional controls.
