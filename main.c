/*
 * This example code creates an simple audio stream for playing sound, and
 * generates a sine wave sound effect for it to play as time goes on. This
 * is the simplest way to get up and running with procedural sound.
 *
 * This code is public domain. Feel free to use it for any purpose!
 */

#define SDL_MAIN_USE_CALLBACKS 1  /* use the callbacks instead of main() */
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>

/* We will use this renderer to draw into this window every frame. */
static SDL_Window *window = NULL;
static SDL_Renderer *renderer = NULL;
static SDL_AudioStream *stream = NULL;

static FILE *output_wav = NULL;
static short output_buffer[44100 * 60 * 60]; // 1 hour of audio buffer
static int output_buffer_index = 0;

static SDL_AudioStream *scarlettStream = NULL;

#define MAX_PATH_LENGTH 1024
static char filepath[MAX_PATH_LENGTH]; // todo check if max len is acceptable

// todo add error checking, logging, and dad friendly error reporting
// todo add safeguards such as not overwriting existing recordings and/or saving old recordings to a backup directory on overwrite
// todo append new data to file and fixup header when recording to an existing file
// todo show time recorded so far
// todo add big button to stop recording/exit
// todo handle multiple paths sent to this program i.e. drop everything after first
// todo clean up visualization
// todo warn when clipping occurs
// todo figure out how to add to right click menu in nautilus without additional click into scripts submenu
// todo write ffmpeg command such that it will never prompt for user input, such as when attempting to overwrite a file
// todo pin sdl3 version
// todo test on other distros
// todo organize better / refactor / split into separate source files
// todo see if you can get 24bit audio working
// todo add message when quitting if writing out is taking awhile (though probably not needed if writing out as we go)
// todo create a slideshow application that plays the audio with the corresponding picture, advancing to the next once the audio is done. slideshow will play everything in directory
// todo   add optional "music" for background since dad wants to put specific music in the background.
// todo   slideshow will also display metadata that dad has entered such as title and comments etc
// todo add keyboard shortcut to nautilus extension?


