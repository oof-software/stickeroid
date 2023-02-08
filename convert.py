'''convert webp emotes from 7tv to whatsapp stickers'''

import subprocess
import re
import os
import sys
import argparse
import logging as log
import time
import functools
import urllib.error
import multiprocessing as mp

import humanize
import wget

DOWNLOAD_DIR = './emotes/'
FRAME_DIR = './frames/'
RESIZED_DIR = './resized/'
STICKER_DIR = './stickers/'
SVG_DIR = './svg/'
MAX_THREADS = 16


class Emote:
    '''emote-class'''

    def __init__(self, emote_id_: str):
        assert len(emote_id_) == 24
        self.emote_id: str = emote_id_
        self.frame_delays: list[int] | None = None

    def __link(self) -> str:
        '''7tv link to the 4x version of the emote'''
        return f'https://cdn.7tv.app/emote/{self.emote_id}/4x.webp'

    def __file_name(self) -> str:
        '''filename for the emote'''
        return f'{self.emote_id}.webp'

    def __emote_path(self) -> str:
        '''path to the downloaded emote'''
        return os.path.join(DOWNLOAD_DIR, self.__file_name())

    def __sticker_path(self) -> str:
        '''path to the created sticker'''
        return os.path.join(STICKER_DIR, self.__file_name())

    def __raw_frames_dir(self) -> str:
        '''directory for extracted frames'''
        return os.path.join(FRAME_DIR, f'{self.emote_id}/')

    def __frames_dir(self) -> str:
        '''directory for resized frames'''
        return os.path.join(RESIZED_DIR, f'{self.emote_id}/')

    @staticmethod
    def __list_files_in_dir(path: str, ext: str) -> list[str]:
        files: list[str] = []
        for dir_entry in os.scandir(path):
            assert dir_entry.is_file()
            assert dir_entry.name.endswith(ext)
            files.append(dir_entry.name)
        return files

    def frame_count(self) -> int:
        '''how many frames does the animation have'''
        assert self.frame_delays is not None
        return len(self.frame_delays)

    def duration_ms(self) -> int:
        '''how long does the animation last in ms'''
        assert self.frame_delays is not None
        return functools.reduce(lambda acc, next: acc + next, self.frame_delays, 0)

    def download(self):
        '''download the emote from 7tv cdn'''
        if not os.path.exists(DOWNLOAD_DIR):
            os.mkdir(DOWNLOAD_DIR)

        dst = self.__emote_path()
        if os.path.exists(dst):
            log.warning('%s skipped download, already exists', self.emote_id)
            return

        try:
            wget.download(self.__link(), dst, bar=None)
            log.info('%s downloaded', dst)
        except urllib.error.HTTPError as ex:
            log.error('%s couldn\'t download: %s (%d)',
                      self.emote_id, ex.reason, ex.code)
            raise ex

    def extract_frames(self):
        '''extract frames from animated webp file'''
        if not os.path.exists(FRAME_DIR):
            os.mkdir(FRAME_DIR)

        dst = self.__raw_frames_dir()
        if not os.path.exists(dst):
            os.mkdir(dst)
        else:
            log.warning('%s found existing extracted frames', self.emote_id)
            return

        subprocess.run([
            'anim_dump', '-prefix', '', '-folder', dst, self.__emote_path()
        ], capture_output=True, text=True, check=True)

        log.info('%s extracted %d frames',
                 self.emote_id, len(self.frame_files_raw))

    def extract_frame_times(self):
        '''read the frame delays'''
        assert self.emote_file is not None

        output = subprocess.run([
            'webpinfo', os.path.join(DOWNLOAD_DIR, self.__file_name())
        ], capture_output=True, text=True, check=True)

        self.frame_delays = [
            int(d) for d in re.findall(r'^  Duration: (\d+)$',
                                       output.stdout, flags=re.MULTILINE)
        ]

        assert len(self.frame_delays) > 0
        log.info('%s extracted %d frame times',
                 self.emote_id, len(self.frame_delays))

    def resize_extracted_frames(self):
        '''resize the extracted frames'''
        assert self.frame_files_raw is not None
        assert len(self.frame_files_raw) > 0

        if not os.path.exists(RESIZED_DIR):
            os.mkdir(RESIZED_DIR)

        dst = self.__frames_dir()
        if not os.path.exists(dst):
            os.mkdir(dst)
        elif self.frame_files_raw is not None:
            log.warning('%s found existing resized frames', self.emote_id)

        subprocess.run([
            'ffmpeg', '-i', os.path.join(self.__raw_frames_dir(), '%04d.png'),
            '-vf', 'scale=512:512', '-y', os.path.join(
                self.__frames_dir(), '%04d.png')
        ], capture_output=True, text=True, check=True)

        self.frame_files = Emote.__list_files_in_dir(dst, '.png')
        assert len(self.frame_files) > 0

        log.info('%s resized %d extracted frames',
                 self.emote_id, len(self.frame_files))

    def build_sticker(self, compression: int, method: int):
        '''
        builds the sticker from the resized frames
        - `compression`: specify the compression factor between 0 and 100.
        - `method`: the trade off between encoding speed and the compressed file size and quality.
          possible values range from 0 to 6.
        '''
        assert self.frame_files is not None
        assert self.frame_delays is not None
        assert len(self.frame_files) > 0
        assert len(self.frame_files) == len(self.frame_delays)

        if not os.path.exists(STICKER_DIR):
            os.mkdir(STICKER_DIR)

        dst = os.path.join(STICKER_DIR, self.__file_name())
        cmd = ['img2webp', '-o', dst, '-loop', '0']
        for frame_file_name, frame_duration in zip(self.frame_files, self.frame_delays):
            frame_file_path = os.path.join(
                self.__frames_dir(), frame_file_name)
            cmd.extend(['-d', str(frame_duration), '-lossy',
                        '-q', str(compression), '-m', str(method), frame_file_path])

        start = time.perf_counter()
        subprocess.run(cmd, capture_output=True, text=True, check=True)
        elapsed_ms = (time.perf_counter() - start) * 1000
        sticker_size = humanize.naturalsize(os.path.getsize(dst), binary=True)

        log.info('%s built sticker (%d frames, took %.1fms, %s)',
                 self.emote_id, len(self.frame_delays), elapsed_ms, sticker_size)


