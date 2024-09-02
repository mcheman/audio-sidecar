cmake_minimum_required(VERSION 3.29)
project(audio_sidecar C)

set(CMAKE_C_STANDARD 11)



# See here: https://wiki.libsdl.org/SDL3/README/cmake
include(FetchContent)
set(SDL_STATIC TRUE CACHE BOOL "Build a SDL static library (if available)")
FetchContent_Declare(
        SDL
        GIT_REPOSITORY https://github.com/libsdl-org/SDL.git
        GIT_TAG main  # Replace this with a particular git tag or git hash
        GIT_SHALLOW TRUE
        GIT_PROGRESS TRUE
)
message(STATUS "Using SDL3 via FetchContent")
FetchContent_MakeAvailable(SDL)
set_property(DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/_deps/sdl-src" PROPERTY EXCLUDE_FROM_ALL TRUE)




add_executable(audio_sidecar
        main.c)

# Static link to the actual SDL3 library.
target_link_libraries(audio_sidecar PRIVATE SDL3::SDL3-static)