/* This function runs once at startup. */
SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{

    if (argc > 1)
    {
        int len = strlen(argv[1]);
        if (len > MAX_PATH_LENGTH - 11) // -11 for "-audio.flac" suffix
        {
            len = MAX_PATH_LENGTH - 11;
        }
        // todo remove old file extension first
        memcpy(filepath, argv[1], len);
        // walk backward and replace first "." with null to remove the file extension. todo do this better
        for (int i = len; i > 1; i--)
        {
            if (filepath[i] == '/')
            {
                break; // do not continue past the filename
            }

            if (filepath[i] == '.')
            {
                filepath[i] = '\0';
                break;
            }
        }
    } else
    {
        memcpy(filepath, "/tmp/output", 11);
    }

    SDL_AudioSpec spec;

    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS)) {
        SDL_Log("Couldn't initialize SDL! %s", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    /* we don't _need_ a window for audio-only things but it's good policy to have one. */
    if (!SDL_CreateWindowAndRenderer("examples/audio/simple-playback", 640, 480, 0, &window, &renderer)) {
        SDL_Log("Couldn't create window/renderer! %s", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    /* We're just playing a single thing here, so we'll use the simplified option.
       We are always going to feed audio in as mono, float32 data at 8000Hz.
       The stream will convert it to whatever the hardware wants on the other side. */
    // spec.channels = 1;
    // spec.format = SDL_AUDIO_F32;
    // spec.freq = 8000;
    // stream = SDL_OpenAudioDeviceStream(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, &spec, NULL, NULL);
    // if (!stream) {
    //     SDL_Log(SDL_MESSAGEBOX_ERROR, "Couldn't create audio stream!", SDL_GetError(), window);
    //     return SDL_APP_FAILURE;
    // }
    //
    // /* SDL_OpenAudioDeviceStream starts the device paused. You have to tell it to start! */
    // SDL_ResumeAudioStreamDevice(stream);

    int count;

    printf("sizeof(short): %d\n", (int) sizeof(short));

    // todo grab the "Scarlett" interface (case insensitive) and use that for recording. if it's not there display a message about needing to turn it on?

    SDL_AudioDeviceID* ids = SDL_GetAudioRecordingDevices(&count);
    SDL_Log("Count: %d" ,count);
    if (!ids)
    {
        SDL_Log("No recording devices found!", SDL_GetError());
    }

    for (int i = 0; i < count; i++)
    {
        const char* name = SDL_GetAudioDeviceName(ids[i]);

        SDL_Log("device[%d] name: %s", i, name);
    }

    SDL_AudioDeviceID physicalScarlett = ids[0];


    // scarlett is the opened logical device that points at the physical scarlett device
    SDL_AudioDeviceID scarlettDevice = SDL_OpenAudioDevice(physicalScarlett, NULL);
    // todo replace null with ideal recording spec
    // todo check errors of sdl functions


    SDL_AudioSpec src_spec = {
        .freq = 44100,
        .format = SDL_AUDIO_F32,
        .channels = 1,
    };
    SDL_AudioSpec dst_spec = {
        .freq = 44100,
        .format = SDL_AUDIO_S16,
        .channels = 1,
    };

    scarlettStream = SDL_CreateAudioStream(&src_spec, &dst_spec);
    // todo is this thread safe to pass pointer to stack data?

    SDL_BindAudioStream(scarlettDevice, scarlettStream);


    // SDL_Log("Err: %s", SDL_GetError());



    SDL_free(ids);
    return SDL_APP_CONTINUE;  /* carry on with the program! */
}

/* This function runs when a new event (mouse input, keypresses, etc) occurs. */
SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    if (event->type == SDL_EVENT_QUIT) {
        return SDL_APP_SUCCESS;  /* end the program, reporting success to the OS. */
    }
    return SDL_APP_CONTINUE;  /* carry on with the program! */
}

/* This function runs once per frame, and is the heart of the program. */
SDL_AppResult SDL_AppIterate(void *appstate)
{
    /* see if we need to feed the audio stream more data yet.
       We're being lazy here, but if there's less than half a second queued, generate more.
       A sine wave is unchanging audio--easy to stream--but for video games, you'll want
       to generate significantly _less_ audio ahead of time! */
    // const int minimum_audio = (8000 * sizeof (float)) / 2;  /* 8000 float samples per second. Half of that. */
    // if (SDL_GetAudioStreamAvailable(stream) < minimum_audio) {
    //     static float samples[512];  /* this will feed 512 samples each frame until we get to our maximum. */
    //     int i;
    //
    //     for (i = 0; i < SDL_arraysize(samples); i++) {
    //         /* You don't have to care about this math; we're just generating a simple sine wave as we go.
    //            https://en.wikipedia.org/wiki/Sine_wave */
    //         const float time = total_samples_generated / 8000.0f;
    //         const int sine_freq = 500;   /* run the wave at 500Hz */
    //         samples[i] = SDL_sinf(6.283185f * sine_freq * time);
    //         total_samples_generated++;
    //     }
    //
    //     /* feed the new data to the stream. It will queue at the end, and trickle out as the hardware needs more data. */
    //     SDL_PutAudioStreamData(stream, samples, sizeof (samples));
    // }
    //

    /* we're not doing anything with the renderer, so just blank it out. */
    SDL_FRect rect;

    /* as you can see from this, rendering draws over whatever was drawn before it. */
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);  /* black, full alpha */
    SDL_RenderClear(renderer);  /* start with a blank canvas. */

    /* draw a filled rectangle in the middle of the canvas. */
    SDL_SetRenderDrawColor(renderer, 0, 0, 255, 255);  /* blue, full alpha */
    rect.x = rect.y = 100;
    rect.w = 440;
    rect.h = 300;
    SDL_RenderFillRect(renderer, &rect);



    short data[44100] = {0};
    int bytesRead = SDL_GetAudioStreamData(scarlettStream, &data, sizeof(data));


    for (int i = 0; i < bytesRead / sizeof(short); i++)
    {
        if (output_buffer_index < sizeof(output_buffer) / sizeof(short))
        {
            output_buffer[output_buffer_index] = data[i];
            output_buffer_index++;
        }
    }




        // amplitude = (average * 300) /2500;
        // amplitude = (max * 300) / (SHRT_MAX / 2);

        // SDL_Log("%d", average);


    SDL_SetRenderDrawColor(renderer, 255, 150, 255, 255);  /* blue, full alpha */

    int numBars = 100;
    rect.y = 100;
    rect.w = 440 / numBars;

    int averageSamples = output_buffer_index / numBars;
    for (int i = 0; i < numBars; i++)
    {
        rect.x = 100 + rect.w * i;

        int max = SHRT_MIN;
        for (int j = 0; j < averageSamples; j++)
        {
            if (output_buffer[averageSamples * i + j] > max)
            {
                max = output_buffer[averageSamples * i + j];
            }
        }
        rect.h = (max * 300) / (SHRT_MAX / 2);

        SDL_RenderFillRect(renderer, &rect);

    }


    SDL_RenderPresent(renderer);
    return SDL_APP_CONTINUE;  /* carry on with the program! */
}


// see https://en.wikipedia.org/wiki/WAV
struct WAV_HEADER // little endian
{
    char FileTypeBlockID[4];
    uint32_t FileSize;
    char FileFormatID[4];

    char FormatBlocID[4];
    uint32_t BlocSize;
    uint16_t AudioFormat;
    uint16_t NbrChannels;
    uint32_t Frequency;
    uint32_t BytePerSec;
    uint16_t BytePerBloc;
    uint16_t BitsPerSample;

    uint32_t DataBlockID;
    uint32_t DataSize;
    // sampled data
};

int writeAudio(FILE *file, short *data, int length)
{
    // todo write audio periodically so a crash doesn't lose data



    struct WAV_HEADER header = {
        .FileTypeBlockID = "RIFF",
        .FileSize = sizeof(header) + (sizeof(short) * length) - 8, // (overall filesize - 8 bytes) part of standard
        .FileFormatID = "WAVE",

        .FormatBlocID = "fmt ",
        .BlocSize = 16,
        .AudioFormat = 1,
        .NbrChannels = 1,
        .Frequency = 44100,
        .BytePerSec = 44100 * sizeof(short),
        .BytePerBloc = 1 * sizeof(short),
        .BitsPerSample = 16,

        .DataBlockID = 0x61746164, // data
        .DataSize = length * sizeof(short),

    };

    fwrite(&header, 1, sizeof(header), file);
    fwrite(data, sizeof(short), length, file);
    fflush(file);
    fclose(file);
}

/* This function runs once at shutdown. */
void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    /* SDL will clean up the window/renderer for us. */

    SDL_FlushAudioStream(scarlettStream);

    int bytesRead = 0;
    do
    {
        short data[44100] = {0};
        bytesRead = SDL_GetAudioStreamData(scarlettStream, &data, sizeof(data));

        for (int i = 0; i < bytesRead / sizeof(short); i++)
        {
            if (output_buffer_index < sizeof(output_buffer) / sizeof(short))
            {
                output_buffer[output_buffer_index] = data[i];
                output_buffer_index++;
            }
        }
    } while (bytesRead > 0);

    printf("duration: %f", output_buffer_index / 44100.0);
    printf("length: %d", output_buffer_index);

    // todo do not use a hard coded temp file in case multiple recording programs are opened/crashed at once

    output_wav = fopen("/tmp/output.wav", "wb");
    writeAudio(output_wav, output_buffer, output_buffer_index);

    char command[MAX_PATH_LENGTH + 1000];

    snprintf(command, sizeof(command), "ffmpeg -i /tmp/output.wav -af aformat=s16:41000 -compression_level 12 '%s-audio.flac'", filepath);

    printf(command);

    system(command);
}