class Svg:
    '''svg file conversion'''

    def __init__(self, file_name: str):
        self.file_name = file_name

    def __svg_path(self) -> str:
        '''path to the svg file'''
        return os.path.join(SVG_DIR, f'{self.file_name}.svg')

    def __sticker_path(self) -> str:
        '''path to the png file'''
        return os.path.join(STICKER_DIR, f'{self.file_name}.webp')

    def convert_to_png(self):
        '''convert the svg to a png'''
        if not os.path.exists(STICKER_DIR):
            os.mkdir(STICKER_DIR)

        subprocess.run([
            'magick', '-size', '512x512', '-background', 'none', self.__svg_path(),
            '-gravity', 'center', '-extent', '512x512',
            '-define', 'webp:lossless=true', self.__sticker_path()
        ], capture_output=True, check=True, text=True)
        sticker_size = humanize.naturalsize(
            os.path.getsize(self.__sticker_path()))

        log.info('%s converted svg to sticker (%s)',
                 self.file_name, sticker_size)


def check_ffmpeg():
    '''check for ffmpeg binary'''
    try:
        subprocess.run(['ffmpeg', '-version'],
                       capture_output=True, text=True, check=True)
        log.info('found dependency ffmpeg')
    except FileNotFoundError as ex:
        log.error('missing dependency ffmpeg: %s', ex.strerror)
        sys.exit(-1)


def check_animdump():
    '''check for animdump binary'''
    try:
        subprocess.run(['anim_dump', '-version'],
                       capture_output=True, text=True, check=True)
        log.info('found dependency animdump')
    except FileNotFoundError as ex:
        log.error('missing dependency animdump: %s', ex.strerror)
        sys.exit(-1)


def check_img2webp():
    '''check for img2webp binary'''
    try:
        subprocess.run(['img2webp', '-version'],
                       capture_output=True, text=True, check=True)
        log.info('found dependency img2webp')
    except FileNotFoundError as ex:
        log.error('missing dependency img2webp: %s', ex.strerror)
        sys.exit(-1)


def check_image_magick():
    '''check for magick binary'''
    try:
        subprocess.run(['magick', '-version'],
                       capture_output=True, text=True, check=True)
        log.info('found dependency image magick')
    except FileNotFoundError as ex:
        log.error('missing dependency image magick: %s', ex.strerror)
        sys.exit(-1)


def check_dependencies():
    '''check_dependencies'''
    check_ffmpeg()
    check_animdump()
    check_img2webp()
    check_image_magick()


def process_emote(emote: Emote):
    '''process emote'''
    try:
        log.basicConfig(encoding='utf-8', level=log.INFO)
        emote.download()
        emote.extract_frame_times()
        emote.extract_frames()
        emote.resize_extracted_frames()
        emote.build_sticker(50, 6)
    except urllib.error.HTTPError:
        return


def process_svg(svg: Svg):
    '''process svg'''
    log.basicConfig(encoding='utf-8', level=log.INFO)
    svg.convert_to_png()


if __name__ == '__main__':
    log.basicConfig(encoding='utf-8', level=log.INFO)

    check_dependencies()

    parser = argparse.ArgumentParser(
        prog='converter',
        description='Convert 7TV emotes and SVG files to WhatsApp stickers')

    parser.add_argument('--test', action='store_true',
                        help='don\'t process anything, just parse the arguments')
    parser.add_argument('--parallel', type=int, choices=range(1, MAX_THREADS + 1),
                        metavar=f'[1-{MAX_THREADS}]', default=1,
                        help='how many elements should be processed in parallel')
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--svg', nargs='+', type=str,
                       help='filenames of files in the \'./svg/\' directory, \
                        without the \'.svg\' extension')
    group.add_argument('--7tv', nargs='+', type=str,
                       help='ids of the 7tv emotes to process')

    args = vars(parser.parse_args())

    if args['7tv'] is not None:
        regex = re.compile(r'^[0-9a-f]{24}$')
        for emote_id in args['7tv']:
            if regex.match(emote_id) is None:
                parser.error(f'invalid emote_id ({emote_id})')

        if args['test']:
            sys.exit(0)

        emotes = [Emote(id) for id in args['7tv']]

        if args['parallel'] > 1:
            with mp.Pool(args['parallel']) as pool:
                pool.map(process_emote, emotes)
        else:
            for emote_ in emotes:
                process_emote(emote_)

    elif args['svg'] is not None:
        for svg_name in args['svg']:
            svg_path = os.path.join(SVG_DIR, f'{svg_name}.svg')
            if not os.path.exists(svg_path):
                parser.error(f'svg file \'{svg_path}\' doesn\'t exist')

        if args['test']:
            sys.exit(0)

        svgs = [Svg(file_name_)for file_name_ in args['svg']]

        if args['parallel'] > 1:
            with mp.Pool(args['parallel']) as pool:
                pool.map(process_svg, svgs)
        else:
            for svg_ in svgs:
                process_svg(svg_)
