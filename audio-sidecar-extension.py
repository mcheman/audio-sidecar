"""
must have python-nautilus installed

sudo pacman -Sy python-nautilus
sudo apt install nautilus-python

copy this file to the ~/.local/share/nautilus-python/extensions directory
"""

from gi.repository import GObject, Nautilus
from typing import List
import subprocess


class AudioSidecarExtension(GObject.GObject, Nautilus.MenuProvider):
    # VALID_MIMETYPES = ('image/png', 'image/jpeg')

    def __init__(self):
        super().__init__()

    def menu_activate_cb(
            self,
            menu: Nautilus.MenuItem,
            file: Nautilus.FileInfo,
    ) -> None:
        subprocess.Popen(
            ['/speed/programs/audio-sidecar/target/debug/audio_sidecar', file.get_location().get_path()],
            cwd='/speed/programs/audio-sidecar/')

    def get_file_items(
            self,
            files: List[Nautilus.FileInfo],
    ) -> List[Nautilus.MenuItem]:
        # if len(files) != 1 or files[0].get_mime_type() not in self.VALID_MIMETYPES:
        #     return []

        file = files[0]

        item = Nautilus.MenuItem(
            name="SimpleMenuExtension::Show_File_Name",
            label="Record Audio",
            tip="Record audio for this file. The audio will be named the same so it sorts next to the file",
        )
        item.connect("activate", self.menu_activate_cb, file)

        return [
            item,
        ]

    # Even though we're not using background items, Nautilus will generate
    # a warning if the method isn't present
    def get_background_items(
            self,
            current_folder: Nautilus.FileInfo,
    ) -> List[Nautilus.MenuItem]:
        return []
