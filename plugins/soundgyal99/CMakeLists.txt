cmake_minimum_required(VERSION 3.25)
project(soundgyal99 VERSION 0.3.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
file(GLOB_RECURSE PLAITS CONFIGURE_DEPENDS vendor/plaits/dsp/*.cc)
add_library(soundgyal_core ${PLAITS} vendor/plaits/resources.cc vendor/stmlib/dsp/units.cc vendor/stmlib/dsp/atan.cc vendor/stmlib/utils/random.cc src/engine.cpp src/synth.cpp)
target_include_directories(soundgyal_core PUBLIC src vendor)
target_compile_definitions(soundgyal_core PUBLIC TEST _USE_MATH_DEFINES)
if(MSVC)
 target_compile_options(soundgyal_core PUBLIC /permissive- /utf-8 /FIcstdio /wd4244 /wd4267 /wd4305)
endif()
add_executable(soundgyal_validate tests/validate.cpp)
target_link_libraries(soundgyal_validate PRIVATE soundgyal_core)
enable_testing()
add_test(NAME original_engines COMMAND soundgyal_validate)

option(SOUNDGYAL_BUILD_PLUGIN "Build VST3 and validation host" ON)
if(SOUNDGYAL_BUILD_PLUGIN)
 set(JUCE_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/soundgyal-deps/JUCE" CACHE PATH "JUCE 7.0.12 source")
 if(NOT EXISTS "${JUCE_SOURCE_DIR}/CMakeLists.txt")
  message(FATAL_ERROR "Run scripts/bootstrap.ps1 to fetch pinned JUCE sources")
 endif()
 add_subdirectory("${JUCE_SOURCE_DIR}" "${CMAKE_CURRENT_BINARY_DIR}/juce")
 juce_add_plugin(Soundgyal99
  COMPANY_NAME "Voidmutt Enterprises"
  BUNDLE_ID "org.voidmutt.soundgyal99"
  PLUGIN_MANUFACTURER_CODE Vmtt
  PLUGIN_CODE Sg99
  PRODUCT_NAME "SOUNDGYAL 99"
  DESCRIPTION "SOUNDGYAL 99 Enterprise Edition"
  VERSION 0.3.0
  IS_SYNTH TRUE NEEDS_MIDI_INPUT TRUE NEEDS_MIDI_OUTPUT FALSE
  IS_MIDI_EFFECT FALSE EDITOR_WANTS_KEYBOARD_FOCUS TRUE
  COPY_PLUGIN_AFTER_BUILD FALSE
  VST3_CATEGORIES Instrument Synth
  FORMATS VST3)
 target_sources(Soundgyal99 PRIVATE plugin/Processor.cpp plugin/Editor.cpp)
 target_include_directories(Soundgyal99 PUBLIC plugin)
 target_compile_definitions(Soundgyal99 PUBLIC JUCE_DISPLAY_SPLASH_SCREEN=0 JUCE_REPORT_APP_USAGE=0 JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_VST3_CAN_REPLACE_VST2=0)
 target_link_libraries(Soundgyal99 PUBLIC soundgyal_core juce::juce_audio_utils PRIVATE juce::juce_recommended_config_flags)
endif()

if(SOUNDGYAL_BUILD_PLUGIN)
 juce_add_console_app(soundgyal_plugin_tests PRODUCT_NAME soundgyal_plugin_tests)
 target_sources(soundgyal_plugin_tests PRIVATE tests/plugin_tests.cpp)
 target_link_libraries(soundgyal_plugin_tests PRIVATE Soundgyal99)
 add_test(NAME plugin_state_factory_rates COMMAND soundgyal_plugin_tests "${CMAKE_CURRENT_BINARY_DIR}/validation")
 juce_add_console_app(soundgyal_host PRODUCT_NAME soundgyal_host)
 target_sources(soundgyal_host PRIVATE tests/vst3_host.cpp)
 target_compile_definitions(soundgyal_host PRIVATE JUCE_MODAL_LOOPS_PERMITTED=1 JUCE_DISPLAY_SPLASH_SCREEN=0 JUCE_REPORT_APP_USAGE=0 JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_PLUGINHOST_VST3=1 JUCE_VST3_CAN_REPLACE_VST2=0)
 target_link_libraries(soundgyal_host PRIVATE juce::juce_audio_utils)
endif()

add_executable(soundgyal_synth_tests tests/synth_tests.cpp)
target_link_libraries(soundgyal_synth_tests PRIVATE soundgyal_core)
add_test(NAME realtime_synth COMMAND soundgyal_synth_tests)
