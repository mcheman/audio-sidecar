#define SDL_MAIN_USE_CALLBACKS 1

#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>

static SDL_Window *window = NULL;
static SDL_Renderer *renderer = NULL;

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
// todo   add optional "music" for background since he wants to put specific music in the background.
// todo   slideshow will also display metadata that was entered such as title and comments etc
// todo add keyboard shortcut to nautilus extension?

// todo periodically check if new audio devices have been added (especially if none of the ideal ones are detected yet), see getaudiorecordingdevices or eventing
// todo assign flac album cover art to image it was created for with extra audio icon????

// todo load values from config file: interface text to search for,

struct ProgramConfig
{
    char* interface;
} config;

int loadConfig(char* filename);



/* This function runs once at startup. */
SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{

    loadConfig("audio-sidecar-config");

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

    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS)) {
        SDL_Log("Couldn't initialize SDL! %s", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    if (!SDL_CreateWindowAndRenderer("Record Audio", 640, 480, 0, &window, &renderer)) {
        SDL_Log("Couldn't create window/renderer! %s", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    int count;



    SDL_AudioDeviceID* ids = SDL_GetAudioRecordingDevices(&count);
    SDL_Log("Count: %d" ,count);
    if (!ids)
    {
        SDL_Log("No recording devices found!", SDL_GetError());
    }

    SDL_AudioDeviceID physicalScarlett = 0;

    for (int i = 0; i < count; i++)
    {
        const char* name = SDL_GetAudioDeviceName(ids[i]);

        SDL_Log("device[%d] name: %s", i, name);

        if (strcasestr(name, config.interface) != NULL)
        {
            // todo try to select the first input, or test both inputs to see which has any audio signal and use that one
            SDL_Log("FOUND SCARLETT: device[%d] name: %s ", i, name);
            physicalScarlett = ids[i];
        }


    }

    if (physicalScarlett == 0)
    {
        // todo display a user facing message about needing to turn it on
        SDL_Log("Couldn't find scarlett!", SDL_GetError());
    }

    // todo replace "scarlett" naming convention with generic name and put "scarlett" as a string for the preferred audio interface

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


    SDL_free(ids);
    return SDL_APP_CONTINUE;  /* carry on with the program! */
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    if (event->type == SDL_EVENT_QUIT) {
        return SDL_APP_SUCCESS;
    }
    return SDL_APP_CONTINUE;
}


SDL_AppResult SDL_AppIterate(void *appstate)
{
    SDL_FRect rect;

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);

    SDL_SetRenderDrawColor(renderer, 0, 0, 255, 255);
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
    return SDL_APP_CONTINUE;
}

int loadConfig(char* filename)
{
    // todo use sdl async io api
    SDL_Storage *readStorage = SDL_OpenTitleStorage("/speed/programs/audio-sidecar/", 0);
    if (readStorage == NULL) {
        SDL_Log("Couldn't open sdl read only storage! %s", SDL_GetError());
    }
    while (!SDL_StorageReady(readStorage)) {
        SDL_Delay(1);
    }


    Uint64 dstLen = 0;

    if (SDL_GetStorageFileSize(readStorage, filename, &dstLen) && dstLen > 0) {
        char* dst = SDL_malloc(dstLen);
        if (!SDL_ReadStorageFile(readStorage, filename, dst, dstLen)) {
            SDL_Log("Couldn't read %s: %s", filename, SDL_GetError());
        }

        // parse config file into struct
        // for (Uint64 i = 0; i < dstLen; i++)
        // {
        //     if (dst[i] == '#')
        //     {
        //         // It's a comment line, skip to end
        //         while (i < dstLen && dst[++i] != '\n');
        //     }
        //
        //     if (dst[i])
        // }

        char* line = strtok(dst, "\n");
        while (line != NULL)
        {
            if (line[0] == '#')
            {
                // comment line, ignore
            }
            else if (strncmp(line, "Interface", sizeof("Interface") - 1) == 0)
            {
                // todo better handle whitespace instead of expecting exactly one character's space between the key and the value
                // todo also do error checking for things like "interface\n" and valid values
                char* value = line + sizeof("Interface ");

                config.interface = malloc(strlen(value)); // intentionally not free'd since config has static lifetime
                if (config.interface == NULL)
                {
                    SDL_Log("Couldn't allocate memory for Interface");
                    return 1;
                }
                strcpy(config.interface, value);
            }

            line = strtok(NULL, "\n"); // get next token
        }




        SDL_free(dst);
    } else {
        SDL_Log("Couldn't find file to get size %s: %s", filename, SDL_GetError());
    }

    SDL_CloseStorage(readStorage);
    return 0;
}

char* getConfigValue(char* line, char* key)
{

}


// see https://en.wikipedia.org/wiki/WAV
struct WAV_HEADER
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

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
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

    snprintf(command, sizeof(command), "ffmpeg -y -i /tmp/output.wav -af aformat=s16:41000 -compression_level 12 '%s-audio.flac'", filepath);

    printf(command);

    system(command);
}

